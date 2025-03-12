use super::{EdgeLabel, Engine, NodeFlags};
use crate::engine::node::{NodeId, NodeItem, NodeLabel, Payload};
use crate::entity::content::TableRow;
use crate::entity::objective::Objective;
use crate::entity::search::SearchTerm;
use crate::entity::web::link::Link;
use crate::entity::workflow::WorkflowStep;
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

#[derive(Clone, Deserialize, Display, TS)]
#[ts(export)]
pub enum NodeWrite {
    Link(LinkWrite),
    SearchTerm(String),
    Objective(String),
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub enum EngineRequestPayload {
    GetLabels,
    GetNodesWithLabel(String),
    GetNodesWithIds(Vec<u32>),
    GetAllNodes,
    GetAllEdges,
    CreateNode(NodeWrite),

    // Some nodes allow a "query", which can generate any number of nodes, like a search
    Query(u32),
    // ToggleCrawl(u32),
}

#[derive(Clone, Default, Serialize, TS)]
#[ts(export)]
pub struct EngineResponseResults {
    pub nodes: Vec<APINodeItem>,
    pub labels: Vec<String>,
    #[ts(type = "{ [node_id: number]: Array<[number, string]> }")]
    pub edges: HashMap<NodeId, Vec<(NodeId, EdgeLabel)>>,
    #[ts(type = "{ [label: string]: Array<number> }")]
    pub node_ids_by_label: HashMap<String, Vec<u32>>,
}

#[derive(Clone, Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum EngineResponsePayload {
    Success,
    Results(EngineResponseResults),
    Error(String),
}

#[derive(Clone, Display, Deserialize, Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum APIPayload {
    // StepPrompt(String),
    Step(WorkflowStep),
    Link(Link),
    Text(String),
    Tree(String),
    TableRow(TableRow),
}

