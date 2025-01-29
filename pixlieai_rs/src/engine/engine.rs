use super::{Engine, Node, NodeId, NodeWorker, Payload, PendingNode, RelationType};
use crate::entity::web::{Domain, Link};
use chrono::Utc;
use log::{error, info};
use postcard::{from_bytes, to_allocvec};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rocksdb::DB;
use std::time::Instant;
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

impl Engine {
    pub fn new(storage_root: PathBuf) -> Engine {
        let engine = Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: RwLock::new(HashMap::new()),
            nodes_to_write: RwLock::new(vec![]),
            last_node_id: Mutex::new(0),
            storage_root: storage_root.to_str().unwrap().to_string(),
            node_ids_by_label: RwLock::new(HashMap::new()),
        };
        engine
    }

    pub fn new_with_project(storage_root: &String, project_name: &String) -> Engine {
        let mut storage_dir = PathBuf::from(storage_root);
        storage_dir.push(format!("{}.rocksdb", project_name));
        let mut engine = Engine::new(storage_dir);
        engine.load_from_disk();
        engine
    }

    pub fn execute(&self) {
        let updated = self.process_nodes();
        let added = self.add_pending_nodes();
        if updated || added {
            self.save_to_disk();
        }
    }

    pub fn process_nodes(&self) -> bool {
        let updates: Vec<(NodeId, Option<Payload>)> = match self.nodes.read() {
            Ok(nodes) => nodes
                .iter()
                .map(|(node_id, node)| {
                    let node = node.read().unwrap();
                    let node_id = node_id.clone();
                    match node.payload {
                        Payload::Link(ref payload) => {
                            let update = payload.process(self, &node_id);
                            match update {
                                Some(payload) => (node_id, Some(Payload::Link(payload))),
                                None => (node_id, None),
                            }
                        }
                        Payload::FileHTML(ref payload) => {
                            let update = payload.process(self, &node_id);
                            match update {
                                Some(payload) => (node_id, Some(Payload::FileHTML(payload))),
                                None => (node_id, None),
                            }
                        }
                        _ => (node_id, None),
                    }
                })
                .collect(),
            Err(_err) => vec![],
        };

        let count = updates.len();
        for (node_id, update) in updates {
            match update {
                Some(update) => match self.nodes.read() {
                    Ok(nodes) => match nodes.get(&node_id) {
                        Some(node) => match node.write() {
                            Ok(mut node) => node.payload = update,
                            Err(_err) => {}
                        },
                        None => {}
                    },
                    Err(_err) => {}
                },
                None => {}
            };
        }
        count > 0
    }

    pub fn add_node(&self, payload: Payload) -> NodeId {
        // Get new ID after incrementing existing node ID
        let label = payload.to_string();
        let id = Arc::new({
            let mut id = self.last_node_id.lock().unwrap();
            *id += 1;
            *id
        });
        // Store the label in the engine
        {
            match self.labels.write() {
                Ok(mut labels) => {
                    labels.insert(label.clone());
                }
                Err(_err) => {}
            }
        };
        // Store the node in the engine
        {
            match self.nodes.write() {
                Ok(mut nodes) => {
                    nodes.insert(
                        id.clone(),
                        RwLock::new(Node {
                            id: id.clone(),
                            label: label.clone(),
                            payload,

                            parent_id: None,
                            part_node_ids: vec![],
                            related_node_ids: vec![],
                            written_at: Utc::now(),
                        }),
                    );
                }
                Err(_err) => {}
            }
        }
        // Store the node in nodes_by_label_id
        {
            let mut nodes_by_label = self.node_ids_by_label.write().unwrap();
            nodes_by_label
                .entry(label.clone())
                .and_modify(|entries| entries.push(id.clone()))
                .or_insert(vec![id.clone()]);
        }
        id
    }

    pub fn add_pending_nodes(&self) -> bool {
        let mut count = 0;
        let mut nodes_to_write: Vec<PendingNode> =
            self.nodes_to_write.write().unwrap().drain(..).collect();
        while let Some(pending_node) = nodes_to_write.pop() {
            let id = if let Some(existing_node_id) = self.find_existing(&pending_node.payload) {
                // If the payload already exists in the graph, we simply add a relation edge
                existing_node_id
            } else {
                self.add_node(pending_node.payload)
            };
            // Add a relation edge or part edge from the parent node to the new node
            match self.nodes.read() {
                Ok(nodes) => match nodes.get(&pending_node.creating_node_id) {
                    Some(node) => match pending_node.related_type {
                        RelationType::IsPart => {
                            node.write().unwrap().part_node_ids.push(id.clone());
                        }
                        RelationType::IsRelated => {
                            node.write().unwrap().related_node_ids.push(id.clone());
                        }
                    },
                    None => {}
                },
                Err(_err) => {}
            }
            // Add a relation edge from the new node to the parent node
            match self.nodes.read() {
                Ok(nodes) => match nodes.get(&id) {
                    Some(node) => match pending_node.related_type {
                        // RelationType::IsPart => {
                        //     node.write().unwrap().part_node_ids.push(pending_node.creating_node_id);
                        // }
                        RelationType::IsRelated => {
                            node.write()
                                .unwrap()
                                .related_node_ids
                                .push(pending_node.creating_node_id);
                        }
                        _ => {}
                    },
                    None => {}
                },
                Err(_err) => {}
            }
            count += 1;
        }
        if count > 0 {
            info!("Added {} nodes", count);
        }
        count > 0
    }

    pub fn add_part_node(&self, parent_id: &NodeId, payload: Payload) {
        self.nodes_to_write.write().unwrap().push(PendingNode {
            payload,
            creating_node_id: parent_id.clone(),
            related_type: RelationType::IsPart,
        });
    }

    pub fn add_related_node(&self, parent_id: &NodeId, payload: Payload) {
        self.nodes_to_write.write().unwrap().push(PendingNode {
            payload,
            creating_node_id: parent_id.clone(),
            related_type: RelationType::IsRelated,
        });
    }

    fn find_existing(&self, payload: &Payload) -> Option<Arc<u32>> {
        // For certain node payloads, check if there is a node with the same payload
        match payload {
            Payload::Domain(ref domain) => {
                // We do not want duplicate domains in the graph
                match self.nodes.read() {
                    Ok(nodes) => nodes.par_iter().find_map_any(|other_node| {
                        match other_node.1.read().unwrap().payload {
                            Payload::Domain(ref other_domain) => {
                                if domain == other_domain {
                                    Some(other_node.0.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }),
                    Err(_err) => None,
                }
            }
            Payload::Link(ref link) => {
                // We do not want duplicate links in the graph
                match self.nodes.read() {
                    Ok(nodes) => nodes.par_iter().find_map_any(|other_node| {
                        match other_node.1.read().unwrap().payload {
                            Payload::Link(ref other_link) => {
                                if link == other_link {
                                    Some(other_node.0.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }),
                    Err(_err) => None,
                }
            }
            Payload::Label(ref label) => {
                // We do not want duplicate labels in the graph
                match self.nodes.read() {
                    Ok(nodes) => nodes.par_iter().find_map_any(|other_node| {
                        match other_node.1.read().unwrap().payload {
                            Payload::Label(ref other_label) => {
                                if label == other_label {
                                    Some(other_node.0.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }),
                    Err(_err) => None,
                }
            }
            Payload::NamedEntity(ref label, ref text) => {
                // We do not want duplicate named entities in the graph
                match self.nodes.read() {
                    Ok(nodes) => nodes.par_iter().find_map_any(|other_node| {
                        match other_node.1.read().unwrap().payload {
                            Payload::NamedEntity(ref other_label, ref other_text) => {
                                if label == other_label && text == other_text {
                                    Some(other_node.0.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }),
                    Err(_err) => None,
                }
            }
            _ => None,
        }
    }

    pub fn save_to_disk(&self) {
        // We use RocksDB to store the graph
        let db = DB::open_default(&self.storage_root).unwrap();
        match self.nodes.read() {
            Ok(nodes) => {
                for (node_id, node) in nodes.iter() {
                    let bytes = match node.read() {
                        Ok(node) => match to_allocvec(&*node) {
                            Ok(bytes) => bytes,
                            Err(err) => {
                                error!("Error serializing node: {}", err);
                                break;
                            }
                        },
                        Err(err) => {
                            error!("Error reading node: {}", err);
                            break;
                        }
                    };
                    match db.put(node_id.to_le_bytes(), bytes) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error writing node: {}", err);
                            break;
                        }
                    }
                }
            }
            Err(_err) => {}
        }
    }

    pub fn load_from_disk(&mut self) {
        let db = DB::open_default(&self.storage_root).unwrap();
        let iter = db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, value) = match item {
                Ok(item) => item,
                Err(err) => {
                    error!("Error iterating over RocksDB: {}", err);
                    break;
                }
            };
            let node_id = Arc::new(read_le_u32(&mut &*key));
            let node: Node = match from_bytes(&value) {
                Ok(node) => node,
                Err(err) => {
                    error!("Error deserializing node: {}", err);
                    break;
                }
            };
            let label = node.payload.to_string().clone();
            {
                match self.nodes.write() {
                    Ok(mut nodes) => {
                        nodes.insert(node_id.clone(), RwLock::new(node));
                    }
                    Err(_err) => {}
                }
            }

            // Store the node in nodes_by_label_id
            self.node_ids_by_label
                .write()
                .unwrap()
                .entry(label)
                .and_modify(|entries| entries.push(node_id.clone()))
                .or_insert(vec![node_id.clone()]);
        }
    }

    pub fn can_fetch_within_domain(&self, node_id: &NodeId, link: &Link) -> bool {
        // Get the related domain node for the URL from the engine
        let (domain, domain_node_id) = match self.nodes.read() {
            Ok(nodes) => match nodes.get(node_id) {
                Some(node) => {
                    match node.read().unwrap().related_node_ids.iter().find_map(
                        |node_id| match self.nodes.read() {
                            Ok(nodes) => match nodes.get(node_id) {
                                Some(node) => match node.read() {
                                    Ok(node) => match node.payload {
                                        Payload::Domain(ref domain) => {
                                            Some((domain.clone(), node_id.clone()))
                                        }
                                        _ => None,
                                    },
                                    Err(_err) => None,
                                },
                                None => None,
                            },
                            Err(_err) => None,
                        },
                    ) {
                        Some((domain, domain_node_id)) => (domain, domain_node_id),
                        None => {
                            error!("Can not find domain for link: {}", &link.url);
                            return false;
                        }
                    }
                }
                None => {
                    error!("Can not find domain for link: {}", &link.url);
                    return false;
                }
            },
            Err(_err) => {
                error!("Can not find domain for link: {}", &link.url);
                return false;
            }
        };

        if !domain.is_allowed_to_crawl {
            error!("Domain is not allowed to crawl: {}", &domain.name);
            return false;
        }

        // Check the last fetch time for this domain. We do not want to fetch too often.
        match domain.last_fetched_at {
            Some(start) => {
                if start.elapsed().as_secs() > 2 {
                    // We have fetched from this domain some time ago, we can fetch now
                } else {
                    // We have fetched from this domain very recently, we can not fetch now
                    return false;
                }
            }
            None => {
                // We have not fetched from this domain before, we should fetch now
            }
        }

        // Update the domain at the domain node id
        match self.nodes.write() {
            Ok(mut nodes) => match nodes.get_mut(&domain_node_id) {
                Some(node) => match node.write() {
                    Ok(mut node) => {
                        node.payload = Payload::Domain(Domain {
                            name: domain.name.clone(),
                            is_allowed_to_crawl: true,
                            last_fetched_at: Some(Instant::now()),
                        });
                    }
                    Err(_err) => {
                        return false;
                    }
                },
                None => {
                    return false;
                }
            },
            Err(_err) => {
                return false;
            }
        }

        true
    }
}

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}
