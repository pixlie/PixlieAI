use super::{Engine, NodeItem};
use crate::entity::web::link::Link;
use crate::PiEvent;
use crate::{api::ApiState, error::PiResult};
use actix_web::{web, Responder};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub enum EngineRequestPayload {
    GetLabels,
    GetNodesWithLabel(String),
    GetRelatedNodes(u32),
    GetPartNodes(u32),
    CreateNode(NodeWrite),
}

#[derive(Clone, Default, Serialize, TS)]
#[ts(export)]
pub struct EngineResponseResults {
    pub nodes: Vec<NodeItem>,
    pub labels: Vec<String>,
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

#[derive(Clone)]
pub struct EngineResponse {
    pub request_id: u32,
    pub payload: EngineResponsePayload,
}

#[derive(Deserialize)]
pub struct NodesByLabelParams {
    label: String,
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

pub async fn get_nodes_by_label(
    project_id: web::Path<String>,
    params: web::Query<NodesByLabelParams>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to get nodes with label {}",
        request_id, project_id, params.label
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetNodesWithLabel(params.label.clone()),
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

pub fn handle_engine_api_request(
    request: EngineRequest,
    engine: &Engine,
    pi_channel_main_tx: crossbeam_channel::Sender<PiEvent>,
) -> PiResult<()> {
    let response: EngineResponsePayload = match request.payload {
        EngineRequestPayload::GetLabels => match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => {
                let labels = node_ids_by_label.keys().cloned().collect();
                EngineResponsePayload::Results(EngineResponseResults {
                    labels,
                    ..Default::default()
                })
            }
            Err(err) => {
                error!("Error reading nodes_by_label: {}", err);
                EngineResponsePayload::Error(format!("Error reading nodes_by_label: {}", err))
            }
        },
        EngineRequestPayload::GetNodesWithLabel(label) => match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => match node_ids_by_label.get(&label) {
                Some(node_ids) => {
                    let nodes: Vec<NodeItem> = node_ids
                        .iter()
                        .filter_map(|node_id| match engine.nodes.read() {
                            Ok(nodes) => match nodes.get(node_id) {
                                Some(node) => match node.read() {
                                    Ok(node) => Some(node.clone()),
                                    Err(_err) => None,
                                },
                                None => None,
                            },
                            Err(_err) => None,
                        })
                        .collect();

                    EngineResponsePayload::Results(EngineResponseResults {
                        node_ids_by_label: HashMap::from([(
                            label,
                            nodes.iter().map(|x| *x.id).collect(),
                        )]),
                        nodes,
                        ..Default::default()
                    })
                }
                None => {
                    EngineResponsePayload::Error(format!("No node IDs found for label {}", label))
                }
            },
            Err(err) => {
                error!("Error reading nodes_by_label: {}", err);
                EngineResponsePayload::Error(format!("Error reading nodes_by_label: {}", err))
            }
        },
        EngineRequestPayload::CreateNode(node_write) => {
            match node_write {
                NodeWrite::Link(link_write) => {
                    Link::add(&engine, &link_write.url)?;
                }
            }

            EngineResponsePayload::Success
        }
        _ => EngineResponsePayload::Error("Could not understand request".to_string()),
    };

    pi_channel_main_tx.send(PiEvent::APIResponse(
        request.project_id,
        EngineResponse {
            request_id: request.request_id,
            payload: response,
        },
    ))?;

    Ok(())
}
