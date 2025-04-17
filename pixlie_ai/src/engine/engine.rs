// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use super::{EdgeLabel, NodeEdges, NodeFlags};
use crate::config::Settings;
use crate::engine::api::{handle_engine_api_request, EngineResponse, EngineResponsePayload};
use crate::engine::edges::Edges;
use crate::engine::node::{
    ArcedNodeId, ArcedNodeItem, ExistingOrNewNodeId, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::nodes::Nodes;
use crate::entity::search::saved_search::SavedSearch;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::projects::{Project, ProjectForEngine, ProjectOwner};
use crate::{FetchRequest, InternalFetchRequest, PiChannel, PiEvent};
use chrono::{TimeDelta, Utc};
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::atomic::AtomicU32;
use std::sync::RwLock;
use std::time::Duration;
use std::{path::PathBuf, sync::Arc, thread};
use texting_robots::Robot;

// The engine keeps track of all the data nodes and their relationships
pub struct Engine {
    nodes: RwLock<Nodes>, // All nodes that are in the engine
    edges: RwLock<Edges>,

    last_node_id: AtomicU32,
    engine_project: ProjectForEngine,

    my_pi_channel: PiChannel, // Used to communicate with the main thread
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
    fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,

    count_open_fetch_requests: AtomicU32,
}

impl Engine {
    fn new(
        project: ProjectForEngine,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> Engine {
        let engine = Engine {
            nodes: RwLock::new(Nodes::new()),
            edges: RwLock::new(Edges::new()),
            // node_ids_by_label: Mutex::new(NodeIdsByLabel::new()),
            last_node_id: AtomicU32::new(0),
            engine_project: project,

            my_pi_channel,
            main_channel_tx,
            fetcher_tx,

            count_open_fetch_requests: AtomicU32::new(0),
        };
        engine
    }

    pub fn open_project(
        engine_project: ProjectForEngine,
        my_pi_channel: PiChannel,
        main_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    ) -> Engine {
        Engine::new(engine_project, my_pi_channel, main_channel_tx, fetcher_tx)
    }

    pub fn load_project_db(&mut self) -> PiResult<()> {
        self.engine_project.load_db()
    }

    pub fn ticker(&self) {
        loop {
            thread::sleep(Duration::from_millis(2000));
            self.process_nodes();
        }
    }

    fn exit(&self) {
        // We tell the main thread that we are done ticking
        match self
            .main_channel_tx
            .send(PiEvent::EngineExit(self.get_project_id().clone()))
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
                PiEvent::APIRequest(project_id, request) => {
                    if self.get_project_id() == project_id {
                        let request_id = request.request_id;
                        match handle_engine_api_request(
                            request,
                            arced_self.clone(),
                            self.main_channel_tx.clone(),
                        ) {
                            Ok(_) => {}
                            Err(error) => {
                                match self.main_channel_tx.send(PiEvent::APIResponse(
                                    project_id,
                                    EngineResponse {
                                        request_id,
                                        payload: EngineResponsePayload::Error(error.to_string()),
                                    },
                                )) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("Error sending PiEvent in Engine: {}", err);
                                    }
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

        // When the number of the nodes to be processed is large,
        // we do not process nodes that generate more nodes. WebPage nodes are like that.
        let limited_labels_to_be_processed = vec![
            NodeLabel::Domain,
            NodeLabel::Objective,
            NodeLabel::WebSearch,
        ];
        let all_labels_to_be_processed = vec![
            NodeLabel::Domain,
            NodeLabel::Link,
            NodeLabel::Objective,
            NodeLabel::WebPage,
            NodeLabel::WebSearch,
        ];
        let mut node_count: usize = 0;
        let current_time = Utc::now();
        let mut node_ids: Vec<NodeId> = match self.nodes.read() {
            Ok(nodes) => nodes
                .data
                .iter()
                .filter_map(|item| {
                    if flags_to_be_skipped
                        .iter()
                        .any(|flag| item.1.flags.contains(flag.clone()))
                    {
                        None
                    } else if item.1.flags.contains(NodeFlags::HAD_ERROR) {
                        if current_time - item.1.written_at > TimeDelta::seconds(60) {
                            Some(*item.0.deref())
                        } else {
                            None
                        }
                    } else {
                        if node_count < 10
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
                .collect(),
            Err(err) => {
                error!("Error locking nodes in process_nodes: {}", err);
                return;
            }
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
                    Err(_error) => {}
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
                nodes.save_item_chunk_to_disk(self.engine_project.get_arced_db()?, &id)?;
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
                edges.save_item_chunk_to_disk(self.engine_project.get_arced_db()?, &node_ids.0)?;
                edges.save_item_chunk_to_disk(self.engine_project.get_arced_db()?, &node_ids.1)?;
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
                nodes.save_item_chunk_to_disk(self.engine_project.get_arced_db()?, node_id)?;
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
                Payload::Text(search_term) => SavedSearch::find_existing(engine, search_term),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn preload_data_from_db(&mut self) -> PiResult<()> {
        // Load all nodes and edges from the database
        // We need to run this before loading the project database
        // since nodes.load_all_from_disk() & edges.load_all_from_disk() need
        // to open the database with their prefix extractors and once the database
        // is opened(and locked), we cannot open it again with a different prefix
        // extractor or set a prefix extractor
        let last_node_id = match self.nodes.write() {
            Ok(mut nodes) => nodes.load_all_from_disk(&self.engine_project.get_db_path())?,
            Err(err) => {
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
        match self.edges.write() {
            Ok(mut edges) => {
                edges.load_all_from_disk(&self.engine_project.get_db_path())?;
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
    ) -> PiResult<Vec<NodeId>> {
        match self.edges.try_read() {
            Ok(edges) => {
                let mut connected_node_ids: Vec<NodeId> = vec![];

                match edges.data.get(my_node_id) {
                    Some(edges_from_node) => {
                        for (node_id, node_label) in edges_from_node.edges.iter() {
                            if node_label == my_edge_to_other
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
        } else if calling_node.labels.contains(&NodeLabel::Domain) {
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
                self.get_project_id(),
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
            InternalFetchRequest::from_api_request(fetch_request, self.get_project_id()),
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
        match self.nodes.try_read() {
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
        match self.nodes.try_read() {
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
        let node_ids: Vec<ArcedNodeId> = match self.nodes.try_read() {
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
        match self.nodes.try_read() {
            Ok(nodes) => nodes.data.values().map(|x| x.clone()).collect(),
            Err(err) => {
                error!("Error locking nodes: {}", err);
                vec![]
            }
        }
    }

    pub fn get_all_edges(&self) -> HashMap<ArcedNodeId, NodeEdges> {
        match self.edges.try_read() {
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

    pub fn get_project_id(&self) -> String {
        self.engine_project.project.uuid.clone()
    }

    pub fn toggle_flag(&self, node_id: &NodeId, flag: NodeFlags) -> PiResult<()> {
        match self.nodes.write() {
            Ok(mut nodes) => {
                nodes.toggle_flag(node_id, flag);
                nodes.save_item_chunk_to_disk(self.engine_project.get_arced_db()?, node_id)?;
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
    let path_to_storage_dir = PathBuf::from(temp_dir.path());
    let engine_project = ProjectForEngine::new(
        Project::new(
            Some("Test project".to_string()),
            Some("Test project description".to_string()),
            ProjectOwner::Myself,
        ),
        path_to_storage_dir.clone(),
    )
    .unwrap();
    let channel_for_engine = PiChannel::new();
    let main_channel = PiChannel::new();
    let pi_channel_tx = main_channel.tx.clone();
    let (fetcher_tx, _fetcher_rx) = tokio::sync::mpsc::channel::<PiEvent>(100);
    let mut engine = Engine::open_project(
        engine_project,
        channel_for_engine,
        pi_channel_tx,
        fetcher_tx,
    );
    engine.engine_project.load_db().unwrap();
    engine
}
