use super::{
    ArcedEdgeLabel, ArcedNodeId, ArcedNodeItem, ArcedNodeLabel, EdgeLabel, ExistingOrNewNodeId,
    Node, NodeFlags, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::api::handle_engine_api_request;
use crate::engine::edges::Edges;
use crate::engine::nodes::Nodes;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::{FetchRequest, PiChannel, PiEvent};
use chrono::Utc;
use log::{debug, error};
use rocksdb::DB;
use std::collections::{HashMap, HashSet};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub(crate) struct Labels {
    data: HashSet<ArcedNodeLabel>,
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

// The engine keeps track of all the data nodes and their relationships
pub struct Engine {
    labels: Mutex<Labels>,
    nodes: Mutex<Nodes>, // All nodes that are in the engine
    edges: Mutex<Edges>,
    node_ids_by_label: Mutex<NodeIdsByLabel>,

    last_node_id: Mutex<u32>,
    project_id: String,
    project_path_on_disk: PathBuf,

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
            labels: Mutex::new(Labels::new()),
            nodes: Mutex::new(Nodes::new()),
            edges: Mutex::new(Edges::new()),
            node_ids_by_label: Mutex::new(NodeIdsByLabel::new()),

            last_node_id: Mutex::new(0),
            project_id,
            project_path_on_disk: storage_root,

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
        engine.load_from_disk().unwrap();
        engine
    }

    fn tick(&self) {
        self.process_nodes();
        match self.save_to_disk() {
            Ok(_) => {}
            Err(err) => {
                error!("Error saving to disk: {}", err);
                self.exit();
                return;
            }
        }
        self.tick_me_later();
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
                    self.tick();
                }
                PiEvent::FetchResponse(response) => {
                    // Call the node that had needs this data
                    match self.get_node_by_id(&response.node_id) {
                        Some(node) => match node.payload {
                            Payload::Link(ref payload) => {
                                let engine = Arc::new(self);
                                let node_id = Arc::new(response.node_id.clone());
                                match payload.process(engine, &node_id, Some(response.contents)) {
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
        let node_ids: Vec<ArcedNodeId> = match self.nodes.try_lock() {
            Ok(nodes) => nodes.data.keys().map(|x| x.clone()).collect(),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return;
            }
        };
        for node_id in node_ids {
            if let Some(node) = self.get_node_by_id(&node_id) {
                if node.flags.contains(NodeFlags::IS_PROCESSED) {
                    continue;
                }
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
    }

    fn save_node(
        &self,
        id: NodeId,
        payload: Payload,
        given_labels: Vec<NodeLabel>,
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
                            flags: NodeFlags::default(),
                            written_at: Utc::now(),
                        }),
                    );
                    nodes.save_item_chunk_to_disk(&self.get_db()?, &id)?;
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
        self.tick_me_later();
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
            return Ok(ExistingOrNewNodeId::Existing(*existing_node_id));
        }

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
                edges.save_item_chunk_to_disk(&self.get_db()?, &node_ids.0)?;
                edges.save_item_chunk_to_disk(&self.get_db()?, &node_ids.1)?;
            }
            Err(err) => {
                error!("Error locking edges: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking edges: {}",
                    err
                )));
            }
        };
        self.tick_me_later();
        Ok(())
    }

    pub fn update_node(&self, node_id: &NodeId, payload: Payload) -> PiResult<()> {
        match self.nodes.try_lock() {
            Ok(mut nodes) => {
                nodes.update_node(node_id, payload)?;
                self.tick_me_later();
                Ok(())
            }
            Err(err) => {
                error!("Error locking nodes: {}", err);
                Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )))
            }
        }
    }

    fn find_existing(&self, payload: &Payload) -> Option<ArcedNodeId> {
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

    fn get_db(&self) -> PiResult<DB> {
        match DB::open_default(self.project_path_on_disk.as_os_str()) {
            Ok(db) => Ok(db),
            Err(err) => {
                error!("Error opening DB: {}", err);
                Err(PiError::InternalError(format!("Error opening DB: {}", err)))
            }
        }
    }

    fn save_to_disk(&self) -> PiResult<()> {
        // We use RocksDB to store the graph
        let db = self.get_db()?;
        match self.nodes.lock() {
            Ok(nodes) => {
                nodes.save_all_to_disk(&db)?;
            }
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )));
            }
        }
        match self.edges.lock() {
            Ok(edges) => {
                edges.save_all_to_disk(&db)?;
            }
            Err(err) => {
                error!("Error locking edges: {}", err);
            }
        }
        Ok(())
    }

    fn load_from_disk(&mut self) -> PiResult<()> {
        let last_node_id = match self.nodes.lock() {
            Ok(mut nodes) => nodes.load_all_from_disk(&self.project_path_on_disk.as_path())?,
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )));
            }
        };
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
        match self.edges.lock() {
            Ok(mut edges) => {
                edges.load_all_from_disk(&self.project_path_on_disk.as_path())?;
            }
            Err(err) => {
                error!("Error locking edges: {}", err);
            }
        }
        Ok(())
    }

    pub fn get_node_ids_connected_with_label(
        &self,
        starting_node_id: &NodeId,
        label: &EdgeLabel,
    ) -> PiResult<Vec<ArcedNodeId>> {
        match self.edges.try_lock() {
            Ok(edges) => match edges.data.get(starting_node_id) {
                Some(edges_from_node) => Ok(edges_from_node
                    .iter()
                    .filter_map(|(x_node_id, x_label)| {
                        if **x_label == *label {
                            Some(x_node_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect()),
                None => Err(PiError::GraphError(format!(
                    "Node {} does not exist",
                    starting_node_id
                ))),
            },
            Err(err) => Err(PiError::GraphError(format!("Error locking edges: {}", err))),
        }
    }

    pub fn fetch_url(&self, url: &str, link_node_id: &NodeId) -> PiResult<()> {
        let engine = Arc::new(self);
        let link_node = match engine.get_node_by_id(link_node_id) {
            Some(node) => node,
            None => {
                error!("Cannot find link node for URL {}", url);
                return Err(PiError::GraphError(format!(
                    "Cannot find link node for URL {}",
                    url
                )));
            }
        };
        if link_node.flags.contains(NodeFlags::IS_REQUESTING) {
            debug!("Link node {} is already being fetched", link_node_id);
            return Ok(());
        }
        debug!("Checking if we can fetch within domain: {}", url);
        let existing_domain: Option<(ArcedNodeItem, ArcedNodeId)> =
            Domain::find_existing(engine.clone(), FindDomainOf::Node(link_node_id.clone()))?;
        let (domain, _) = match existing_domain {
            Some(existing_domain) => existing_domain,
            None => {
                error!("Cannot find domain node for URL {}", url);
                return Err(PiError::GraphError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };

        let domain_payload = match domain.payload {
            Payload::Domain(ref payload) => {
                if !payload.is_allowed_to_crawl {
                    debug!("Domain is not allowed to crawl: {}", &payload.name);
                    return Err(PiError::FetchError(
                        "Domain is not allowed to crawl".to_string(),
                    ));
                }
                payload
            }
            _ => {
                return Err(PiError::GraphError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };
        debug!("Domain {} is allowed to crawl", &domain_payload.name);

        let full_url = format!("https://{}{}", domain_payload.name, url);
        match self
            .fetcher_tx
            .blocking_send(PiEvent::FetchRequest(FetchRequest {
                project_id: self.project_id.clone(),
                node_id: *link_node_id,
                domain: domain_payload.name.clone(),
                url: full_url.clone(),
            })) {
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

    pub fn get_all_node_labels(&self) -> Vec<ArcedNodeLabel> {
        match self.node_ids_by_label.try_lock() {
            Ok(node_ids_by_label) => node_ids_by_label.data.keys().map(|x| x.clone()).collect(),
            Err(err) => {
                error!("Could not lock node_ids_by_label: {}", err);
                vec![]
            }
        }
    }

    pub fn get_node_ids_with_label(&self, label: &NodeLabel) -> Vec<ArcedNodeId> {
        match self.node_ids_by_label.try_lock() {
            Ok(node_ids_by_label) => match node_ids_by_label.data.get(label) {
                Some(node_ids) => node_ids.clone(),
                None => vec![],
            },
            Err(err) => {
                error!("Could not lock node_ids_by_label: {}", err);
                vec![]
            }
        }
    }

    pub fn map_nodes(
        &self,
        f: impl Fn(&ArcedNodeId, &ArcedNodeItem) -> Option<NodeItem>,
    ) -> PiResult<Vec<Option<NodeItem>>> {
        // TODO: Create a version which can take a closure which captures and updates the
        // environment of the function in parameter instead of returning Option<NodeItem>
        let node_ids: Vec<ArcedNodeId> = match self.nodes.try_lock() {
            Ok(nodes) => nodes.data.keys().map(|x| x.clone()).collect(),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )));
            }
        };
        let mut results: Vec<Option<NodeItem>> = vec![];
        for node_id in node_ids {
            let node = self.get_node_by_id(&node_id);
            if let Some(node) = node {
                results.push(f(&node_id, &node));
            }
        }
        Ok(results)
    }

    pub fn get_all_nodes(&self) -> Vec<ArcedNodeItem> {
        match self.nodes.try_lock() {
            Ok(nodes) => nodes.data.values().map(|x| x.clone()).collect(),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                vec![]
            }
        }
    }

    pub fn get_all_edges(&self) -> HashMap<ArcedNodeId, Vec<(ArcedNodeId, ArcedEdgeLabel)>> {
        match self.edges.try_lock() {
            Ok(edges) => edges
                .data
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            Err(err) => {
                error!("Error locking edges: {}", err);
                HashMap::new()
            }
        }
    }

    pub fn set_flag(&self, node_id: &NodeId, flag: NodeFlags) -> PiResult<()> {
        match self.nodes.try_lock() {
            Ok(mut nodes) => {
                nodes.update_flag(node_id, flag);
                self.tick_me_later();
                Ok(())
            }
            Err(err) => {
                error!("Error locking nodes: {}", err);
                Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )))
            }
        }
    }
}

// fn read_le_u32(input: &mut &[u8]) -> u32 {
//     let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
//     *input = rest;
//     u32::from_le_bytes(int_bytes.try_into().unwrap())
// }
