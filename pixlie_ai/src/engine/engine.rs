use super::{
    ArcedEdgeLabel, ArcedNodeId, ArcedNodeItem, ArcedNodeLabel, EdgeLabel, ExistingOrNewNodeId,
    Node, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::api::handle_engine_api_request;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::{PiChannel, PiEvent};
use chrono::Utc;
use log::{debug, error, info};
use postcard::{from_bytes, to_allocvec};
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
    node_ids: (NodeId, NodeId),          // First and second node IDs
    edge_labels: (EdgeLabel, EdgeLabel), // From first to second and back
}

pub(crate) struct Labels {
    data: HashSet<ArcedNodeLabel>,
}

pub(crate) struct Nodes {
    data: HashMap<ArcedNodeId, ArcedNodeItem>,
}

impl Labels {
    fn new() -> Labels {
        Labels {
            data: HashSet::new(),
        }
    }
}

impl Iterator for Labels {
    type Item = ArcedNodeLabel;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.iter().next().map(|x| x.clone())
    }
}

impl Nodes {
    fn new() -> Nodes {
        Nodes {
            data: HashMap::new(),
        }
    }
}

impl Iterator for Nodes {
    type Item = (ArcedNodeId, ArcedNodeItem);

    fn next(&mut self) -> Option<Self::Item> {
        self.data.iter().next().map(|(k, v)| (k.clone(), v.clone()))
    }
}

pub(crate) struct Edges {
    data: HashMap<ArcedNodeId, Vec<(ArcedNodeId, ArcedEdgeLabel)>>,
}

impl Edges {
    fn new() -> Edges {
        Edges {
            data: HashMap::new(),
        }
    }
}

impl Iterator for Edges {
    type Item = (ArcedNodeId, Vec<(ArcedNodeId, ArcedEdgeLabel)>);

    fn next(&mut self) -> Option<Self::Item> {
        self.data.iter().next().map(|(k, v)| (k.clone(), v.clone()))
    }
}

pub(crate) struct NodeIdsByLabel {
    data: HashMap<ArcedNodeLabel, Vec<ArcedNodeId>>,
}

impl NodeIdsByLabel {
    fn new() -> NodeIdsByLabel {
        NodeIdsByLabel {
            data: HashMap::new(),
        }
    }
}

impl Iterator for NodeIdsByLabel {
    type Item = (ArcedNodeLabel, Vec<ArcedNodeId>);

    fn next(&mut self) -> Option<Self::Item> {
        self.data.iter().next().map(|(k, v)| (k.clone(), v.clone()))
    }
}

// The engine keeps track of all the data nodes and their relationships
// All the entity labels are loaded in the engine
// All data may not be loaded in the engine, some of them may be on disk
pub struct Engine {
    // pub labels: RwLock<HashSet<NodeLabel>>,
    // pub nodes: RwLock<HashMap<NodeId, RwLock<NodeItem>>>, // All nodes that are in the engine
    // pub node_ids_by_label: RwLock<HashMap<NodeLabel, Vec<NodeId>>>,
    labels: Mutex<Labels>,
    nodes: Mutex<Nodes>, // All nodes that are in the engine
    edges: Mutex<Edges>,
    node_ids_by_label: Mutex<NodeIdsByLabel>,

    // pending_nodes_to_add: RwLock<Vec<PendingNode>>, // Nodes pending to be added
    // pending_edges_to_add: RwLock<Vec<PendingEdge>>, // Edges pending to be added
    // pending_nodes_to_update: RwLock<Vec<PendingNode>>, // Nodes pending to be updated
    last_node_id: Mutex<u32>,
    project_id: String,
    project_path_on_disk: PathBuf,
    last_ticked_at: RwLock<Instant>,

    my_pi_channel: PiChannel, // Used to communicate with the main thread
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
    fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
}

