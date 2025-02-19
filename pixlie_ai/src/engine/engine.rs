use super::{EdgeLabel, ExistingOrNewNodeId, Node, NodeId, NodeItem, NodeLabel, Payload};
use crate::engine::api::handle_engine_api_request;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::utils::fetcher::{FetchEvent, Fetcher};
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

// The engine keeps track of all the data nodes and their relationships
// All the entity labels are loaded in the engine
// All data may not be loaded in the engine, some of them may be on disk
pub struct Engine {
    pub labels: RwLock<HashSet<NodeLabel>>,
    pub nodes: RwLock<HashMap<NodeId, RwLock<NodeItem>>>, // All nodes that are in the engine
    pub node_ids_by_label: RwLock<HashMap<NodeLabel, Vec<NodeId>>>,

    pending_nodes_to_add: RwLock<Vec<PendingNode>>, // Nodes pending to be added
    pending_edges_to_add: RwLock<Vec<PendingEdge>>, // Edges pending to be added
    pending_nodes_to_update: RwLock<Vec<PendingNode>>, // Nodes pending to be updated

    last_node_id: Mutex<u32>,
    project_id: String,
    project_path_on_disk: PathBuf,
    last_ticked_at: RwLock<Instant>,

    fetcher: Arc<Fetcher>,    // Used to fetch URLs, managed by the main thread
    my_pi_channel: PiChannel, // Used to communicate with the main thread
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
}

