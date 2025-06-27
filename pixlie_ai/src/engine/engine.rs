// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use super::{EdgeLabel, NodeEdges, NodeFlags};
use crate::engine::api::{handle_engine_api_request, EngineResponsePayload};
use crate::engine::edges::Edges;
use crate::engine::node::{
    ArcedNodeId, ArcedNodeItem, ExistingOrNewNodeId, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::nodes::Nodes;
use crate::entity::conclusion::Conclusion;
use crate::entity::pixlie::{enable_tools_by_labels, is_tool_enabled};
use crate::entity::search::saved_search::SavedSearch;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::projects::{Project, ProjectOwner};
use crate::utils::notifier::EmailNotifier;
use crate::{FetchRequest, InternalFetchRequest, PiChannel, PiEvent};
use chrono::{TimeDelta, Utc};
use log::{debug, error, info};
use rocksdb::DB;
use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{path::PathBuf, sync::Arc, thread};
use texting_robots::Robot;

// The engine keeps track of all the data nodes and their relationships
pub struct Engine {
    nodes: RwLock<Nodes>, // All nodes that are in the engine
    edges: RwLock<Edges>,

    last_node_id: AtomicU32,

    project_uuid: String,
    arced_db: Arc<DB>,

    my_pi_channel: PiChannel, // Used to communicate with the main thread
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
    fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,

    count_open_fetch_requests: AtomicU32,
    last_monitoring_check: AtomicU64, // Timestamp of last monitoring check for change detection
    email_notifier: EmailNotifier,    // For sending email notifications about content changes
}

impl Engine {
    pub fn open(
        project_uuid: &str,
        path_to_storage_dir: &PathBuf,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> PiResult<Self> {
        let path_to_db = path_to_storage_dir.join(format!("{}.rocksdb", project_uuid));

        // Load all nodes and edges from the database
        // We need to run this before loading the project database
        // since nodes.load_all_from_disk() & edges.load_all_from_disk() need
        // to open the database with their prefix extractors and once the database
        // is opened(and locked), we cannot open it again with a different prefix
        // extractor or set a prefix extractor
        let (nodes, last_node_id) = Nodes::open(&path_to_db)?;
        let edges = Edges::open(&path_to_db)?;
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(false);
        let db = match DB::open(&opts, path_to_db.as_os_str()) {
            Ok(db) => db,
            Err(err) => {
                error!("Could not open DB for project {}: {}", project_uuid, err);
                return Err(err.into());
            }
        };

        let engine = Engine {
            nodes: RwLock::new(nodes),
            edges: RwLock::new(edges),

            last_node_id: AtomicU32::new(0),

            project_uuid: project_uuid.to_string(),
            arced_db: Arc::new(db),

            my_pi_channel,
            main_channel_tx,
            fetcher_tx,

            count_open_fetch_requests: AtomicU32::new(0),
            last_monitoring_check: AtomicU64::new(0),
            email_notifier: EmailNotifier::from_workspace(),
        };

        if last_node_id != 0 {
            engine
                .last_node_id
                .store(last_node_id + 1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(engine)
    }

    pub fn get_project_id(&self) -> &str {
        &self.project_uuid
    }

    pub fn ticker(&self) {
        loop {
            thread::sleep(Duration::from_millis(2000));
            match Project::check_project_db(&self.project_uuid) {
                Ok(_) => {}
                Err(_) => {
                    self.exit();
                    break;
                }
            }
            self.process_nodes();
            self.check_for_url_changes();
        }
    }

    fn exit(&self) {
        // We tell the main thread that we are done ticking
        error!(
            "Engine exiting for project {}\nBacktrace:\n{}",
            self.project_uuid,
            Backtrace::capture()
        );
        match self
            .main_channel_tx
            .send(PiEvent::EngineExit(self.project_uuid.clone()))
        {
            Ok(_) => {}
            Err(err) => {
                error!("Error sending PiEvent::EngineExit in Engine: {}", err);
            }
        }
    }

    pub fn channel_listener(&self) {
        // We block on the channel of this engine
        let arced_self = Arc::new(self);
        for event in self.my_pi_channel.rx.iter() {
            match event {
                PiEvent::APIRequest {
                    project_id,
                    request_id,
                    payload,
                } => {
                    let request_id = request_id;
                    match handle_engine_api_request(
                        project_id.clone(),
                        request_id,
                        payload,
                        arced_self.clone(),
                        self.main_channel_tx.clone(),
                    ) {
                        Ok(_) => {}
                        Err(error) => {
                            match self.main_channel_tx.send(PiEvent::APIResponse {
                                project_id,
                                request_id,
                                payload: EngineResponsePayload::Error(error.to_string()),
                            }) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Error sending PiEvent in Engine: {}", err);
                                }
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
        match self.nodes.read() {
            Ok(nodes) => nodes.data.get(node_id).map(|x| x.clone()),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                None
            }
        }
    }

    pub fn process_nodes(&self) {
        let arced_self = Arc::new(self);

        let flags_to_be_skipped = vec![
            NodeFlags::IS_PROCESSED,
            NodeFlags::IS_REQUESTING,
            NodeFlags::IS_BLOCKED,
        ];

        // TODO: Make dependency processing more generic,
        // so edge based dependencies can be added
        // For now, we cautiously check only nodes that are known to currently have solo nodes.
        // For e.g. Objective
        let mut processing_dependencies: HashMap<NodeLabel, Vec<NodeLabel>> = HashMap::new();
        processing_dependencies.insert(NodeLabel::WebPage, vec![NodeLabel::Objective]);
        processing_dependencies.insert(NodeLabel::WebSearch, vec![NodeLabel::Objective]);
        let mut dependency_labels_to_check = vec![];
        processing_dependencies
            .iter()
            .for_each(|(_, dependencies)| {
                dependencies.iter().for_each(|dependency| {
                    if !dependency_labels_to_check.contains(dependency) {
                        dependency_labels_to_check.push(dependency.clone());
                    }
                });
            });

        // When the number of the nodes to be processed is large,
        // we do not process nodes that generate more nodes. WebPage nodes are like that.
        let limited_labels_to_be_processed = vec![
            NodeLabel::DomainName,
            NodeLabel::Objective,
            NodeLabel::WebPage,
            NodeLabel::WebSearch,
            NodeLabel::Conclusion,
        ];
        let all_labels_to_be_processed = vec![
            NodeLabel::DomainName,
            NodeLabel::Link,
            NodeLabel::Objective,
            NodeLabel::WebPage,
            NodeLabel::WebSearch,
            NodeLabel::Conclusion,
        ];
        let mut node_count: usize = 0;
        let current_time = Utc::now();
        let mut node_ids: Vec<NodeId> = {
            let nodes = match self.nodes.read() {
                Ok(nodes) => nodes,
                Err(err) => {
                    error!("Error locking nodes: {}", err);
                    return;
                }
            };
            let mut processing_status: HashMap<NodeLabel, bool> = HashMap::new();
            nodes.data.iter().for_each(|item| {
                if dependency_labels_to_check
                    .iter()
                    .any(|label| item.1.labels.contains(label))
                    && item.1.flags.contains(NodeFlags::IS_PROCESSED)
                {
                    processing_status.insert(item.1.labels[0].clone(), true);
                }
            });
            nodes
                .data
                .iter()
                .filter_map(|item| {
                    // Skip nodes that are not ready to be processed:
                    // - If the node has one of the flags to be skipped
                    // - If the node depends on other nodes having been processed
                    if flags_to_be_skipped
                        .iter()
                        .any(|flag| item.1.flags.contains(flag.clone()))
                        || (processing_dependencies.iter().any(|(label, dependencies)| {
                            item.1.labels.contains(label)
                                && dependencies.iter().all(|dependency| {
                                    *processing_status.get(dependency).unwrap_or_else(|| &false)
                                })
                        }))
                    {
                        None
                    } else if item.1.flags.contains(NodeFlags::HAD_ERROR) {
                        if current_time - item.1.written_at > TimeDelta::seconds(60) {
                            Some(*item.0.deref())
                        } else {
                            None
                        }
                    } else {
                        if node_count < 100
                            && all_labels_to_be_processed
                                .iter()
                                .any(|label| item.1.labels.contains(label))
                        {
                            node_count += 1;
                            Some(*item.0.deref())
                        } else if limited_labels_to_be_processed
                            .iter()
                            .any(|label| item.1.labels.contains(label))
                        {
                            Some(*item.0.deref())
                        } else {
                            None
                        }
                    }
                })
                .collect()
        };
        node_ids.sort();

        // info!(
        //     "Processing nodes {}",
        //     node_ids.iter().map(|x| x.to_string()).join(", ")
        // );
        for node_id in node_ids {
            if let Some(node) = self.get_node_by_id(&node_id) {
                match node.process(arced_self.clone()) {
                    Ok(_) => {}
                    Err(error) => {
                        error!("Error processing node {}: {}", node_id, error);
                    }
                }
            }
        }
    }

    fn create_node(&self, id: NodeId, payload: Payload, labels: Vec<NodeLabel>) -> PiResult<()> {
        let arced_id = Arc::new(id);

        // Store the node in the engine
        match self.nodes.write() {
            Ok(mut nodes) => {
                nodes.data.insert(
                    arced_id.clone(),
                    Arc::new(NodeItem {
                        id,
                        payload,

                        labels,
                        flags: NodeFlags::default(),
                        written_at: Utc::now(),
                    }),
                );
                nodes.save_item_chunk_to_disk(self.arced_db.clone(), &id)?;
            }
            Err(error) => {
                return Err(PiError::InternalError(format!(
                    "Error locking nodes: {}",
                    error
                )));
            }
        }
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
        // let arced_edge_labels = (edge_labels.0, edge_labels.1);
        // Add a connection edge from the parent node to the new node and vice versa
        match self.edges.write() {
            Ok(mut edges) => {
                edges
                    .data
                    .entry(arced_node_ids.0.clone())
                    .or_insert(NodeEdges {
                        edges: vec![],
                        written_at: Utc::now(),
                    });
                edges
                    .data
                    .entry(arced_node_ids.1.clone())
                    .or_insert(NodeEdges {
                        edges: vec![],
                        written_at: Utc::now(),
                    });
                // Update connections data for the parent node
                edges
                    .data
                    .get_mut(&arced_node_ids.0)
                    .unwrap()
                    .edges
                    .push((*arced_node_ids.1, edge_labels.0));
                // Update the last written time for the parent node
                edges.data.get_mut(&arced_node_ids.0).unwrap().written_at = Utc::now();

                // Update connections data for the child node
                edges
                    .data
                    .get_mut(&arced_node_ids.1)
                    .unwrap()
                    .edges
                    .push((*arced_node_ids.0, edge_labels.1));
                // Update the last written time for the child node
                edges.data.get_mut(&arced_node_ids.1).unwrap().written_at = Utc::now();
                edges.save_item_chunk_to_disk(self.arced_db.clone(), &node_ids.0)?;
                edges.save_item_chunk_to_disk(self.arced_db.clone(), &node_ids.1)?;
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
        match self.nodes.write() {
            Ok(mut nodes) => {
                nodes.update_node(node_id, payload)?;
                nodes.save_item_chunk_to_disk(self.arced_db.clone(), node_id)?;
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

    pub fn update_node_flags(&self, node_id: &NodeId, flags: NodeFlags) -> PiResult<()> {
        match self.nodes.write() {
            Ok(mut nodes) => {
                nodes.update_node_flags(node_id, flags)?;
                nodes.save_item_chunk_to_disk(self.arced_db.clone(), node_id)?;
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
        if labels.contains(&NodeLabel::DomainName) {
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
                Payload::Text(search_term) => SavedSearch::find_existing(engine, search_term),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_connected_nodes(&self, my_node_id: &NodeId) -> PiResult<Option<NodeEdges>> {
        // Return all nodes that are connected to me
        let edges = match self.edges.read() {
            Ok(edges) => edges,
            Err(err) => {
                error!("Error locking edges in get_connected_nodes: {}", err);
                return Err(PiError::GraphError(format!(
                    "Error locking edges in get_connected_nodes: {}",
                    err
                )));
            }
        };
        Ok(edges.data.get(my_node_id).cloned())
    }

    pub fn get_node_ids_connected_with_label(
        &self,
        my_node_id: &NodeId,
        my_edge_to_other: &EdgeLabel,
    ) -> PiResult<Vec<NodeId>> {
        let edges = match self.edges.read() {
            Ok(edges) => edges,
            Err(err) => {
                error!(
                    "Error locking edges in get_node_ids_connected_with_label: {}",
                    err
                );
                return Err(PiError::GraphError(format!(
                    "Error locking edges in get_node_ids_connected_with_label: {}",
                    err
                )));
            }
        };
        let mut connected_node_ids: Vec<NodeId> = vec![];
        if let Some(edges_from_node) = edges.data.get(my_node_id) {
            for (node_id, node_label) in edges_from_node.edges.iter() {
                if node_label == my_edge_to_other && !connected_node_ids.contains(node_id) {
                    connected_node_ids.push(node_id.clone());
                }
            }
        }
        Ok(connected_node_ids)
    }

    pub fn fetch(&self, fetch_request: FetchRequest) -> PiResult<()> {
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
        let calling_node = match engine.get_node_by_id(&fetch_request.requesting_node_id) {
            Some(node) => node,
            None => {
                error!("Cannot find link node for URL {}", &fetch_request.url);
                return Err(PiError::GraphError(format!(
                    "Cannot find link node for URL {}",
                    &fetch_request.url
                )));
            }
        };
        if calling_node.flags.contains(NodeFlags::IS_REQUESTING) {
            debug!(
                "Cannot fetch URL {} since it is already fetching",
                &fetch_request.url
            );
            return Ok(());
        }
        if calling_node.flags.contains(NodeFlags::IS_BLOCKED) {
            // debug!("Cannot fetch URL {} since it is blocked", &url);
            return Err(PiError::FetchError(format!(
                "Cannot fetch URL {} since it is blocked",
                &fetch_request.url
            )));
        }

        let domain: ArcedNodeItem = if calling_node.labels.contains(&NodeLabel::Link) {
            match calling_node.payload {
                Payload::Link(_) => {
                    let existing_domain: Option<ArcedNodeItem> = Domain::find_existing(
                        engine.clone(),
                        FindDomainOf::Node(fetch_request.requesting_node_id.clone()),
                    )?;
                    match existing_domain {
                        Some(existing_domain) => existing_domain,
                        None => {
                            error!("Cannot find domain node for URL {}", &fetch_request.url);
                            return Err(PiError::GraphError(format!(
                                "Cannot find domain node for URL {}",
                                &fetch_request.url
                            )));
                        }
                    }
                }
                _ => return Err(PiError::InternalError("Expected a Link node".to_string())),
            }
        } else if calling_node.labels.contains(&NodeLabel::DomainName) {
            match calling_node.payload {
                Payload::Text(_) => calling_node.clone(),
                _ => {
                    error!("Cannot find domain node for URL {}", &fetch_request.url);
                    return Err(PiError::GraphError(format!(
                        "Cannot find domain node for URL {}",
                        &fetch_request.url
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
                    let connected_node_ids =
                        self.get_node_ids_connected_with_label(&domain.id, &EdgeLabel::OwnerOf)?;
                    if connected_node_ids.len() == 0 {
                        // We will try to fetch the robots.txt file in the next tick
                        debug!("robots.txt node not found for domain {}", text);
                        return Ok(());
                    }
                    for connected_node_id in connected_node_ids {
                        match self.get_node_by_id(&connected_node_id) {
                            Some(node) => match node.payload {
                                Payload::Text(ref robots_txt) => {
                                    if robots_txt.is_empty() {
                                        debug!("robots.txt is empty for domain {}", text);
                                        break;
                                    } else {
                                        let robot =
                                            match Robot::new("Pixlie AI", &robots_txt.as_bytes()) {
                                                Ok(robot) => robot,
                                                Err(error) => {
                                                    return Err(PiError::FetchError(format!(
                                                        "Error parsing robots.txt: {}",
                                                        error,
                                                    )));
                                                }
                                            };
                                        // Check if we can crawl
                                        if !robot.allowed(&fetch_request.url) {
                                            return Err(PiError::FetchError(format!(
                                                "URL {} is not allowed to crawl by robots.txt",
                                                &fetch_request.url,
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
                    &fetch_request.url
                )));
            }
        };

        self.toggle_flag(&fetch_request.requesting_node_id, NodeFlags::IS_REQUESTING)?;
        self.count_open_fetch_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        match self.fetcher_tx.blocking_send(PiEvent::FetchRequest(
            InternalFetchRequest::from_crawl_request(
                fetch_request,
                self.project_uuid.clone(),
                domain_name,
            ),
        )) {
            Ok(_) => {}
            Err(err) => {
                return Err(PiError::FetchError(format!(
                    "Error sending request to fetcher: {}",
                    err
                )));
            }
        }
        Ok(())
    }

    pub fn fetch_api(&self, fetch_request: FetchRequest) -> PiResult<()> {
        if self
            .count_open_fetch_requests
            .load(std::sync::atomic::Ordering::Relaxed)
            >= 5
        {
            return Ok(());
        }
        let engine = Arc::new(self);
        let calling_node = match engine.get_node_by_id(&fetch_request.requesting_node_id) {
            Some(node) => node,
            None => {
                error!("Cannot find link node for URL {}", &fetch_request.url);
                return Err(PiError::GraphError(format!(
                    "Cannot find link node for URL {}",
                    &fetch_request.url
                )));
            }
        };
        if calling_node.flags.contains(NodeFlags::IS_REQUESTING) {
            debug!(
                "Cannot fetch URL {} since it is already fetching",
                &fetch_request.url
            );
            return Ok(());
        }
        if calling_node.flags.contains(NodeFlags::IS_BLOCKED) {
            // debug!("Cannot fetch URL {} since it is blocked", &url);
            return Err(PiError::FetchError(format!(
                "Cannot fetch URL {} since it is blocked",
                &fetch_request.url
            )));
        }

        self.toggle_flag(&fetch_request.requesting_node_id, NodeFlags::IS_REQUESTING)?;
        self.count_open_fetch_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        match self.fetcher_tx.blocking_send(PiEvent::FetchRequest(
            InternalFetchRequest::from_api_request(fetch_request, &self.project_uuid),
        )) {
            Ok(_) => {}
            Err(err) => {
                return Err(PiError::FetchError(format!(
                    "Error sending request to fetcher: {}",
                    err
                )));
            }
        }
        Ok(())
    }

    pub fn get_all_node_labels(&self) -> Vec<NodeLabel> {
        let nodes = match self.nodes.read() {
            Ok(nodes) => nodes,
            Err(err) => {
                error!("Could not lock nodes in get_all_node_labels: {}", err);
                return vec![];
            }
        };
        let mut labels: HashSet<NodeLabel> = HashSet::new();
        for node in nodes.data.values() {
            labels.extend(node.labels.iter().cloned());
        }
        labels.iter().map(|x| x.clone()).collect()
    }

    pub fn get_node_ids_with_label(&self, label: &NodeLabel) -> Vec<ArcedNodeId> {
        // TODO: Use a cached HashMap of node_ids_by_label
        let nodes = match self.nodes.read() {
            Ok(nodes) => nodes,
            Err(err) => {
                error!("Could not lock nodes in get_node_ids_with_label: {}", err);
                return vec![];
            }
        };
        nodes
            .data
            .iter()
            .filter_map(|(id, node)| {
                if node.labels.contains(label) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn map_nodes(
        &self,
        f: impl Fn(&ArcedNodeId, &ArcedNodeItem) -> Option<NodeItem>,
    ) -> PiResult<Vec<Option<NodeItem>>> {
        // TODO: Create a version which can take a closure which captures and updates the
        // environment of the function in parameter instead of returning Option<NodeItem>
        let node_ids: Vec<ArcedNodeId> = {
            let nodes = match self.nodes.read() {
                Ok(nodes) => nodes,
                Err(err) => {
                    error!("Error locking nodes in map_nodes: {}", err);
                    return Err(PiError::InternalError(format!(
                        "Error locking nodes in map_nodes: {}",
                        err
                    )));
                }
            };
            nodes.data.keys().cloned().collect()
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
        let nodes = match self.nodes.read() {
            Ok(nodes) => nodes,
            Err(err) => {
                error!("Error locking nodes in get_all_nodes: {}", err);
                return vec![];
            }
        };
        nodes.data.values().map(|x| x.clone()).collect()
    }

    pub fn get_all_edges(&self) -> HashMap<ArcedNodeId, NodeEdges> {
        let edges = match self.edges.read() {
            Ok(edges) => edges,
            Err(err) => {
                error!("Error locking edges in get_all_edges: {}", err);
                return HashMap::new();
            }
        };
        edges
            .data
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn toggle_flag(&self, node_id: &NodeId, flag: NodeFlags) -> PiResult<()> {
        match self.nodes.write() {
            Ok(mut nodes) => {
                nodes.toggle_flag(node_id, flag);
                nodes.save_item_chunk_to_disk(self.arced_db.clone(), node_id)?;
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

    /// Check for URL changes if monitoring is enabled (called every 30 seconds)
    fn check_for_url_changes(&self) {
        // Check if 30 seconds have passed since last monitoring check
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let last_check = self.last_monitoring_check.load(Ordering::Relaxed);
        log::info!("Last monitoring check: {}", last_check);

        // Only check every 5 minutes (300 seconds)
        if now - last_check < 300 {
            return;
        }

        // Check if monitoring is enabled in project settings
        if !self.is_monitoring_enabled() {
            debug!("Monitoring is disabled in project settings, skipping URL checks.");
            return;
        }

        let engine_arc = Arc::new(self);
        if let Ok(is_enabled) = is_tool_enabled(engine_arc.clone(), NodeLabel::ClassifierSettings) {
            if is_enabled {
                debug!("Classifier is enabled, skipping URL checks.");
                return;
            }
        }

        if let Ok(is_enabled) = is_tool_enabled(engine_arc.clone(), NodeLabel::CrawlerSettings) {
            if is_enabled {
                debug!("Crawler is enabled, skipping URL checks.");
                return;
            }
        }

        // Update last check timestamp
        self.last_monitoring_check.store(now, Ordering::Relaxed);

        log::info!("Checking for URL changes in project {}", self.project_uuid);

        // Get only starting URLs (user-added or web search links)
        let all_link_node_ids = self.get_node_ids_with_label(&NodeLabel::Link);
        let starting_links: Vec<_> = all_link_node_ids
            .into_iter()
            .filter(|link_id| {
                if let Some(node) = self.get_node_by_id(link_id) {
                    node.labels.contains(&NodeLabel::AddedByUser)
                        || node.labels.contains(&NodeLabel::AddedByWebSearch)
                } else {
                    false
                }
            })
            .collect();

        debug!(
            "Found {} starting links to monitor for changes",
            starting_links.len()
        );

        // Check each starting link for content changes
        for link_id in starting_links {
            if let Err(e) = self.check_single_url_for_changes(&link_id) {
                error!("Error checking URL for changes (node {}): {}", link_id, e);
            }
        }
    }

    /// Check if monitoring is enabled in project settings
    fn is_monitoring_enabled(&self) -> bool {
        let project_settings_ids = self.get_node_ids_with_label(&NodeLabel::ProjectSettings);

        for settings_id in project_settings_ids {
            if let Some(node) = self.get_node_by_id(&settings_id) {
                if let Payload::ProjectSettings(settings) = &node.payload {
                    return settings.monitor_links_for_changes;
                }
            }
        }

        false // Default to monitoring disabled
    }

    /// Check a single URL for content changes and trigger re-processing if changed
    fn check_single_url_for_changes(&self, link_id: &NodeId) -> PiResult<()> {
        let link_node = match self.get_node_by_id(link_id) {
            Some(node) => node,
            None => return Ok(()), // Node not found, skip
        };

        // Get the Link payload and construct full URL
        let (mut link_data, full_url) = match &link_node.payload {
            Payload::Link(link) => {
                // Find the domain for this link
                let existing_domain =
                    Domain::find_existing(Arc::new(&*self), FindDomainOf::Node(link_id.clone()))?;

                if existing_domain.is_none() {
                    debug!(
                        "Cannot find domain node for link {}, skipping monitoring",
                        link_id
                    );
                    return Ok(());
                }

                let domain_name = Domain::get_domain_name(&existing_domain.unwrap())?;
                let full_url = format!("https://{}{}", domain_name, link.get_full_link());

                (link.clone(), full_url)
            }
            _ => return Ok(()), // Not a Link node, skip
        };

        // Add delay between requests to be respectful to servers
        thread::sleep(Duration::from_millis(2000)); // 2 second delay

        // Fetch current content
        let current_content = match self.fetch_url_content_sync(&full_url) {
            Ok(content) => content,
            Err(e) => {
                debug!("Failed to fetch content for {}: {}", full_url, e);
                return Ok(()); // Skip this URL on fetch failure
            }
        };

        // Calculate current content hash
        let current_hash = self.calculate_content_hash(&current_content);

        // Log the content being hashed (truncated for readability)
        let content_preview = if current_content.len() > 500 {
            format!(
                "{}... ({} total chars)",
                &current_content[..500],
                current_content.len()
            )
        } else {
            current_content.clone()
        };
        debug!("📄 Content for {}: {}", full_url, content_preview);

        // Check if content has changed
        let has_changed = match &link_data.content_hash {
            Some(stored_hash) => {
                let changed = stored_hash != &current_hash;
                if changed {
                    info!(
                        "📊 Hash comparison for {}: stored={}, current={}",
                        full_url, stored_hash, current_hash
                    );
                    info!("📝 Content preview: {}", content_preview);
                } else {
                    debug!(
                        "✅ No change detected for {}: hash={}",
                        full_url, current_hash
                    );
                }
                changed
            }
            None => {
                info!(
                    "🆕 First time checking {}: new hash={}",
                    full_url, current_hash
                );
                info!("📝 Initial content preview: {}", content_preview);
                true // No stored hash means first check, always "changed"
            }
        };

        if has_changed {
            info!("🔄 Content change detected for URL: {}", full_url);

            // Update stored hash
            link_data.content_hash = Some(current_hash);
            self.update_node(link_id, Payload::Link(link_data))?;

            // For now, send a simple notification about the change
            // TODO: Later we'll make this smarter to only notify about relevant changes
            self.send_change_notification(&full_url, &content_preview)?;

            // Trigger re-processing for this URL
            self.trigger_reprocessing_for_url(link_id)?;
        }

        Ok(())
    }

    /// Get a random User-Agent string for monitoring requests
    fn get_random_user_agent(&self) -> &'static str {
        let user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0",
            "Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2.1 Safari/605.1.15",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edge/121.0.0.0 Safari/537.36",
        ];

        // Use current timestamp to get pseudo-random selection
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let index = (timestamp % user_agents.len() as u64) as usize;
        user_agents[index]
    }

    /// Fetch URL content synchronously using reqwest blocking client
    fn fetch_url_content_sync(&self, url: &str) -> PiResult<String> {
        debug!("Fetching content from URL: {}", url);

        // Use reqwest blocking client for HTTP requests with timeout and rotating User-Agent
        let user_agent = self.get_random_user_agent();
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent(user_agent)
            .build()
            .map_err(|e| {
                error!("Failed to create HTTP client: {}", e);
                PiError::FetchError(format!("Failed to create HTTP client: {}", e))
            })?;

        debug!("Using User-Agent: {}", user_agent);

        let response = client.get(url).send().map_err(|e| {
            error!("HTTP request failed for {}: {}", url, e);
            PiError::FetchError(format!("HTTP request failed: {}", e))
        })?;

        // Check if response is successful
        if !response.status().is_success() {
            return Err(PiError::FetchError(format!(
                "HTTP {} for URL: {}",
                response.status(),
                url
            )));
        }

        // Read response body
        let content = response.text().map_err(|e| {
            error!("Failed to read response body for {}: {}", url, e);
            PiError::FetchError(format!("Failed to read response body: {}", e))
        })?;

        debug!("Successfully fetched {} bytes from {}", content.len(), url);
        Ok(content)
    }

    /// Calculate SHA-256 hash of content
    fn calculate_content_hash(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};

        // Normalize content (remove excess whitespace)
        let normalized = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        // Log normalized content for debugging
        let normalized_preview = if normalized.len() > 300 {
            format!(
                "{}... ({} total chars)",
                &normalized[..300],
                normalized.len()
            )
        } else {
            normalized.clone()
        };
        debug!("🔗 Normalized content: {}", normalized_preview);

        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        debug!("🔢 Generated hash: {}", hash);
        hash
    }

    /// Trigger re-processing for a changed URL
    fn trigger_reprocessing_for_url(&self, link_id: &NodeId) -> PiResult<()> {
        // 1. Re-enable CrawlerSettings and ClassifierSettings
        let engine_arc = Arc::new(self);
        enable_tools_by_labels(
            engine_arc,
            vec![NodeLabel::CrawlerSettings, NodeLabel::ClassifierSettings],
        )?;

        // 2. Reset flags for this link and related nodes
        self.reset_node_flags_for_reprocessing(link_id)?;

        info!("✅ Triggered re-processing for link node {}", link_id);
        Ok(())
    }

    /// Send email notification when a webpage is classified as relevant
    pub fn send_insight_notification(
        &self,
        url: &str,
        insight: &str,
        reason: &str,
    ) -> PiResult<()> {
        info!("🎯 send_insight_notification called for URL: {}", url);

        // Get the objective for this project
        let objective = self
            .get_project_objective()
            .unwrap_or_else(|_| "Monitor content changes".to_string());

        // Get user email from notifier configuration
        let user_email = &self.email_notifier.config.to_email;

        info!("📧 Sending insight notification to: {}", user_email);

        let notification = self
            .email_notifier
            .create_insight_notification(user_email, url, &objective, insight, reason);

        self.email_notifier.send_notification(notification)?;
        Ok(())
    }

    /// Send email notification about content changes
    fn send_change_notification(&self, url: &str, content_preview: &str) -> PiResult<()> {
        info!("🚨 send_change_notification called for URL: {}", url);

        // Get the objective for this project
        let objective = self
            .get_project_objective()
            .unwrap_or_else(|_| "Monitor content changes".to_string());

        // Get user email from notifier configuration
        let user_email = &self.email_notifier.config.to_email;

        info!("📧 Sending notification to: {}", user_email);

        // For now, treat any change as relevant (we'll make this smarter later)
        let relevant_items = vec![
            format!("Content change detected at {}", url),
            format!(
                "Preview: {}",
                if content_preview.len() > 100 {
                    format!("{}...", &content_preview[..100])
                } else {
                    content_preview.to_string()
                }
            ),
        ];

        let notification = self.email_notifier.create_content_change_notification(
            &user_email,
            url,
            &objective,
            relevant_items,
        );

        self.email_notifier.send_notification(notification)?;
        Ok(())
    }

    /// Get the project objective text
    fn get_project_objective(&self) -> PiResult<String> {
        let objective_node_ids = self.get_node_ids_with_label(&NodeLabel::Objective);

        for node_id in objective_node_ids {
            if let Some(node) = self.get_node_by_id(&node_id) {
                if let Payload::Text(objective_text) = &node.payload {
                    return Ok(objective_text.clone());
                }
            }
        }

        Err(PiError::InternalError("No objective found".to_string()))
    }

    /// Reset node flags for re-processing
    fn reset_node_flags_for_reprocessing(&self, link_id: &NodeId) -> PiResult<()> {
        // Reset the Link node flags
        if let Some(link_node) = self.get_node_by_id(link_id) {
            if link_node.flags.contains(NodeFlags::IS_PROCESSED) {
                let mut new_flags = link_node.flags.clone();
                new_flags.remove(NodeFlags::IS_PROCESSED);
                new_flags.insert(NodeFlags::IS_REQUESTING);
                self.update_node_flags(link_id, new_flags)?;
                debug!(
                    "🔄 Reset Link node {} flags: IS_PROCESSED → IS_REQUESTING",
                    link_id
                );

                let objective_id = self
                    .get_node_ids_with_label(&NodeLabel::Objective)
                    .first()
                    .ok_or_else(|| PiError::InternalError("No Objective nodes found".to_string()))?
                    .clone();

                let conclusion = self
                    .get_or_add_node(
                        Payload::Conclusion(Conclusion::default()),
                        vec![NodeLabel::AddedByAI, NodeLabel::Conclusion],
                        true,
                        None,
                    )?
                    .get_node_id();

                self.add_connection(
                    (*objective_id, conclusion),
                    (EdgeLabel::ConcludedBy, EdgeLabel::Concludes),
                )?;
                debug!("🔄 Create Conclusion node {}", conclusion);
            }
        }

        // // Reset the WebPage node flags
        // let web_page_ids = self.get_node_ids_with_label(&NodeLabel::WebPage);
        // for web_page_id in web_page_ids {
        //     if let Some(web_page_node) = self.get_node_by_id(&web_page_id) {
        //         if web_page_node.flags.contains(NodeFlags::IS_PROCESSED) {
        //             let mut new_flags = web_page_node.flags.clone();
        //             new_flags.remove(NodeFlags::IS_PROCESSED);
        //             new_flags.insert(NodeFlags::IS_REQUESTING);
        //             self.update_node_flags(&web_page_id, new_flags)?;
        //             debug!("🔄 Reset WebPage node {} flags: IS_PROCESSED → IS_REQUESTING", web_page_id);
        //         }
        //     }
        // }

        // Get first objective node

        Ok(())
    }
}

pub fn get_test_engine() -> Engine {
    let temp_dir = tempfile::Builder::new()
        .prefix("_path_for_test_engine")
        .tempdir()
        .expect("Failed to create temporary path for the _path_for_test_engine.");
    let path_to_storage_dir = PathBuf::from(temp_dir.path());
    let project = Project::new(
        Some("Test project".to_string()),
        Some("Test project description".to_string()),
        ProjectOwner::Myself,
    );
    let path_to_db = path_to_storage_dir.join(format!("{}.rocksdb", &project.uuid));
    Project::create_project_db(&path_to_db).unwrap();
    let channel_for_engine = PiChannel::new();
    let main_channel = PiChannel::new();
    let pi_channel_tx = main_channel.tx.clone();
    let (fetcher_tx, _fetcher_rx) = tokio::sync::mpsc::channel::<PiEvent>(100);
    Engine::open(
        &project.uuid,
        &path_to_storage_dir,
        channel_for_engine,
        pi_channel_tx,
        fetcher_tx,
    )
    .unwrap()
}
