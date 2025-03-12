use super::{ArcedEdgeLabel, CommonEdgeLabels, EdgeLabel, NodeFlags};
use crate::engine::api::handle_engine_api_request;
use crate::engine::edges::Edges;
use crate::engine::node::{
    ArcedNodeId, ArcedNodeItem, ExistingOrNewNodeId, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::nodes::Nodes;
use crate::entity::search::SearchTerm;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::{FetchRequest, PiChannel, PiEvent};
use chrono::Utc;
use log::{debug, error, info};
use rocksdb::DB;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU32;
use std::time::Duration;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use texting_robots::Robot;

// The engine keeps track of all the data nodes and their relationships
pub struct Engine {
    nodes: Mutex<Nodes>, // All nodes that are in the engine
    edges: Mutex<Edges>,
    // node_ids_by_label: Mutex<NodeIdsByLabel>,
    last_node_id: AtomicU32,
    project_id: String,
    path_to_storage_dir: PathBuf,

    my_pi_channel: PiChannel, // Used to communicate with the main thread
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
    fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,

    count_open_fetch_requests: AtomicU32,
    arced_db: Option<Arc<DB>>,
}

impl Engine {
    fn new(
        project_id: String,
        path_to_storage_dir: PathBuf,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> Engine {
        let engine = Engine {
            nodes: Mutex::new(Nodes::new()),
            edges: Mutex::new(Edges::new()),
            // node_ids_by_label: Mutex::new(NodeIdsByLabel::new()),
            last_node_id: AtomicU32::new(0),
            project_id,
            path_to_storage_dir,

            my_pi_channel,
            main_channel_tx,
            fetcher_tx,

            count_open_fetch_requests: AtomicU32::new(0),
            arced_db: None,
        };
        engine
    }

    pub fn open_project(
        path_to_storage_dir: PathBuf,
        project_id: &String,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> Engine {
        Engine::new(
            project_id.clone(),
            path_to_storage_dir,
            my_pi_channel,
            main_channel_tx,
            fetcher_tx,
        )
    }

    pub fn ticker(&self) {
        loop {
            thread::sleep(Duration::from_millis(2000));
            self.process_nodes();
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

    pub fn channel_listener(&self) {
        // We block on the channel of this engine
        let arced_self = Arc::new(self);
        for event in self.my_pi_channel.rx.iter() {
            match event {
                PiEvent::APIRequest(project_id, request) => {
                    if self.project_id == project_id {
                        match handle_engine_api_request(
                            request,
                            arced_self.clone(),
                            self.main_channel_tx.clone(),
                        ) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error handling API request: {}", err);
                            }
                        }
                    }
                }
                PiEvent::FetchResponse(response) => {
                    // Call the node that had needs this data
                    match self.get_node_by_id(&response.node_id) {
                        Some(node) => {
                            match self.toggle_flag(&node.id, NodeFlags::IS_REQUESTING) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!(
                                        "Error toggling IS_REQUESTING flag for node with ID {}: {}",
                                        &node.id, err
                                    );
                                    continue;
                                }
                            }
                            // Reduce the number of open fetch requests by 1
                            self.count_open_fetch_requests
                                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                            let arced_engine = Arc::new(self);
                            match node.handle_fetch_response(arced_engine.clone(), response) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Error processing node: {}", err);
                                }
                            }
                        }
                        None => {}
                    }
                }
                PiEvent::FetchError(error) => {
                    // We have received the error from the previous request
                    match self.get_node_by_id(&error.node_id) {
                        Some(node) => {
                            match self.toggle_flag(&node.id, NodeFlags::IS_REQUESTING) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!(
                                        "Error toggling IS_REQUESTING flag for node with ID {}: {}",
                                        &node.id, err
                                    );
                                    continue;
                                }
                            }
                            // Reduce the number of open fetch requests by 1
                            self.count_open_fetch_requests
                                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                            let engine = Arc::new(self);
                            match node.handle_fetch_error(engine, error) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Error processing node: {}", err);
                                }
                            }
                        }
                        None => {}
                    };
                }
                event => {
                    info!("Unhandled event: {}", event.to_string());
                }
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

    pub fn process_nodes(&self) {
        let arced_self = Arc::new(self);
        let labels_to_be_processed = vec![
            NodeLabel::Link,
            NodeLabel::Domain,
            NodeLabel::WebPage,
            NodeLabel::Objective,
        ];
        let flags_to_be_skipped = vec![
            NodeFlags::IS_PROCESSED,
            NodeFlags::IS_REQUESTING,
            NodeFlags::IS_BLOCKED,
        ];
        let node_ids: Vec<NodeId> = match self.nodes.try_lock() {
            Ok(nodes) => nodes
                .data
                .iter()
                .filter_map(|item| {
                    if flags_to_be_skipped
                        .iter()
                        .any(|flag| item.1.flags.contains(flag.clone()))
                    {
                        None
                    } else {
                        if labels_to_be_processed
                            .iter()
                            .any(|label| item.1.labels.contains(label))
                        {
                            Some(item.1.id)
                        } else {
                            None
                        }
                    }
                })
                .collect(),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return;
            }
        };
        if node_ids.len() > 200 {
            return;
        }
        for node_id in node_ids {
            if let Some(node) = self.get_node_by_id(&node_id) {
                match node.process(arced_self.clone()) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Error processing node: {}", err);
                    }
                }
            }
        }
    }

    fn get_db_path(&self) -> PathBuf {
        let mut path_to_storage_dir = self.path_to_storage_dir.clone();
        path_to_storage_dir.push(format!("{}.rocksdb", self.project_id));
        path_to_storage_dir
    }

    pub fn init_db(&mut self) -> PiResult<()> {
        match DB::open_default(self.get_db_path().as_os_str()) {
            Ok(db) => {
                self.arced_db = Some(Arc::new(db));
                Ok(())
            }
            Err(err) => {
                error!("Cannot open DB for project: {}", err);
                Err(PiError::InternalError(format!(
                    "Cannot open DB for project: {}",
                    err
                )))
            }
        }
    }

    fn get_arced_db(&self) -> PiResult<Arc<DB>> {
        match self.arced_db.as_ref() {
            Some(db) => Ok(db.clone()),
            None => Err(PiError::InternalError(
                "Cannot get DB: DB is not initialized".to_string(),
            )),
        }
    }

    fn create_node(&self, id: NodeId, payload: Payload, labels: Vec<NodeLabel>) -> PiResult<()> {
        let arced_id = Arc::new(id);

        // Store the node in the engine
        match self.nodes.try_lock() {
            Ok(mut nodes) => {
                nodes.data.insert(
                    arced_id.clone(),
                    Arc::new(NodeItem {
                        id,
                        payload,

                        labels: labels.clone(),
                        flags: NodeFlags::default(),
                        written_at: Utc::now(),
                    }),
                );
                nodes.save_item_chunk_to_disk(self.get_arced_db()?, &id)?;
            }
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )));
            }
        }

        // TODO: Store the node in nodes_by_label_id for the label from Payload and given labels
        Ok(())
    }

    pub fn get_or_add_node(
        &self,
        payload: Payload,
        labels: Vec<NodeLabel>,
        should_add_new: bool,
        find_related_to: Option<NodeId>,
    ) -> PiResult<ExistingOrNewNodeId> {
        if let Some(existing_node) = self.find_existing_node(&labels, &payload, find_related_to)? {
            // If there is the same payload saved in the graph, we do not add a new node
            return Ok(ExistingOrNewNodeId::Existing(existing_node.id));
        }

        if !should_add_new {
            error!("Could not find existing node and should not add new node");
            return Err(PiError::InternalError(
                "Could not find existing node and should not add new node".to_string(),
            ));
        }

        let id = {
            self.last_node_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        };

        match self.create_node(id, payload, labels) {
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
                edges
                    .data
                    .get_mut(&arced_node_ids.1)
                    .unwrap()
                    .push((arced_node_ids.0.clone(), arced_edge_labels.1.clone()));
                edges.save_item_chunk_to_disk(self.get_arced_db()?, &node_ids.0)?;
                edges.save_item_chunk_to_disk(self.get_arced_db()?, &node_ids.1)?;
            }
            Err(err) => {
                return Err(PiError::InternalError(format!(
                    "Error locking edges: {}",
                    err
                )));
            }
        };
        Ok(())
    }

    pub fn update_node(&self, node_id: &NodeId, payload: Payload) -> PiResult<()> {
        match self.nodes.try_lock() {
            Ok(mut nodes) => {
                nodes.update_node(node_id, payload)?;
                nodes.save_item_chunk_to_disk(self.get_arced_db()?, node_id)?;
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

    fn find_existing_node(
        &self,
        labels: &[NodeLabel],
        payload: &Payload,
        find_related_to: Option<NodeId>,
    ) -> PiResult<Option<ArcedNodeItem>> {
        // For certain node payloads, check if there is a node with the same payload
        let engine = Arc::new(self);
        if labels.contains(&NodeLabel::Domain) {
            match payload {
                Payload::Text(domain) => {
                    Domain::find_existing(engine, FindDomainOf::DomainName(domain))
                }
                _ => Ok(None),
            }
        } else if labels.contains(&NodeLabel::Link) {
            match payload {
                Payload::Link(ref link) => {
                    Link::find_existing(engine, &link.get_full_link(), find_related_to)
                }
                _ => Ok(None),
            }
        } else if labels.contains(&NodeLabel::SearchTerm) {
            match payload {
                Payload::Text(search_term) => SearchTerm::find_existing(engine, search_term),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn load_from_disk(&mut self) -> PiResult<()> {
        let last_node_id = match self.nodes.lock() {
            Ok(mut nodes) => nodes.load_all_from_disk(&self.get_db_path())?,
            Err(err) => {
                error!("Error locking nodes: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    err
                )));
            }
        };
        if last_node_id != 0 {
            self.last_node_id
                .store(last_node_id + 1, std::sync::atomic::Ordering::Relaxed);
        }
        match self.edges.lock() {
            Ok(mut edges) => {
                edges.load_all_from_disk(&self.get_db_path())?;
            }
            Err(err) => {
                error!("Error locking edges: {}", err);
            }
        }
        Ok(())
    }

    pub fn get_node_ids_connected_with_label(
        &self,
        my_node_id: &NodeId,
        my_edge_to_other: &EdgeLabel,
    ) -> PiResult<Vec<ArcedNodeId>> {
        match self.edges.try_lock() {
            Ok(edges) => {
                let mut connected_node_ids: Vec<ArcedNodeId> = vec![];

                match edges.data.get(my_node_id) {
                    Some(edges_from_node) => {
                        for (node_id, node_label) in edges_from_node {
                            if **node_label == *my_edge_to_other
                                && !connected_node_ids.contains(node_id)
                            {
                                connected_node_ids.push(node_id.clone());
                            }
                        }
                    }
                    None => {}
                };
                Ok(connected_node_ids)
            }
            Err(err) => Err(PiError::GraphError(format!("Error locking edges: {}", err))),
        }
    }

    pub fn fetch(&self, url: &str, calling_node_id: &NodeId) -> PiResult<()> {
        // Calling node is usually a Link,
        // but it can also be a Domain when Domain is fetching `robots.txt`
        // We have only a limited number of open fetch requests at a time
        if self
            .count_open_fetch_requests
            .load(std::sync::atomic::Ordering::Relaxed)
            >= 5
        {
            return Ok(());
        }

        let engine = Arc::new(self);
        let calling_node = match engine.get_node_by_id(calling_node_id) {
            Some(node) => node,
            None => {
                error!("Cannot find link node for URL {}", url);
                return Err(PiError::GraphError(format!(
                    "Cannot find link node for URL {}",
                    url
                )));
            }
        };
        if calling_node.flags.contains(NodeFlags::IS_REQUESTING) {
            debug!("Cannot fetch URL {} since it is already fetching", &url);
            return Ok(());
        }
        if calling_node.flags.contains(NodeFlags::IS_BLOCKED) {
            // debug!("Cannot fetch URL {} since it is blocked", &url);
            return Err(PiError::FetchError(format!(
                "Cannot fetch URL {} since it is blocked",
                &url
            )));
        }

        let domain: ArcedNodeItem = if calling_node.labels.contains(&NodeLabel::Link) {
            match calling_node.payload {
                Payload::Link(_) => {
                    let existing_domain: Option<ArcedNodeItem> = Domain::find_existing(
                        engine.clone(),
                        FindDomainOf::Node(calling_node_id.clone()),
                    )?;
                    match existing_domain {
                        Some(existing_domain) => existing_domain,
                        None => {
                            error!("Cannot find domain node for URL {}", url);
                            return Err(PiError::GraphError(format!(
                                "Cannot find domain node for URL {}",
                                url
                            )));
                        }
                    }
                }
                _ => return Err(PiError::InternalError("Expected a Link node".to_string())),
            }
        } else if calling_node.labels.contains(&NodeLabel::Domain) {
            match calling_node.payload {
                Payload::Text(_) => calling_node.clone(),
                _ => {
                    error!("Cannot find domain node for URL {}", url);
                    return Err(PiError::GraphError(format!(
                        "Cannot find domain node for URL {}",
                        url
                    )));
                }
            }
        } else {
            return Err(PiError::InternalError(
                "Expected either a Link or Domain node".to_string(),
            ));
        };

        if domain.flags.contains(NodeFlags::IS_BLOCKED) {
            // debug!("Domain is blocked, cannot fetch");
            return Ok(());
        }

        let domain_name = Domain::get_domain_name(&domain)?;

        // Find the RobotsTxt node connected to the domain node
        match &domain.payload {
            Payload::Text(text) => {
                if calling_node.id != domain.id {
                    let connected_node_ids = self.get_node_ids_connected_with_label(
                        &domain.id,
                        &CommonEdgeLabels::OwnerOf.to_string(),
                    )?;
                    if connected_node_ids.len() == 0 {
                        // We will try to fetch the robots.txt file in the next tick
                        debug!("robots.txt node not found for domain {}", text);
                        return Ok(());
                    }
                    for connected_node_id in connected_node_ids {
                        match self.get_node_by_id(&*connected_node_id) {
                            Some(node) => match node.payload {
                                Payload::Text(ref robots_txt) => {
                                    if robots_txt.is_empty() {
                                        debug!("robots.txt is empty for domain {}", text);
                                        break;
                                    } else {
                                        let robot =
                                            match Robot::new("Pixlie AI", &robots_txt.as_bytes()) {
                                                Ok(robot) => robot,
                                                Err(err) => {
                                                    error!(
                                                    "Error parsing robots.txt for domain {}: {}",
                                                    url, err
                                                );
                                                    return Err(PiError::FetchError(
                                                        "Error parsing robots.txt".to_string(),
                                                    ));
                                                }
                                            };
                                        // Check if we can crawl
                                        if !robot.allowed(&url) {
                                            debug!(
                                                "URL {} is not allowed to crawl by robots.txt",
                                                &url
                                            );
                                            return Err(PiError::FetchError(format!(
                                                "URL {} is not allowed to crawl by robots.txt",
                                                &url,
                                            )));
                                        }
                                        break;
                                    }
                                }
                                _ => {}
                            },
                            None => {}
                        }
                    }
                }
                text
            }
            _ => {
                return Err(PiError::GraphError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };
        debug!("Domain {} is allowed to crawl", &domain_name);

        self.toggle_flag(calling_node_id, NodeFlags::IS_REQUESTING)?;
        self.count_open_fetch_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let full_url = format!("https://{}{}", domain_name, url);
        debug!("Fetching URL {}", &full_url);

        match self
            .fetcher_tx
            .blocking_send(PiEvent::FetchRequest(FetchRequest {
                project_id: self.project_id.clone(),
                node_id: *calling_node_id,
                domain: domain_name.clone(),
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

    pub fn get_all_node_labels(&self) -> Vec<NodeLabel> {
        match self.nodes.try_lock() {
            Ok(nodes) => {
                let mut labels: HashSet<NodeLabel> = HashSet::new();
                for node in nodes.data.values() {
                    labels.extend(node.labels.iter().cloned());
                }
                labels.iter().map(|x| x.clone()).collect()
            }
            Err(err) => {
                error!("Could not lock nodes: {}", err);
                vec![]
            }
        }
    }

    pub fn get_node_ids_with_label(&self, label: &NodeLabel) -> Vec<ArcedNodeId> {
        // TODO: Use a cached HashMap of node_ids_by_label
        match self.nodes.try_lock() {
            Ok(nodes) => nodes
                .data
                .iter()
                .filter_map(|(id, node)| {
                    if node.labels.contains(label) {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect(),
            Err(err) => {
                error!("Could not lock nodes: {}", err);
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

    pub fn toggle_flag(&self, node_id: &NodeId, flag: NodeFlags) -> PiResult<()> {
        match self.nodes.try_lock() {
            Ok(mut nodes) => {
                nodes.toggle_flag(node_id, flag);
                // self.tick_me_later();
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

pub fn get_test_engine() -> Engine {
    let temp_dir = tempfile::Builder::new()
        .prefix("_path_for_test_engine")
        .tempdir()
        .expect("Failed to create temporary path for the _path_for_test_engine.");
    let project_id = "test_project_id".to_string();
    let path_to_storage_dir = PathBuf::from(temp_dir.path());
    let channel_for_engine = PiChannel::new();
    let main_channel = PiChannel::new();
    let pi_channel_tx = main_channel.tx.clone();
    let (fetcher_tx, _fetcher_rx) = tokio::sync::mpsc::channel::<PiEvent>(100);
    let mut engine = Engine::open_project(
        path_to_storage_dir,
        &project_id,
        channel_for_engine,
        pi_channel_tx,
        fetcher_tx,
    );
    engine.init_db().unwrap();
    engine
}