impl Engine {
    fn new(
        project_id: String,
        storage_root: PathBuf,
        fetcher: Arc<Fetcher>,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
    ) -> Engine {
        let engine = Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: RwLock::new(HashMap::new()),
            node_ids_by_label: RwLock::new(HashMap::new()),

            pending_nodes_to_add: RwLock::new(vec![]),
            pending_edges_to_add: RwLock::new(vec![]),
            pending_nodes_to_update: RwLock::new(vec![]),

            last_node_id: Mutex::new(0),
            project_id,
            project_path_on_disk: storage_root,
            last_ticked_at: RwLock::new(Instant::now()),

            fetcher,
            my_pi_channel,
            main_channel_tx,
        };
        engine
    }

    pub fn open_project(
        storage_root: &String,
        project_id: &String,
        fetcher: Arc<Fetcher>,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
    ) -> Engine {
        let mut storage_path = PathBuf::from(storage_root);
        storage_path.push(format!("{}.rocksdb", project_id));
        let mut engine = Engine::new(
            project_id.clone(),
            storage_path.clone(),
            fetcher,
            my_pi_channel,
            main_channel_tx,
        );
        engine.load_from_disk();
        engine
    }

    fn tick(&self) {
        let added_nodes = self.add_pending_nodes();
        let added_edges = self.add_pending_edges();
        if added_nodes || added_edges {
            self.save_to_disk();
        }

        self.process_nodes();
        let updated = self.update_pending_nodes();
        if updated {
            self.save_to_disk();
        }

        if added_nodes || added_edges || updated {
            // We have created or updated some nodes, we need to tick again
            self.tick_me_later();
        }
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
                _ => {}
            }
        }
        self.exit();
    }

    fn process_nodes(&self) {
        let engine = Arc::new(self);
        match self.nodes.read() {
            Ok(nodes) => {
                for (node_id, node) in nodes.iter() {
                    let node = node.read().unwrap();
                    let node_id = node_id.clone();
                    match node.payload {
                        Payload::Link(ref payload) => {
                            match payload.process(engine.clone(), &node_id) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Error processing link: {}", err);
                                }
                            }
                        }
                        Payload::FileHTML(ref payload) => {
                            match payload.process(engine.clone(), &node_id) {
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
            Err(_err) => {
                error!("Error reading nodes");
            }
        }
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
                        RwLock::new(NodeItem {
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
        let mut nodes_to_write: Vec<PendingNode> = self
            .pending_nodes_to_add
            .write()
            .unwrap()
            .drain(..)
            .collect();
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
            info!("Added {} pending nodes to the graph", count_nodes);
        }
        count_nodes > 0
    }

    fn add_pending_edges(&self) -> bool {
        let mut count_edges = 0;
        let mut edges_to_write: Vec<PendingEdge> = self
            .pending_edges_to_add
            .write()
            .unwrap()
            .drain(..)
            .collect();
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
            info!("Added {} pending edges to the graph", count_edges);
        }
        count_edges > 0
    }

    fn update_pending_nodes(&self) -> bool {
        let mut count_nodes = 0;
        let mut pending_nodes_to_update: Vec<PendingNode> =
            match self.pending_nodes_to_update.write() {
                Ok(mut pending_nodes_to_update) => pending_nodes_to_update.drain(..).collect(),
                Err(_err) => vec![],
            };
        match self.nodes.read() {
            Ok(nodes) => {
                while let Some(pending_node) = pending_nodes_to_update.pop() {
                    match nodes.get(&pending_node.id) {
                        Some(node) => match node.write() {
                            Ok(mut node) => {
                                node.payload = pending_node.payload;
                            }
                            Err(_err) => {}
                        },
                        None => {}
                    }
                    count_nodes += 1;
                }
            }
            Err(_err) => {}
        }
        if count_nodes > 0 {
            info!("Updated {} pending nodes to the graph", count_nodes);
            self.tick_me_later();
        }
        count_nodes > 0
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
        if let Some(existing_node_id) = self.find_pending(&payload) {
            // If there is a pending node with the same payload, we do not add a new node
            return Ok(ExistingOrNewNodeId::Pending(existing_node_id));
        }
        if !should_add_new {
            error!("Could not find existing node and should not add new node");
            return Err(PiError::InternalError(
                "Could not find existing node and should not add new node".to_string(),
            ));
        }

        let id = Arc::new({
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
        });
        match self.pending_nodes_to_add.write() {
            Ok(mut pending_nodes) => {
                pending_nodes.push(PendingNode {
                    id: id.clone(),
                    payload,
                    labels,
                });
                self.tick_me_later();
            }
            Err(err) => {
                error!("Could not write to pending_nodes_to_add in Engine: {}", err);
                return Err(PiError::InternalError(format!(
                    "Could not write to pending_nodes_to_add in Engine: {}",
                    err
                )));
            }
        }
        Ok(ExistingOrNewNodeId::New(id))
    }

    pub fn add_connection(&self, node_ids: (NodeId, NodeId), edge_labels: (EdgeLabel, EdgeLabel)) {
        match self.pending_edges_to_add.write() {
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

    pub fn update_node(&self, node_id: &NodeId, payload: Payload) {
        match self.pending_nodes_to_update.write() {
            Ok(mut pending_nodes) => {
                pending_nodes.push(PendingNode {
                    id: node_id.clone(),
                    payload,
                    labels: vec![],
                });
                self.tick_me_later();
            }
            Err(err) => {
                error!("Error writing PendingNode in Engine: {}", err);
            }
        }
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

    fn find_pending(&self, _payload: &Payload) -> Option<Arc<u32>> {
        None
    }

    pub fn get_node_by_id(&self, node_id: &NodeId) -> PiResult<NodeItem> {
        match self.nodes.read() {
            Ok(nodes) => match nodes.get(node_id) {
                Some(node) => match node.read() {
                    Ok(node) => Ok(node.clone()),
                    Err(err) => Err(PiError::InternalError(format!(
                        "Error reading node: {}",
                        err
                    ))),
                },
                None => Err(PiError::GraphError(format!(
                    "Node {} does not exist",
                    node_id
                ))),
            },
            Err(err) => Err(PiError::InternalError(format!(
                "Error reading nodes: {}",
                err
            ))),
        }
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
            {
                match self.nodes.write() {
                    Ok(mut nodes) => {
                        nodes.insert(node_id.clone(), RwLock::new(node));
                    }
                    Err(_err) => {}
                }
            }

            for label in labels.into_iter() {
                // Store the node in nodes_by_label_id
                self.node_ids_by_label
                    .write()
                    .unwrap()
                    .entry(label)
                    .and_modify(|entries| entries.push(node_id.clone()))
                    .or_insert(vec![node_id.clone()]);
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

    pub fn needs_to_tick(&self) -> bool {
        match self.pending_nodes_to_add.read() {
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

    pub fn fetch_url(
        &self,
        url: &str,
        node_id: &NodeId,
    ) -> PiResult<crossbeam_channel::Receiver<FetchEvent>> {
        let engine = Arc::new(self);
        let (domain, domain_node_id) =
            Domain::can_fetch_within_domain(engine.clone(), url, node_id)?;

        // Update the domain at the domain node id
        match engine.nodes.read() {
            Ok(nodes) => match nodes.get(&domain_node_id) {
                Some(node) => match node.write() {
                    Ok(mut node) => {
                        node.payload = Payload::Domain(Domain {
                            name: domain.name.clone(),
                            is_allowed_to_crawl: domain.is_allowed_to_crawl,
                            last_fetched_at: Some(Instant::now()),
                        });
                    }
                    Err(_err) => {
                        return Err(PiError::FetchError(
                            "Error writing to domain node".to_string(),
                        ));
                    }
                },
                None => {
                    return Err(PiError::FetchError(
                        "Cannot find domain node for link".to_string(),
                    ));
                }
            },
            Err(_err) => {
                return Err(PiError::FetchError("Error reading domain node".to_string()));
            }
        }

        let full_url = format!("https://{}{}", domain.name, url);
        self.fetcher.fetch(full_url)
    }
}

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}
