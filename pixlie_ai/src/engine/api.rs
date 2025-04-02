use super::{EdgeLabel, Engine, NodeFlags};
use crate::engine::node::{NodeId, NodeItem, NodeLabel, Payload};
use crate::entity::content::TableRow;
use crate::entity::project_settings::ProjectSettings;
use crate::entity::search::saved_search::SavedSearch;
use crate::entity::web::link::Link;
use crate::error::PiError;
use crate::PiEvent;
use crate::{api::ApiState, error::PiResult};
use actix_web::{web, Responder};
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use strum::Display;
use ts_rs::TS;

#[derive(Clone)]
pub struct EngineRequest {
    pub request_id: u32,
    pub project_id: String,
    pub payload: EngineRequestPayload,
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub struct LinkWrite {
    pub url: String,
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub struct ProjectSettingsWrite {
    pub has_user_specified_starting_links: bool,
}

#[derive(Clone, Deserialize, Display, TS)]
#[ts(export)]
pub enum NodeWrite {
    Link(LinkWrite),
    SearchTerm(String),
    Objective(String),
    ProjectSettings(ProjectSettingsWrite),
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub struct EdgeWrite {
    node_ids: (NodeId, NodeId),
    edge_labels: (EdgeLabel, EdgeLabel),
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub enum EngineRequestPayload {
    GetLabels,
    GetNodesWithLabel(String),
    GetNodesWithIds(Vec<u32>),
    GetAllNodes(Option<DateTime<Utc>>),
    GetAllEdges(Option<DateTime<Utc>>),

    CreateNode(NodeWrite),
    CreateEdge(EdgeWrite),

    // Some nodes allow a "query", which can generate any number of nodes, like a search
    Query(u32),
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct APINodeEdges {
    #[ts(type = "Array<[number, string]>")]
    pub edges: Vec<(NodeId, String)>,
    pub written_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct APIEdges(HashMap<NodeId, APINodeEdges>);

#[derive(Clone, Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum EngineResponsePayload {
    Success,
    Nodes(Vec<APINodeItem>),
    Labels(Vec<String>),
    Edges(APIEdges),
    Error(String),
}

#[derive(Clone, Display, Deserialize, Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum APIPayload {
    Link(Link),
    Text(String),
    Tree(String),
    TableRow(TableRow),
    ProjectSettings(ProjectSettings),
}

impl APIPayload {
    pub fn from_payload(payload: Payload) -> APIPayload {
        match payload {
            Payload::Link(link) => APIPayload::Link(link),
            Payload::Text(text) => APIPayload::Text(text),
            // The empty string is garbage, just to keep the type system happy
            Payload::Tree => APIPayload::Tree("".to_string()),
            Payload::TableRow(table_row) => APIPayload::TableRow(table_row),
            Payload::ProjectSettings(project_settings) => {
                APIPayload::ProjectSettings(project_settings)
            }
        }
    }
}

#[derive(Clone, Default, Serialize, TS)]
#[ts(export)]
pub enum APINodeFlags {
    #[default]
    None, // Not used
    IsProcessed,
    IsRequesting,
    IsBlocked,
}

impl APINodeFlags {
    pub fn from_node_flags(flags: NodeFlags) -> Vec<APINodeFlags> {
        let mut api_flags = vec![];
        if flags.contains(NodeFlags::IS_PROCESSED) {
            api_flags.push(APINodeFlags::IsProcessed);
        }
        if flags.contains(NodeFlags::IS_REQUESTING) {
            api_flags.push(APINodeFlags::IsRequesting);
        }
        if flags.contains(NodeFlags::IS_BLOCKED) {
            api_flags.push(APINodeFlags::IsBlocked);
        }
        api_flags
    }
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct APINodeItem {
    pub id: NodeId,
    pub labels: Vec<NodeLabel>, // A node can have multiple labels, like tags, indexed by relevance
    pub payload: APIPayload,

    #[serde(skip_deserializing)]
    pub flags: Vec<APINodeFlags>,
    pub written_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EngineResponse {
    pub request_id: u32,
    pub payload: EngineResponsePayload,
}

#[derive(Deserialize)]
pub struct QueryNodes {
    label: Option<String>,
    ids: Option<String>,
    since: Option<i64>,
}

#[derive(Deserialize)]
pub struct QueryEdges {
    since: Option<i64>,
}

pub async fn get_labels(
    project_id: web::Path<String>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to get all labels",
        request_id, project_id
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetLabels,
        },
    ))?;

    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn get_nodes(
    project_id: web::Path<String>,
    params: web::Query<QueryNodes>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();
    if let Some(label) = &params.label {
        debug!(
            "API request {} for project {} to get nodes with label {}",
            request_id, project_id, label
        );

        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetNodesWithLabel(label.clone()),
            },
        ))?;
    } else if let Some(ids) = &params.ids {
        let u32_ids: Vec<u32> = ids
            .split(",")
            .map(|id| id.parse::<u32>().unwrap())
            .collect();
        if u32_ids.len() == 0 {
            return Err(PiError::InternalError(format!(
                "No IDs provided for API request {} for project {}",
                request_id, project_id
            )));
        }
        debug!(
            "API request {} for project {} to get nodes with ids {:?}",
            request_id,
            project_id,
            ids.split(",").collect::<Vec<&str>>()
        );

        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetNodesWithIds(u32_ids),
            },
        ))?;
    } else if let Some(since) = &params.since {
        // Read the nodes written since the given timestamp
        debug!(
            "API request {} for project {} to get nodes since {}",
            request_id, project_id, since
        );
        let since = DateTime::from_timestamp_millis(*since);
        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetAllNodes(since),
            },
        ))?;
    } else {
        debug!(
            "API request {} for project {} to get all nodes",
            request_id, project_id
        );

        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetAllNodes(None),
            },
        ))?;
    }

    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn get_edges(
    project_id: web::Path<String>,
    params: web::Query<QueryEdges>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to get all edges",
        request_id, project_id
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    let since = if let Some(since) = params.since {
        DateTime::<Utc>::from_timestamp_millis(since)
    } else {
        None
    };
    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetAllEdges(since),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn create_node(
    project_id: web::Path<String>,
    node: web::Json<NodeWrite>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to create node with label {}",
        request_id,
        project_id,
        node.to_string()
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::CreateNode(node.into_inner()),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn create_edge(
    project_id: web::Path<String>,
    edge: web::Json<EdgeWrite>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to create edge between {} and {} with labels {} and {}",
        request_id,
        project_id,
        &edge.node_ids.0,
        &edge.node_ids.1,
        &edge.edge_labels.0,
        &edge.edge_labels.1,
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::CreateEdge(edge.into_inner()),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn search_results(
    path: web::Path<(String, u32)>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let (project_id, node_id) = path.into_inner();
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::Query(node_id),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub fn handle_engine_api_request(
    request: EngineRequest,
    engine: Arc<&Engine>,
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
) -> PiResult<()> {
    let response: EngineResponsePayload = match request.payload {
        EngineRequestPayload::GetLabels => {
            let labels = engine.get_all_node_labels();
            EngineResponsePayload::Labels(labels.iter().map(|x| x.to_string()).collect())
        }
        EngineRequestPayload::GetNodesWithLabel(label) => {
            let node_ids_with_label = engine.get_node_ids_with_label(&NodeLabel::from_str(&label)?);
            let mut nodes: Vec<APINodeItem> = node_ids_with_label
                .iter()
                .filter_map(|node_id| match engine.get_node_by_id(node_id) {
                    Some(arced_node) => Some(APINodeItem {
                        id: **node_id,
                        labels: arced_node.labels.clone(),
                        payload: APIPayload::from_payload(arced_node.payload.clone()),
                        flags: APINodeFlags::from_node_flags(arced_node.flags.clone()),
                        written_at: arced_node.written_at.clone(),
                    }),
                    None => None,
                })
                .collect();
            nodes.sort_by(|a, b| a.id.cmp(&b.id));
            EngineResponsePayload::Nodes(nodes)
        }
        EngineRequestPayload::GetNodesWithIds(node_ids) => {
            let mut nodes: Vec<APINodeItem> = vec![];
            for node_id in node_ids {
                if let Some(node) = engine.get_node_by_id(&node_id) {
                    nodes.push(APINodeItem {
                        id: node.id.clone(),
                        labels: node.labels.clone(),
                        payload: APIPayload::from_payload(node.payload.clone()),
                        flags: APINodeFlags::from_node_flags(node.flags.clone()),
                        written_at: node.written_at.clone(),
                    });
                }
            }
            nodes.sort_by(|a, b| a.id.cmp(&b.id));
            EngineResponsePayload::Nodes(nodes)
        }
        EngineRequestPayload::GetAllNodes(since) => {
            let mut nodes: Vec<APINodeItem> = match since {
                Some(since) => engine
                    .get_all_nodes()
                    .iter()
                    .filter_map(|node| {
                        if node.written_at > since {
                            Some(APINodeItem {
                                id: node.id.clone(),
                                labels: node.labels.clone(),
                                payload: APIPayload::from_payload(node.payload.clone()),
                                flags: APINodeFlags::from_node_flags(node.flags.clone()),
                                written_at: node.written_at.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),

                None => engine
                    .get_all_nodes()
                    .iter()
                    .map(|node| APINodeItem {
                        id: node.id.clone(),
                        labels: node.labels.clone(),
                        payload: APIPayload::from_payload(node.payload.clone()),
                        flags: APINodeFlags::from_node_flags(node.flags.clone()),
                        written_at: node.written_at.clone(),
                    })
                    .collect(),
            };
            nodes.sort_by(|a, b| a.id.cmp(&b.id));

            EngineResponsePayload::Nodes(nodes)
        }
        EngineRequestPayload::GetAllEdges(since) => {
            let edges: HashMap<NodeId, APINodeEdges> = match since {
                Some(since) => engine
                    .get_all_edges()
                    .iter()
                    .filter_map(|(node_id, node_edges)| {
                        if node_edges.written_at > since {
                            Some((
                                **node_id,
                                APINodeEdges {
                                    edges: node_edges
                                        .edges
                                        .iter()
                                        .map(|x| (x.0, x.1.to_string()))
                                        .collect(),
                                    written_at: node_edges.written_at.clone(),
                                },
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
                None => engine
                    .get_all_edges()
                    .iter()
                    .map(|(node_id, node_edges)| {
                        (
                            **node_id,
                            APINodeEdges {
                                edges: node_edges
                                    .edges
                                    .iter()
                                    .map(|x| (x.0, x.1.to_string()))
                                    .collect(),
                                written_at: node_edges.written_at.clone(),
                            },
                        )
                    })
                    .collect(),
            };

            EngineResponsePayload::Edges(APIEdges(edges))
        }
        EngineRequestPayload::CreateNode(node_write) => {
            match node_write {
                NodeWrite::Link(link_write) => {
                    Link::add(
                        engine.clone(),
                        &link_write.url,
                        vec![NodeLabel::AddedByUser, NodeLabel::Link],
                        vec![],
                        true,
                    )?;
                }
                NodeWrite::SearchTerm(text) => {
                    engine.get_or_add_node(
                        Payload::Text(text.to_string()),
                        vec![NodeLabel::AddedByUser, NodeLabel::SearchTerm],
                        true,
                        None,
                    )?;
                }
                NodeWrite::Objective(text) => {
                    engine.get_or_add_node(
                        Payload::Text(text.to_string()),
                        vec![NodeLabel::AddedByUser, NodeLabel::Objective],
                        true,
                        None,
                    )?;
                }
                NodeWrite::ProjectSettings(project_settings_write) => {
                    engine.get_or_add_node(
                        Payload::ProjectSettings(ProjectSettings {
                            has_user_specified_starting_links: project_settings_write
                                .has_user_specified_starting_links,
                        }),
                        vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
                        true,
                        None,
                    )?;
                }
            }
            EngineResponsePayload::Success
        }
        EngineRequestPayload::CreateEdge(edge_write) => {
            engine.add_connection(edge_write.node_ids, edge_write.edge_labels)?;
            EngineResponsePayload::Success
        }
        EngineRequestPayload::Query(node_id) => match engine.get_node_by_id(&node_id) {
            Some(node) => {
                if node.labels.contains(&NodeLabel::SearchTerm) {
                    match &node.payload {
                        Payload::Text(_) => {
                            let mut results: Vec<NodeItem> =
                                SavedSearch::query(&node, engine.clone(), &node_id.into())?;
                            results.sort_by(|a, b| a.id.cmp(&b.id));

                            EngineResponsePayload::Nodes(
                                results
                                    .iter()
                                    .map(|x| APINodeItem {
                                        id: x.id.clone(),
                                        labels: x.labels.clone(),
                                        payload: APIPayload::from_payload(x.payload.clone()),
                                        flags: APINodeFlags::from_node_flags(x.flags.clone()),
                                        written_at: x.written_at.clone(),
                                    })
                                    .collect::<Vec<APINodeItem>>(),
                            )
                        }
                        _ => EngineResponsePayload::Error(format!(
                            "Query only works on search terms, not on {}",
                            node.payload.to_string()
                        )),
                    }
                } else {
                    EngineResponsePayload::Error(format!(
                        "Query only works on search terms, not on {}",
                        node.payload.to_string()
                    ))
                }
            }
            None => EngineResponsePayload::Error(format!("Node {} not found", node_id)),
        },
    };

    main_channel_tx.send(PiEvent::APIResponse(
        request.project_id,
        EngineResponse {
            request_id: request.request_id,
            payload: response,
        },
    ))?;

    Ok(())
}