impl Engine {
    fn new(
        project_id: String,
        storage_root: PathBuf,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> Engine {
        let engine = Engine {
            // labels: RwLock::new(HashSet::new()),
            // nodes: RwLock::new(HashMap::new()),
            // node_ids_by_label: RwLock::new(HashMap::new()),
            labels: Mutex::new(Labels::new()),
            nodes: Mutex::new(Nodes::new()),
            edges: Mutex::new(Edges::new()),
            node_ids_by_label: Mutex::new(NodeIdsByLabel::new()),

            // pending_nodes_to_add: RwLock::new(vec![]),
            // pending_edges_to_add: RwLock::new(vec![]),
            // pending_nodes_to_update: RwLock::new(vec![]),
            last_node_id: Mutex::new(0),
            project_id,
            project_path_on_disk: storage_root,
            last_ticked_at: RwLock::new(Instant::now()),

            my_pi_channel,
            main_channel_tx,
            fetcher_tx,
        };
        engine
    }

    pub fn open_project(
        storage_root: &String,
        project_id: &String,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> Engine {
        let mut storage_path = PathBuf::from(storage_root);
        storage_path.push(format!("{}.rocksdb", project_id));
        let mut engine = Engine::new(
            project_id.clone(),
            storage_path.clone(),
            my_pi_channel,
            main_channel_tx,
            fetcher_tx,
        );
        engine.load_from_disk();
        engine
    }

    fn tick(&self) {
        // let added_nodes = self.add_pending_nodes();
        // let added_edges = self.add_pending_edges();
        // if added_nodes || added_edges {
        //     self.save_to_disk();
        // }

        self.process_nodes();
        // let updated = self.update_pending_nodes();
        // if updated {
        self.save_to_disk();
        // }

        // if added_nodes || added_edges || updated {
        // We have created or updated some nodes, we need to tick again
        self.tick_me_later();
        // }
    }

    fn tick_me_later(&self) {
        match self
            .main_channel_tx
            .send(PiEvent::TickMeLater(self.project_id.clone()))
        {
            Ok(_) => {}
            Err(err) => {
                error!("Error sending PiEvent::NeedsToTick in Engine: {}", err);
            }
        }
    }

    fn exit(&self) {
        debug!("Exiting engine for project {}", self.project_id);
        // We tell the main thread that we are done ticking
        match self
            .main_channel_tx
            .send(PiEvent::EngineExit(self.project_id.clone()))
        {
            Ok(_) => {}
            Err(err) => {
                error!("Error sending PiEvent::EngineRan in Engine: {}", err);
            }
        }
    }

    pub fn run(&self) {
        // We block on the channel of this engine
        for event in self.my_pi_channel.rx.iter() {
            match event {
                PiEvent::APIRequest(project_id, request) => {
                    if self.project_id == project_id {
                        debug!("API request {} for engine", project_id);
                        match handle_engine_api_request(request, self, self.main_channel_tx.clone())
                        {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error handling API request: {}", err);
                            }
                        }
                    }
                }
                PiEvent::NeedsToTick => {
                    let has_ticked = false;
                    match self.last_ticked_at.read() {
                        Ok(last_tick_at) => {
                            if last_tick_at.elapsed().as_millis() > 10 {
                                self.tick();
                            } else {
                                self.tick_me_later();
                            }
                        }
                        Err(_err) => {
                            error!("Error reading last_ticked_at in Engine");
                        }
                    }
                    if has_ticked {
                        match self.last_ticked_at.write() {
                            Ok(mut last_tick_at) => {
                                *last_tick_at = Instant::now();
                            }
                            Err(_err) => {
                                error!("Error writing last_ticked_at in Engine");
                            }
                        };
                    }
                }
                PiEvent::FetchResponse(_project_id, node_id, _url, contents) => {
                    // Call the node that had needs this data
                    match self.get_node_by_id(&node_id) {
                        Some(node) => match node.payload {
                            Payload::Link(ref payload) => {
                                let engine = Arc::new(self);
                                let node_id = Arc::new(node_id.clone());
                                match payload.process(engine, &node_id, Some(contents)) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("Error processing link: {}", err);
                                    }
                                }
                            }
                            _ => {}
                        },
                        None => {}
                    }
                }
                _ => {}
            }
        }
        self.exit();
    }

    pub fn iter_nodes(&self) -> Option<(ArcedNodeId, ArcedNodeItem)> {
        match self.nodes.try_lock() {
            Ok(mut nodes) => nodes.next(),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                None
            }
        }
    }

    pub fn get_node_by_id(&self, node_id: &NodeId) -> Option<ArcedNodeItem> {
        match self.nodes.try_lock() {
            Ok(nodes) => nodes.data.get(node_id).map(|x| x.clone()),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                None
            }
        }
    }

    fn process_nodes(&self) {
        let engine = Arc::new(self);
        for (node_id, node) in self.iter_nodes() {
            match node.payload {
                Payload::Link(ref payload) => {
                    match payload.process(engine.clone(), &node_id, None) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error processing link: {}", err);
                        }
                    }
                }
                Payload::FileHTML(ref payload) => {
                    match payload.process(engine.clone(), &node_id, None) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error processing WebPage: {}", err);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn save_node(
        &self,
        id: NodeId,
        payload: Payload,
        given_labels: Vec<NodeLabel>,
        // edges: HashMap<EdgeLabel, Vec<NodeId>>,
    ) -> PiResult<()> {
        // Get new ID after incrementing existing node ID
        let primary_label = payload.to_string();
        // Create a new vector of labels with the primary label and given labels, each Arced
        let mut all_given_labels: Vec<ArcedNodeLabel> = given_labels
            .iter()
            .map(|x| Arc::new(x.clone()))
            .collect::<Vec<ArcedNodeLabel>>();
        all_given_labels.push(Arc::new(primary_label.clone()));
        let arced_id = Arc::new(id);

        // Store all labels in the engine
        match self.labels.try_lock() {
            Ok(mut labels) => {
                for label in all_given_labels.iter() {
                    labels.data.insert(label.clone());
                }
            }
            Err(err) => {
                error!("Error locking labels: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking labels: {}",
                    err
                )));
            }
        };

        // Store the node in the engine
        {
            match self.nodes.try_lock() {
                Ok(mut nodes) => {
                    nodes.data.insert(
                        arced_id.clone(),
                        Arc::new(NodeItem {
                            id,
                            payload,

                            labels: given_labels.clone(),
                            // edges: HashMap::new(),
                            written_at: Utc::now(),
                        }),
                    );
                }
                Err(err) => {
                    error!("Error locking nodes: {}", err);
                    return Err(PiError::InternalError(format!(
                        "Error locking nodes: {}",
                        err
                    )));
                }
            }
        }
        // Store the node in nodes_by_label_id for the label from Payload and given labels
        {
            match self.node_ids_by_label.try_lock() {
                Ok(mut node_ids_by_label) => {
                    for label in all_given_labels.iter() {
                        node_ids_by_label
                            .data
                            .entry(label.clone())
                            .and_modify(|entries| entries.push(arced_id.clone()))
                            .or_insert(vec![arced_id.clone()]);
                    }
                }
                Err(err) => {
                    error!("Error locking node_ids_by_label: {}", err);
                    return Err(PiError::InternalError(format!(
                        "Error locking node_ids_by_label: {}",
                        err
                    )));
                }
            };
        }
        Ok(())
    }

    pub fn get_or_add_node(
        &self,
        payload: Payload,
        labels: Vec<NodeLabel>,
        should_add_new: bool,
    ) -> PiResult<ExistingOrNewNodeId> {
        if let Some(existing_node_id) = self.find_existing(&payload) {
            // If there is the same payload saved in the graph, we do not add a new node
            return Ok(ExistingOrNewNodeId::Existing(existing_node_id));
        }
        // if let Some(existing_node_id) = self.find_pending(&payload) {
        //     // If there is a pending node with the same payload, we do not add a new node
        //     return Ok(ExistingOrNewNodeId::Pending(existing_node_id));
        // }
        if !should_add_new {
            error!("Could not find existing node and should not add new node");
            return Err(PiError::InternalError(
                "Could not find existing node and should not add new node".to_string(),
            ));
        }

        let id = {
            match self.last_node_id.lock() {
                Ok(mut id) => {
                    *id += 1;
                    *id
                }
                Err(err) => {
                    error!("Error locking last_node_id: {}", err);
                    return Err(PiError::InternalError(format!(
                        "Error locking last_node_id: {}",
                        err
                    )));
                }
            }
        };
        // match self.pending_nodes_to_add.write() {
        //     Ok(mut pending_nodes) => {
        //         pending_nodes.push(PendingNode {
        //             id: id.clone(),
        //             payload,
        //             labels,
        //         });
        //         self.tick_me_later();
        //     }
        //     Err(err) => {
        //         error!("Could not write to pending_nodes_to_add in Engine: {}", err);
        //         return Err(PiError::InternalError(format!(
        //             "Could not write to pending_nodes_to_add in Engine: {}",
        //             err
        //         )));
        //     }
        // }
        match self.save_node(id, payload, labels) {
            Ok(_) => {}
            Err(err) => {
                error!("Error saving node: {}", err);
                return Err(err);
            }
        };
        Ok(ExistingOrNewNodeId::New(id))
    }

    pub fn add_connection(
        &self,
        node_ids: (NodeId, NodeId),
        edge_labels: (EdgeLabel, EdgeLabel),
    ) -> PiResult<()> {
        let arced_node_ids = (Arc::new(node_ids.0), Arc::new(node_ids.1));
        let arced_edge_labels = (Arc::new(edge_labels.0), Arc::new(edge_labels.1));
        // Add a connection edge from the parent node to the new node and vice versa
        match self.edges.try_lock() {
            Ok(mut edges) => {
                edges.data.entry(arced_node_ids.0.clone()).or_insert(vec![]);
                edges.data.entry(arced_node_ids.1.clone()).or_insert(vec![]);
                edges
                    .data
                    .get_mut(&arced_node_ids.0)
                    .unwrap()
                    .push((arced_node_ids.1.clone(), arced_edge_labels.0.clone()));
                debug!(
                    "Added {} edge from node {} to node {}",
                    arced_edge_labels.0, node_ids.0, node_ids.1
                );
                edges
                    .data
                    .get_mut(&arced_node_ids.1)
                    .unwrap()
                    .push((arced_node_ids.0.clone(), arced_edge_labels.1.clone()));
                debug!(
                    "Added {} edge from node {} to node {}",
                    arced_edge_labels.1, node_ids.1, node_ids.0
                );
            }
            Err(err) => {
                error!("Error locking edges: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking edges: {}",
                    err
                )));
            }
        };
        Ok(())
    }

    pub fn update_node(&self, node_id: &NodeId, payload: Payload) -> PiResult<()> {
        // match self.pending_nodes_to_update.write() {
        //     Ok(mut pending_nodes) => {
        //         pending_nodes.push(PendingNode {
        //             id: node_id.clone(),
        //             payload,
        //             labels: vec![],
        //         });
        //         self.tick_me_later();
        //     }
        //     Err(err) => {
        //         error!("Error writing PendingNode in Engine: {}", err);
        //     }
        // }

        match self.nodes.try_lock() {
            Ok(mut nodes) => match nodes.data.get_mut(node_id) {
                Some(node) => {
                    *node = Arc::new(NodeItem {
                        id: node_id.clone(),
                        payload,
                        labels: vec![],
                        written_at: Utc::now(),
                    });
                }
                None => {}
            },
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )));
            }
        };
        Ok(())
    }

    fn find_existing(&self, payload: &Payload) -> Option<NodeId> {
        // For certain node payloads, check if there is a node with the same payload
        let engine = Arc::new(self);
        match payload {
            Payload::Domain(ref domain) => {
                let existing =
                    match Domain::find_existing(engine, FindDomainOf::DomainName(&domain.name)) {
                        Ok(domain) => domain,
                        Err(_err) => {
                            return None;
                        }
                    };
                match existing {
                    Some((_existing_node, existing_node_id)) => Some(existing_node_id),
                    None => None,
                }
            }
            Payload::Link(ref link) => {
                let existing = match Link::find_existing(engine, &link.get_full_link()) {
                    Ok(link) => link,
                    Err(_err) => {
                        return None;
                    }
                };
                match existing {
                    Some((_existing_node, existing_node_id)) => Some(existing_node_id),
                    None => None,
                }
            }
            _ => None,
        }
    }

    fn save_to_disk(&self) {
        // We use RocksDB to store the graph
        let db = DB::open_default(self.project_path_on_disk.as_os_str()).unwrap();
        match self.nodes.lock() {
            Ok(nodes) => {
                for (node_id, node) in nodes.data.iter() {
                    let bytes = match to_allocvec(&*node) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            error!("Error serializing node: {}", err);
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
            Err(err) => {
                error!("Error locking nodes: {}", err);
            }
        }
    }

    fn load_from_disk(&mut self) {
        let db = DB::open_default(self.project_path_on_disk.as_os_str()).unwrap();
        let iter = db.iterator(rocksdb::IteratorMode::Start);
        let mut last_node_id: u32 = 0;
        for item in iter {
            let (key, value) = match item {
                Ok(item) => item,
                Err(err) => {
                    error!("Error iterating over RocksDB: {}", err);
                    break;
                }
            };
            let node_id = Arc::new(read_le_u32(&mut &*key));
            let node: NodeItem = match from_bytes(&value) {
                Ok(node) => node,
                Err(err) => {
                    error!("Error deserializing node: {}", err);
                    break;
                }
            };
            // The first label if the type of the payload
            let mut labels: Vec<NodeLabel> = vec![node.payload.to_string().clone()];
            labels.extend(node.labels.iter().cloned());
            let labels: Vec<ArcedNodeLabel> = labels.iter().map(|x| Arc::new(x.clone())).collect();
            match self.nodes.lock() {
                Ok(mut nodes) => {
                    nodes.data.insert(node_id.clone(), Arc::new(node));
                }
                Err(err) => {
                    error!("Error locking nodes: {}", err);
                    self.exit();
                }
            }

            // Store the node in nodes_by_label_id
            match self.node_ids_by_label.lock() {
                Ok(mut node_ids_by_label) => {
                    for label in labels.into_iter() {
                        node_ids_by_label
                            .data
                            .entry(label)
                            .and_modify(|entries| entries.push(node_id.clone()))
                            .or_insert(vec![node_id.clone()]);
                    }
                }
                Err(err) => {
                    error!("Error locking node_ids_by_label: {}", err);
                    self.exit();
                }
            }

            last_node_id = *node_id;
        }
        match self.last_node_id.lock() {
            Ok(mut inner) => {
                *inner = last_node_id;
                self.tick_me_later();
            }
            Err(err) => {
                error!("Error locking last_node_id: {}", err);
                self.exit();
            }
        }
    }

    pub fn get_node_ids_connected_with_label(
        &self,
        starting_node_id: &NodeId,
        label: &NodeLabel,
    ) -> PiResult<Vec<NodeId>> {
        match self.nodes.read() {
            Ok(nodes) => match nodes.get(starting_node_id) {
                Some(node) => match node.read() {
                    Ok(node) => match node.edges.get(label) {
                        Some(node_ids) => Ok(node_ids.clone()),
                        None => Err(PiError::GraphError(format!(
                            "Node {} does not have any edges with label {}",
                            starting_node_id, label
                        ))),
                    },
                    Err(err) => Err(PiError::InternalError(format!(
                        "Could not read node {} from the engine: {}",
                        starting_node_id, err
                    ))),
                },
                None => Err(PiError::GraphError(format!(
                    "Node {} does not exist",
                    starting_node_id
                ))),
            },
            Err(err) => Err(PiError::InternalError(format!(
                "Could not read nodes from the engine: {}",
                err
            ))),
        }
    }

    pub fn get_first_node_id_connected_with_label(
        &self,
        starting_node_id: &NodeId,
        label: &NodeLabel,
    ) -> PiResult<NodeId> {
        match self.get_node_ids_connected_with_label(starting_node_id, label) {
            Ok(node_ids) => match node_ids.first() {
                Some(node_id) => Ok(node_id.clone()),
                None => Err(PiError::GraphError(format!(
                    "Node {} does not have any edges with label {}",
                    starting_node_id, label
                ))),
            },
            Err(err) => Err(err),
        }
    }

    pub fn fetch_url(&self, url: &str, node_id: &NodeId) -> PiResult<()> {
        let engine = Arc::new(self);
        let (domain, domain_node_id) =
            Domain::can_fetch_within_domain(engine.clone(), url, node_id)?;

        debug!("Domain {} is allowed to crawl", &domain.name);
        // Update the domain at the domain node id
        match engine.nodes.read() {
            Ok(nodes) => match nodes.get(&domain_node_id) {
                Some(node) => match node.write() {
                    Ok(mut node) => {
                        node.payload = Payload::Domain(Domain {
                            name: domain.name.clone(),
                            is_allowed_to_crawl: domain.is_allowed_to_crawl,
                        });
                        debug!("Set domain.last_fetched_at to now for {}", &domain.name);
                    }
                    Err(_err) => {
                        return Err(PiError::GraphError(
                            "Error writing to domain node".to_string(),
                        ));
                    }
                },
                None => {
                    return Err(PiError::GraphError(
                        "Cannot find domain node for link".to_string(),
                    ));
                }
            },
            Err(err) => {
                return Err(PiError::GraphError(format!(
                    "Error reading domain node: {}",
                    err
                )));
            }
        }

        let full_url = format!("https://{}{}", domain.name, url);
        match self.fetcher_tx.blocking_send(PiEvent::FetchRequest(
            self.project_id.clone(),
            node_id,
            full_url.clone(),
        )) {
            Ok(_) => {
                debug!("Sent fetch request for url {}", &full_url);
            }
            Err(err) => {
                error!("Error sending request to fetcher: {}", err);
                return Err(PiError::FetchError(format!(
                    "Error sending request to fetcher: {}",
                    err
                )));
            }
        }
        Ok(())
    }
}

// fn add_pending_nodes(&self) -> bool {
//     let mut count_nodes = 0;
//     let mut nodes_to_write: Vec<PendingNode> = self
//         .pending_nodes_to_add
//         .write()
//         .unwrap()
//         .drain(..)
//         .collect();
//     while let Some(pending_node) = nodes_to_write.pop() {
//         self.save_node(
//             pending_node.id,
//             pending_node.payload,
//             pending_node.labels,
//             HashMap::new(),
//         );
//         count_nodes += 1;
//     }
//
//     if count_nodes > 0 {
//         info!("Added {} pending nodes to the graph", count_nodes);
//     }
//     count_nodes > 0
// }

// fn add_pending_edges(&self) -> bool {
//     let mut count_edges = 0;
//     let mut edges_to_write: Vec<PendingEdge> = self
//         .pending_edges_to_add
//         .write()
//         .unwrap()
//         .drain(..)
//         .collect();
//     match self.nodes.read() {
//         Ok(nodes) => {
//             while let Some(pending_edge) = edges_to_write.pop() {
//                 // Add a connection edge from the parent node to the new node
//                 match nodes.get(&pending_edge.node_ids.0.clone()) {
//                     Some(node) => match node.write() {
//                         Ok(mut node) => {
//                             node.edges
//                                 .entry(pending_edge.edge_labels.0.clone())
//                                 .and_modify(|existing| {
//                                     existing.push(pending_edge.node_ids.1.clone())
//                                 })
//                                 .or_insert(vec![pending_edge.node_ids.1.clone()]);
//                             count_edges += 1;
//                             debug!(
//                                 "Added {} edge from node {} to node {}",
//                                 pending_edge.edge_labels.0,
//                                 pending_edge.node_ids.0,
//                                 pending_edge.node_ids.1
//                             );
//                         }
//                         Err(e) => {
//                             error!(
//                                 "Failed to add {} edge from node {} to node {}: {}",
//                                 pending_edge.edge_labels.0,
//                                 pending_edge.node_ids.0,
//                                 pending_edge.node_ids.1,
//                                 e
//                             );
//                         }
//                     },
//                     None => {
//                         error!(
//                             "Failed to add {} edge from node {} to node {}",
//                             pending_edge.edge_labels.0,
//                             pending_edge.node_ids.0,
//                             pending_edge.node_ids.1,
//                         );
//                     }
//                 };
//                 // Add a connection edge from the new node to the parent node
//                 match nodes.get(&pending_edge.node_ids.1.clone()) {
//                     Some(node) => match node.write() {
//                         Ok(mut node) => {
//                             node.edges
//                                 .entry(pending_edge.edge_labels.1.clone())
//                                 .and_modify(|existing| {
//                                     existing.push(pending_edge.node_ids.0.clone())
//                                 })
//                                 .or_insert(vec![pending_edge.node_ids.0.clone()]);
//                             count_edges += 1;
//                             debug!(
//                                 "Added {} edge from node {} to node {}",
//                                 pending_edge.edge_labels.1,
//                                 pending_edge.node_ids.1,
//                                 pending_edge.node_ids.0
//                             );
//                         }
//                         Err(e) => {
//                             error!(
//                                 "Failed to add {} edge from node {} to node {}: {}",
//                                 pending_edge.edge_labels.1,
//                                 pending_edge.node_ids.1,
//                                 pending_edge.node_ids.0,
//                                 e
//                             );
//                         }
//                     },
//                     None => {
//                         error!(
//                             "Failed to add {} edge from node {} to node {}",
//                             pending_edge.edge_labels.1,
//                             pending_edge.node_ids.1,
//                             pending_edge.node_ids.0,
//                         );
//                     }
//                 }
//             }
//         }
//         Err(_err) => {}
//     }
//
//     if count_edges > 0 {
//         info!("Added {} pending edges to the graph", count_edges);
//     }
//     count_edges > 0
// }

// fn update_pending_nodes(&self) -> bool {
//     let mut count_nodes = 0;
//     let mut pending_nodes_to_update: Vec<PendingNode> =
//         match self.pending_nodes_to_update.write() {
//             Ok(mut pending_nodes_to_update) => pending_nodes_to_update.drain(..).collect(),
//             Err(_err) => vec![],
//         };
//     match self.nodes.read() {
//         Ok(nodes) => {
//             while let Some(pending_node) = pending_nodes_to_update.pop() {
//                 match nodes.get(&pending_node.id) {
//                     Some(node) => match node.write() {
//                         Ok(mut node) => {
//                             node.payload = pending_node.payload;
//                         }
//                         Err(_err) => {}
//                     },
//                     None => {}
//                 }
//                 count_nodes += 1;
//             }
//         }
//         Err(_err) => {}
//     }
//     if count_nodes > 0 {
//         info!("Updated {} pending nodes to the graph", count_nodes);
//         self.tick_me_later();
//     }
//     count_nodes > 0
// }

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}
