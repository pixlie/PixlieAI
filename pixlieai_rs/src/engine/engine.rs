use super::{CommonEdgeLabels, EdgeLabel, Node, NodeId, NodeLabel, NodeWorker, Payload};
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

struct PendingRootNode {
    id: NodeId,
    payload: Payload,
}

struct PendingRelatedNode {
    id: NodeId,                          // New node ID
    payload: Payload,                    // New node payload
    parent_node_id: NodeId,              // Who is creating this node (parent)
    edge_labels: (EdgeLabel, EdgeLabel), // From parent to child and back
}

enum PendingNode {
    Root(PendingRootNode),
    Related(PendingRelatedNode),
}

struct LastTick {
    // ran_at: Option<Instant>,
    nodes_added: usize,
    nodes_updated: usize,
}

// The engine keeps track of all the data nodes and their relationships
// All the entity labels are loaded in the engine
// All data may not be loaded in the engine, some of them may be on disk
pub struct Engine {
    pub labels: RwLock<HashSet<NodeLabel>>,
    pub nodes: RwLock<HashMap<NodeId, RwLock<Node>>>, // All nodes that are in the engine
    pending_nodes: RwLock<Vec<PendingNode>>, // Nodes pending to be written at the end of nodes.iter_mut()
    last_node_id: Mutex<u32>,
    pub node_ids_by_label: RwLock<HashMap<String, Vec<NodeId>>>,
    last_tick: LastTick,
}

pub type LockedEngine = RwLock<Engine>;

