use super::{
    ArcedEdgeLabel, ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, EdgeLabel, ExistingOrNewNodeId,
    Node, NodeFlags, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::api::handle_engine_api_request;
use crate::engine::edges::Edges;
use crate::engine::nodes::Nodes;
use crate::entity::search::SearchTerm;
use crate::entity::topic::Topic;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::{ExternalData, FetchRequest, PiChannel, PiEvent};
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
                            let engine = Arc::new(self);
                            let node_id = Arc::new(response.node_id.clone());
                            match node.payload {
                                Payload::Domain(ref payload) => {
                                    match payload.process(
                                        engine,
                                        &node_id,
                                        Some(ExternalData::Response(response)),
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error processing Domain: {}", err);
                                        }
                                    }
                                }
                                Payload::Link(ref payload) => {
                                    match payload.process(
                                        engine,
                                        &node_id,
                                        Some(ExternalData::Response(response)),
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error processing Link: {}", err);
                                        }
                                    }
                                }
                                Payload::Topic(ref payload) => {
                                    match payload.process(
                                        engine,
                                        &node_id,
                                        Some(ExternalData::Response(response)),
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error processing Topic: {}", err);
                                        }
                                    }
                                }
                                _ => {}
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
                            let node_id = Arc::new(error.node_id.clone());
                            match node.payload {
                                Payload::Domain(ref payload) => {
                                    match payload.process(
                                        engine,
                                        &node_id,
                                        Some(ExternalData::Error(error)),
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error processing Domain: {}", err);
                                        }
                                    }
                                }
                                Payload::Link(ref payload) => {
                                    match payload.process(
                                        engine,
                                        &node_id,
                                        Some(ExternalData::Error(error)),
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error processing Link: {}", err);
                                        }
                                    }
                                }
                                Payload::Topic(ref payload) => {
                                    match payload.process(
                                        engine,
                                        &node_id,
                                        Some(ExternalData::Error(error)),
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error processing Topic: {}", err);
                                        }
                                    }
                                }
                                _ => {}
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

    fn process_nodes(&self) {
        let arced_self = Arc::new(self);
        let node_ids: Vec<ArcedNodeId> = match self.nodes.try_lock() {
            Ok(nodes) => nodes
                .data
                .iter()
                .filter_map(|item| {
                    if item.1.flags.contains(NodeFlags::IS_PROCESSED)
                        || item.1.flags.contains(NodeFlags::IS_REQUESTING)
                        || item.1.flags.contains(NodeFlags::IS_BLOCKED)
                    {
                        None
                    } else {
                        match item.1.payload {
                            Payload::Domain(_) | Payload::Link(_) | Payload::FileHTML(_) | Payload::Topic(_) => {
                                Some(item.0.clone())
                            }
                            _ => None,
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
                match node.payload {
                    Payload::Domain(ref payload) => {
                        match payload.process(arced_self.clone(), &node_id, None) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error processing domain: {}", err);
                            }
                        }
                    }
                    Payload::Link(ref payload) => {
                        match payload.process(arced_self.clone(), &node_id, None) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error processing link: {}", err);
                            }
                        }
                    }
                    Payload::FileHTML(ref payload) => {
                        match payload.process(arced_self.clone(), &node_id, None) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error processing WebPage: {}", err);
                            }
                        }
                    }
                    Payload::Topic(ref payload) => {
                        match payload.process(arced_self.clone(), &node_id, None) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error processing Topic: {}", err);
                            }
                        }
                    }
                    _ => {}
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

        // Add the payload.to_string() to the labels
        let mut labels = labels.clone();
        labels.push(payload.to_string());

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
        if let Some(existing_node_id) = self.find_existing_node(&payload, find_related_to)? {
            // If there is the same payload saved in the graph, we do not add a new node
            return Ok(ExistingOrNewNodeId::Existing(*existing_node_id.1));
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
        payload: &Payload,
        find_related_to: Option<NodeId>,
    ) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        // For certain node payloads, check if there is a node with the same payload
        let engine = Arc::new(self);
        match payload {
            Payload::Domain(ref domain) => {
                Domain::find_existing(engine, FindDomainOf::DomainName(&domain.name))
            }
            Payload::Link(ref link) => {
                Link::find_existing(engine, &link.get_full_link(), find_related_to)
            }
            Payload::Topic(ref topic) => {
                Topic::find_existing(engine, &topic.0)
            }
            Payload::SearchTerm(ref search_term) => {
                SearchTerm::find_existing(engine, &search_term.0)
            }
            _ => Ok(None),
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
        starting_node_id: &NodeId,
        label: &EdgeLabel,
    ) -> PiResult<Vec<ArcedNodeId>> {
        match self.edges.try_lock() {
            Ok(edges) => {
                let mut connnected_node_ids: Vec<ArcedNodeId> = vec![];

                match edges.data.get(starting_node_id) {
                    Some(edges_from_node) => {
                        for (node_id, node_label) in edges_from_node {
                            if **node_label == *label && !connnected_node_ids.contains(node_id) {
                                connnected_node_ids.push(node_id.clone());
                            }
                        }
                    },
                    None => {},
                };
                Ok(connnected_node_ids)
            },
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

        let (domain, domain_node_id): (ArcedNodeItem, NodeId) = match calling_node.payload {
            Payload::Link(_) => {
                let existing_domain: Option<(ArcedNodeItem, ArcedNodeId)> = Domain::find_existing(
                    engine.clone(),
                    FindDomainOf::Node(calling_node_id.clone()),
                )?;
                match existing_domain {
                    Some(existing_domain) => (existing_domain.0, *existing_domain.1),
                    None => {
                        error!("Cannot find domain node for URL {}", url);
                        return Err(PiError::GraphError(format!(
                            "Cannot find domain node for URL {}",
                            url
                        )));
                    }
                }
            }
            Payload::Domain(_) => (calling_node.clone(), calling_node_id.clone()),
            _ => {
                error!("Cannot find domain node for URL {}", url);
                return Err(PiError::GraphError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };

        if domain.flags.contains(NodeFlags::IS_BLOCKED) {
            // debug!("Domain is blocked, cannot fetch");
            return Ok(());
        }

        let domain_payload = match domain.payload {
            Payload::Domain(ref payload) => {
                if !payload.is_allowed_to_crawl {
                    debug!("Domain is not allowed to crawl: {}", &payload.name);
                    self.toggle_flag(&calling_node_id, NodeFlags::IS_BLOCKED)?;
                    return Ok(());
                }
                if *calling_node_id != domain_node_id {
                    // Find the RobotsTxt node connected to the domain node
                    let connected_node_ids = self.get_node_ids_connected_with_label(
                        &domain_node_id,
                        &CommonEdgeLabels::OwnerOf.to_string(),
                    )?;
                    if connected_node_ids.len() == 0 {
                        // We will try to fetch the robots.txt file in the next tick
                        debug!("robots.txt node not found for domain {}", payload.name);
                        return Ok(());
                    }
                    for connected_node_id in connected_node_ids {
                        match self.get_node_by_id(&*connected_node_id) {
                            Some(node) => match node.payload {
                                Payload::Text(ref robots_txt) => {
                                    if robots_txt.is_empty() {
                                        debug!("robots.txt is empty for domain {}", payload.name);
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
                                        if !robot.allowed(&url) {
                                            debug!("URL {} is not allowed to crawl", url);
                                            return Err(PiError::FetchError(
                                                "URL is not allowed to crawl".to_string(),
                                            ));
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

        self.toggle_flag(calling_node_id, NodeFlags::IS_REQUESTING)?;
        self.count_open_fetch_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let full_url = format!("https://{}{}", domain_payload.name, url);
        debug!("Fetching URL {}", &full_url);

        match self
            .fetcher_tx
            .blocking_send(PiEvent::FetchRequest(FetchRequest {
                project_id: self.project_id.clone(),
                node_id: *calling_node_id,
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

    pub fn get_all_node_labels(&self) -> Vec<NodeLabel> {
        match self.nodes.try_lock() {
            Ok(nodes) => {
                let mut labels: HashSet<NodeLabel> = HashSet::new();
                for node in nodes.data.values() {
                    labels.insert(node.get_label());
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
                    if node.get_label() == *label {
                        Some(id.clone())
                    } else if node.labels.contains(label) {
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