impl APIPayload {
    pub fn from_payload(payload: Payload) -> APIPayload {
        match payload {
            Payload::Step(step) => APIPayload::Step(step),
            Payload::Link(link) => APIPayload::Link(link),
            Payload::Text(text) => APIPayload::Text(text),
            // The empty string is garbage, just to keep the type system happy
            Payload::Tree => APIPayload::Tree("".to_string()),
            Payload::TableRow(table_row) => APIPayload::TableRow(table_row),
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
                payload: EngineRequestPayload::GetAllNodes,
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

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetAllEdges,
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

// pub async fn toggle_crawl(
//     path: web::Path<(String, u32)>,
//     api_state: web::Data<ApiState>,
// ) -> PiResult<impl Responder> {
//     let request_id = api_state.req_id.fetch_add(1);
//     let (project_id, node_id) = path.into_inner();
//     // Subscribe to the API channel, so we can receive the response
//     let mut rx = api_state.api_channel_tx.subscribe();
//
//     api_state.main_tx.send(PiEvent::APIRequest(
//         project_id.clone(),
//         EngineRequest {
//             request_id: request_id.clone(),
//             project_id: project_id.clone(),
//             payload: EngineRequestPayload::ToggleCrawl(node_id),
//         },
//     ))?;
//
//     debug!("Waiting for response for request {}", request_id);
//     let mut response_opt: Option<EngineResponsePayload> = None;
//     while response_opt.is_none() {
//         match rx.recv().await {
//             Ok(event) => match event {
//                 PiEvent::APIResponse(p_id, response) => {
//                     if p_id == project_id && response.request_id == request_id {
//                         response_opt = Some(response.payload.clone());
//                     }
//                 }
//                 _ => {}
//             },
//             Err(_err) => {}
//         }
//     }
//
//     debug!("Got response for request {}", request_id);
//     match response_opt {
//         Some(response) => Ok(web::Json(response)),
//         None => Ok(web::Json(EngineResponsePayload::Error(
//             "Could not get a response".to_string(),
//         ))),
//     }
// }

pub fn handle_engine_api_request(
    request: EngineRequest,
    engine: Arc<&Engine>,
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
) -> PiResult<()> {
    let response: EngineResponsePayload = match request.payload {
        EngineRequestPayload::GetLabels => {
            let labels = engine.get_all_node_labels();
            EngineResponsePayload::Results(EngineResponseResults {
                labels: labels.iter().map(|x| x.to_string()).collect(),
                ..Default::default()
            })
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
            EngineResponsePayload::Results(EngineResponseResults {
                node_ids_by_label: HashMap::from([(label, nodes.iter().map(|x| x.id).collect())]),
                nodes,
                ..Default::default()
            })
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

            EngineResponsePayload::Results(EngineResponseResults {
                nodes,
                ..Default::default()
            })
        }
        EngineRequestPayload::GetAllNodes => {
            let mut nodes: Vec<APINodeItem> = engine
                .get_all_nodes()
                .iter()
                .map(|node| APINodeItem {
                    id: node.id.clone(),
                    labels: node.labels.clone(),
                    payload: APIPayload::from_payload(node.payload.clone()),
                    flags: APINodeFlags::from_node_flags(node.flags.clone()),
                    written_at: node.written_at.clone(),
                })
                .collect();
            nodes.sort_by(|a, b| a.id.cmp(&b.id));

            EngineResponsePayload::Results(EngineResponseResults {
                nodes,
                ..Default::default()
            })
        }
        EngineRequestPayload::GetAllEdges => {
            let edges = engine.get_all_edges();
            EngineResponsePayload::Results(EngineResponseResults {
                edges: edges
                    .iter()
                    .map(|(k, v)| (**k, v.iter().map(|x| (*x.0, x.1.to_string())).collect()))
                    .collect(),
                ..Default::default()
            })
        }
        EngineRequestPayload::CreateNode(node_write) => {
            match node_write {
                NodeWrite::Link(link_write) => {
                    Link::add_manually(engine.clone(), &link_write.url)?;
                }
                NodeWrite::SearchTerm(text) => {
                    SearchTerm::add_manually(engine.clone(), &text)?;
                }
                NodeWrite::Objective(text) => {
                    Objective::add_manually(engine.clone(), &text)?;
                }
            }
            EngineResponsePayload::Success
        }
        EngineRequestPayload::Query(node_id) => match engine.get_node_by_id(&node_id) {
            Some(node) => {
                if node.labels.contains(&NodeLabel::SearchTerm) {
                    match &node.payload {
                        Payload::Text(_) => {
                            let mut results: Vec<NodeItem> =
                                SearchTerm::query(&node, engine.clone(), &node_id.into())?;
                            results.sort_by(|a, b| a.id.cmp(&b.id));

                            EngineResponsePayload::Results(EngineResponseResults {
                                nodes: results
                                    .iter()
                                    .map(|x| APINodeItem {
                                        id: x.id.clone(),
                                        labels: x.labels.clone(),
                                        payload: APIPayload::from_payload(x.payload.clone()),
                                        flags: APINodeFlags::from_node_flags(x.flags.clone()),
                                        written_at: x.written_at.clone(),
                                    })
                                    .collect::<Vec<APINodeItem>>(),
                                ..Default::default()
                            })
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
        // EngineRequestPayload::ToggleCrawl(node_id) => match engine.get_node_by_id(&node_id) {
        //     Some(node) => match node.payload {
        //         Payload::Domain(ref domain) => {
        //             engine.update_node(
        //                 &node_id,
        //                 Payload::Domain(Domain {
        //                     name: domain.name.clone(),
        //                     is_allowed_to_crawl: !domain.is_allowed_to_crawl,
        //                 }),
        //             )?;
        //             engine.toggle_flag(&node_id, NodeFlags::IS_BLOCKED)?;
        //             EngineResponsePayload::Success
        //         }
        //         _ => EngineResponsePayload::Error(format!("Node {} is not a domain", node_id)),
        //     },
        //     None => EngineResponsePayload::Error(format!("Node {} not found", node_id)),
        // },
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