impl Engine {
    fn new() -> Engine {
        let engine = Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: RwLock::new(HashMap::new()),
            pending_nodes: RwLock::new(vec![]),
            last_node_id: Mutex::new(0),
            node_ids_by_label: RwLock::new(HashMap::new()),
            last_tick: LastTick {
                // ran_at: None,
                nodes_added: 0,
                nodes_updated: 0,
            },
        };
        engine
    }

    pub fn open_project(storage_root: &String, project_id: &String) -> Engine {
        let mut storage_path = PathBuf::from(storage_root);
        storage_path.push(format!("{}.rocksdb", project_id));
        let mut engine = Engine::new();
        engine.load_from_disk(&storage_path.to_str().unwrap().to_string());
        engine
    }

    pub fn tick(&self, storage_root: &String) -> (bool, bool) {
        let updated = self.process_nodes();
        let added = self.add_pending_nodes();
        if added || updated {
            self.save_to_disk(storage_root);
        }
        (added, updated)
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

    fn save_node(&self, id: NodeId, payload: Payload) {
        // Get new ID after incrementing existing node ID
        let label = payload.to_string();
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
                            payload,

                            labels: vec![],
                            edges: HashMap::new(),
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
        };
    }

    fn add_pending_nodes(&self) -> bool {
        let mut count = 0;
        let mut nodes_to_write: Vec<PendingNode> =
            self.pending_nodes.write().unwrap().drain(..).collect();
        while let Some(pending_node) = nodes_to_write.pop() {
            match pending_node {
                PendingNode::Root(pending_root_node) => {
                    self.save_node(pending_root_node.id, pending_root_node.payload);
                }
                PendingNode::Related(pending_related_node) => {
                    self.save_node(
                        pending_related_node.id.clone(),
                        pending_related_node.payload,
                    );
                    // Add a connection edge from the parent node to the new node
                    match self.nodes.read() {
                        Ok(nodes) => {
                            match nodes.get(&pending_related_node.parent_node_id.clone()) {
                                Some(node) => {
                                    node.write()
                                        .unwrap()
                                        .edges
                                        .entry(pending_related_node.edge_labels.0)
                                        .and_modify(|existing| {
                                            existing.push(pending_related_node.id.clone())
                                        })
                                        .or_insert(vec![])
                                        .push(pending_related_node.id.clone());
                                }
                                None => {}
                            }
                        }
                        Err(_err) => {}
                    }
                    // Add a connection edge from the new node to the parent node
                    match self.nodes.read() {
                        Ok(nodes) => match nodes.get(&pending_related_node.id.clone()) {
                            Some(node) => {
                                node.write()
                                    .unwrap()
                                    .edges
                                    .entry(pending_related_node.edge_labels.1)
                                    .and_modify(|existing| {
                                        existing.push(pending_related_node.parent_node_id.clone())
                                    })
                                    .or_insert(vec![])
                                    .push(pending_related_node.parent_node_id);
                            }
                            None => {}
                        },
                        Err(_err) => {}
                    }
                    count += 1;
                }
            }
        }

        if count > 0 {
            info!("Added {} nodes", count);
        }
        count > 0
    }

    pub fn add_node(&self, payload: Payload) -> NodeId {
        if let Some(existing_node_id) = self.find_existing(&payload) {
            // If there is the same payload saved in the graph, we do not add a new node
            return existing_node_id;
        }
        if let Some(existing_node_id) = self.find_pending(&payload) {
            // If there is a pending node with the same payload, we do not add a new node
            return existing_node_id;
        }
        let id = Arc::new({
            let mut id = self.last_node_id.lock().unwrap();
            *id += 1;
            *id
        });
        self.pending_nodes
            .write()
            .unwrap()
            .push(PendingNode::Root(PendingRootNode {
                id: id.clone(),
                payload,
            }));
        id
    }

    pub fn add_connection(
        &self,
        parent_id: &NodeId,
        payload: Payload,
        edge_labels: (EdgeLabel, EdgeLabel),
    ) -> NodeId {
        if let Some(existing_node_id) = self.find_existing(&payload) {
            // If there is the same payload saved in the graph, we do not add a new node
            return existing_node_id;
        }
        if let Some(existing_node_id) = self.find_pending(&payload) {
            // If there is a pending node with the same payload, we do not add a new node
            return existing_node_id;
        }
        let id = Arc::new({
            let mut id = self.last_node_id.lock().unwrap();
            *id += 1;
            *id
        });
        self.pending_nodes
            .write()
            .unwrap()
            .push(PendingNode::Related(PendingRelatedNode {
                id: id.clone(),
                payload,
                parent_node_id: parent_id.clone(),
                edge_labels,
            }));
        id
    }

    fn find_existing(&self, payload: &Payload) -> Option<NodeId> {
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

    fn find_pending(&self, _payload: &Payload) -> Option<Arc<u32>> {
        None
    }

    fn save_to_disk(&self, storage_path: &String) {
        // We use RocksDB to store the graph
        let db = DB::open_default(storage_path).unwrap();
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

    fn load_from_disk(&mut self, storage_path: &String) {
        let db = DB::open_default(storage_path).unwrap();
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
        // TODO: Move this function to the Domain node
        let (domain, domain_node_id): (Domain, NodeId) = {
            let connected = self.get_node_ids_connected_with_label(
                node_id.clone(),
                CommonEdgeLabels::Related.to_string(),
            );
            let found: Option<(Domain, NodeId)> = match self.nodes.read() {
                Ok(nodes) => connected
                    .iter()
                    .find_map(|node_id| match nodes.get(node_id) {
                        Some(node) => match node.read().unwrap().payload {
                            Payload::Domain(ref domain) => Some((domain.clone(), node_id.clone())),
                            _ => None,
                        },
                        None => None,
                    }),
                Err(_err) => None,
            };
            match found {
                Some(found) => found,
                None => {
                    error!("Can not find domain for link: {}", &link.url);
                    return false;
                }
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

    pub fn needs_to_tick(&self) -> bool {
        if self.last_tick.nodes_added > 0 || self.last_tick.nodes_updated > 0 {
            return true;
        }
        match self.pending_nodes.read(){
            Ok(nodes) => {
                if nodes.len() > 0 {
                    return true;
                }
            }
            Err(_err) => {
                error!("Error reading pending nodes");
            }
        }
        false
    }

    pub fn get_node_ids_connected_with_label(
        &self,
        starting_node_id: NodeId,
        label: NodeLabel,
    ) -> Vec<NodeId> {
        match self.nodes.read() {
            Ok(nodes) => match nodes.get(&starting_node_id) {
                Some(node) => match node.read().unwrap().edges.get(&label) {
                    Some(node_ids) => node_ids.clone(),
                    None => vec![],
                },
                None => vec![],
            },
            Err(_err) => vec![],
        }
    }

    pub fn get_first_node_id_connected_with_label(
        &self,
        starting_node_id: NodeId,
        label: NodeLabel,
    ) -> Option<NodeId> {
        match self.nodes.read() {
            Ok(nodes) => match nodes.get(&starting_node_id) {
                Some(node) => match node.read().unwrap().edges.get(&label) {
                    Some(node_ids) => node_ids.first().cloned(),
                    None => None,
                },
                None => None,
            },
            Err(_err) => None,
        }
    }
}

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}
