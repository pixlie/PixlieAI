use super::{CommonEdgeLabels, EdgeLabel, Node, NodeId, NodeLabel, NodeWorker, Payload};
use crate::entity::web::{Domain, Link};
use chrono::Utc;
use log::{debug, error, info};
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

struct PendingNode {
    id: NodeId,
    payload: Payload,
    labels: Vec<NodeLabel>,
}

struct PendingEdge {
    node_ids: (NodeId, NodeId),          // Parent and child node IDs
    edge_labels: (EdgeLabel, EdgeLabel), // From parent to child and back
}

// The engine keeps track of all the data nodes and their relationships
// All the entity labels are loaded in the engine
// All data may not be loaded in the engine, some of them may be on disk
pub struct Engine {
    pub labels: RwLock<HashSet<NodeLabel>>,
    pub nodes: RwLock<HashMap<NodeId, RwLock<Node>>>, // All nodes that are in the engine
    pending_nodes: RwLock<Vec<PendingNode>>,          // Nodes pending to be written
    pending_edges: RwLock<Vec<PendingEdge>>,          // Edges pending to be written
    last_node_id: Mutex<u32>,
    pub node_ids_by_label: RwLock<HashMap<NodeLabel, Vec<NodeId>>>,
    project_path_on_disk: PathBuf,
}

pub type LockedEngine = RwLock<Engine>;

impl Engine {
    fn new(storage_root: PathBuf) -> Engine {
        let engine = Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: RwLock::new(HashMap::new()),
            pending_nodes: RwLock::new(vec![]),
            pending_edges: RwLock::new(vec![]),
            last_node_id: Mutex::new(0),
            node_ids_by_label: RwLock::new(HashMap::new()),
            project_path_on_disk: storage_root,
        };
        engine
    }

    pub fn open_project(storage_root: &String, project_id: &String) -> Engine {
        let mut storage_path = PathBuf::from(storage_root);
        storage_path.push(format!("{}.rocksdb", project_id));
        let mut engine = Engine::new(storage_path.clone());
        engine.load_from_disk();
        engine
    }

    pub fn tick(&self) -> bool {
        let added_nodes = self.add_pending_nodes();
        let added_edges = self.add_pending_edges();
        let updated = self.process_nodes();
        if added_nodes || added_edges || updated {
            self.save_to_disk();
        }
        added_nodes || added_edges || updated
    }

    fn process_nodes(&self) -> bool {
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

    fn save_node(
        &self,
        id: NodeId,
        payload: Payload,
        labels: Vec<NodeLabel>,
        edges: HashMap<EdgeLabel, Vec<NodeId>>,
    ) {
        // Get new ID after incrementing existing node ID
        let label = payload.to_string();
        // Store the label from the type of Payload in the engine
        {
            match self.labels.write() {
                Ok(mut labels) => {
                    labels.insert(label.clone());
                }
                Err(_err) => {}
            }
        };
        // Store the other given labels in the engine
        for label in labels.iter() {
            // Store the label in the engine
            {
                match self.labels.write() {
                    Ok(mut labels) => {
                        labels.insert(label.clone());
                    }
                    Err(_err) => {}
                }
            };
        }
        // Store the node in the engine
        {
            match self.nodes.write() {
                Ok(mut nodes) => {
                    nodes.insert(
                        id.clone(),
                        RwLock::new(Node {
                            id: id.clone(),
                            payload,

                            labels: labels.clone(),
                            edges,
                            written_at: Utc::now(),
                        }),
                    );
                }
                Err(_err) => {}
            }
        }
        // Store the node in nodes_by_label_id for the label from Payload and given labels
        {
            let mut nodes_by_label = self.node_ids_by_label.write().unwrap();
            nodes_by_label
                .entry(label.clone())
                .and_modify(|entries| entries.push(id.clone()))
                .or_insert(vec![id.clone()]);
            for label in labels.iter() {
                nodes_by_label
                    .entry(label.clone())
                    .and_modify(|entries| entries.push(id.clone()))
                    .or_insert(vec![id.clone()]);
            }
        };
    }

    fn add_pending_nodes(&self) -> bool {
        let mut count_nodes = 0;
        let mut nodes_to_write: Vec<PendingNode> =
            self.pending_nodes.write().unwrap().drain(..).collect();
        while let Some(pending_node) = nodes_to_write.pop() {
            self.save_node(
                pending_node.id,
                pending_node.payload,
                pending_node.labels,
                HashMap::new(),
            );
            count_nodes += 1;
        }

        if count_nodes > 0 {
            info!("Added {} nodes", count_nodes);
        }
        count_nodes > 0
    }

    fn add_pending_edges(&self) -> bool {
        let mut count_edges = 0;
        let mut edges_to_write: Vec<PendingEdge> =
            self.pending_edges.write().unwrap().drain(..).collect();
        match self.nodes.read() {
            Ok(nodes) => {
                while let Some(pending_edge) = edges_to_write.pop() {
                    // Add a connection edge from the parent node to the new node
                    match nodes.get(&pending_edge.node_ids.0.clone()) {
                        Some(node) => match node.write() {
                            Ok(mut node) => {
                                node.edges
                                    .entry(pending_edge.edge_labels.0.clone())
                                    .and_modify(|existing| {
                                        existing.push(pending_edge.node_ids.1.clone())
                                    })
                                    .or_insert(vec![pending_edge.node_ids.1.clone()]);
                                count_edges += 1;
                                debug!(
                                    "Added {} edge from node {} to node {}",
                                    pending_edge.edge_labels.0,
                                    pending_edge.node_ids.0,
                                    pending_edge.node_ids.1
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to add {} edge from node {} to node {}: {}",
                                    pending_edge.edge_labels.0,
                                    pending_edge.node_ids.0,
                                    pending_edge.node_ids.1,
                                    e
                                );
                            }
                        },
                        None => {
                            error!(
                                "Failed to add {} edge from node {} to node {}",
                                pending_edge.edge_labels.0,
                                pending_edge.node_ids.0,
                                pending_edge.node_ids.1,
                            );
                        }
                    };
                    // Add a connection edge from the new node to the parent node
                    match nodes.get(&pending_edge.node_ids.1.clone()) {
                        Some(node) => match node.write() {
                            Ok(mut node) => {
                                node.edges
                                    .entry(pending_edge.edge_labels.1.clone())
                                    .and_modify(|existing| {
                                        existing.push(pending_edge.node_ids.0.clone())
                                    })
                                    .or_insert(vec![pending_edge.node_ids.0.clone()]);
                                count_edges += 1;
                                debug!(
                                    "Added {} edge from node {} to node {}",
                                    pending_edge.edge_labels.1,
                                    pending_edge.node_ids.1,
                                    pending_edge.node_ids.0
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to add {} edge from node {} to node {}: {}",
                                    pending_edge.edge_labels.1,
                                    pending_edge.node_ids.1,
                                    pending_edge.node_ids.0,
                                    e
                                );
                            }
                        },
                        None => {
                            error!(
                                "Failed to add {} edge from node {} to node {}",
                                pending_edge.edge_labels.1,
                                pending_edge.node_ids.1,
                                pending_edge.node_ids.0,
                            );
                        }
                    }
                }
            }
            Err(_err) => {}
        }

        if count_edges > 0 {
            info!("Added {} edges", count_edges);
        }
        count_edges > 0
    }

    pub fn add_node(&self, payload: Payload, labels: Vec<NodeLabel>) -> NodeId {
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
        self.pending_nodes.write().unwrap().push(PendingNode {
            id: id.clone(),
            payload,
            labels,
        });
        id
    }

    pub fn add_connection(&self, node_ids: (NodeId, NodeId), edge_labels: (EdgeLabel, EdgeLabel)) {
        match self.pending_edges.write() {
            Ok(mut pending_edges) => {
                pending_edges.push(PendingEdge {
                    node_ids,
                    edge_labels,
                });
            }
            Err(e) => {
                error!("Failed to add pending edge: {}", e);
            }
        }
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

    fn save_to_disk(&self) {
        // We use RocksDB to store the graph
        let db = DB::open_default(self.project_path_on_disk.as_os_str()).unwrap();
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

    fn load_from_disk(&mut self) {
        let db = DB::open_default(self.project_path_on_disk.as_os_str()).unwrap();
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
        debug!("Checking if we can fetch within domain: {}", link.url);
        let (domain, domain_node_id): (Domain, NodeId) = {
            let connected = self.get_node_ids_connected_with_label(
                node_id.clone(),
                CommonEdgeLabels::Related.to_string(),
            );
            debug!(
                "Found {} connected nodes with edge label {}",
                connected.len(),
                CommonEdgeLabels::Related.to_string()
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
                    error!("Cannot find domain for link: {}", &link.url);
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
        match self.nodes.read() {
            Ok(nodes) => match nodes.get(&domain_node_id) {
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

        debug!("Domain {} is allowed to crawl", domain.name);
        true
    }

    pub fn needs_to_tick(&self) -> bool {
        match self.pending_nodes.read() {
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
