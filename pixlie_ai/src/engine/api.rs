use super::{CommonEdgeLabels, CommonNodeLabels, NodeItem, Payload};
use crate::engine::LockedEngine;
use crate::entity::web::{Domain, Link};
use crate::{api::ApiState, error::PiResult};
use crate::{CommsChannel, PiEvent};
use actix_web::{web, Responder};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::Display;
use ts_rs::TS;
use url::Url;

pub struct EngineRequestMessage {
    pub request_id: u32,
    pub project_id: String,
    pub payload: EngineRequest,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct LinkWrite {
    pub url: String,
}

#[derive(Deserialize, Display, TS)]
#[ts(export)]
pub enum NodeWrite {
    Link(LinkWrite),
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub enum EngineRequest {
    GetLabels,
    GetNodesWithLabel(String),
    GetRelatedNodes(u32),
    GetPartNodes(u32),
    CreateNode(NodeWrite),
}

#[derive(Default, Serialize, TS)]
#[ts(export)]
pub struct EngineApiData {
    pub nodes: Vec<NodeItem>,
    pub labels: Vec<String>,
    #[ts(type = "{ [label: string]: Array<number> }")]
    pub node_ids_by_label: HashMap<String, Vec<u32>>,
}

#[derive(Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum EngineApiResponse {
    Success,
    Results(EngineApiData),
    Error(String),
}

pub struct EngineResponseMessage {
    pub request_id: u32,
    pub payload: EngineApiResponse,
}

#[derive(Deserialize)]
pub struct NodesByLabelParams {
    label: String,
}

pub async fn get_labels(
    project_id: web::Path<String>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    debug!("Label request for get_labels");
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            project_id: project_id.into_inner(),
            payload: EngineRequest::GetLabels,
        }))?;

    debug!("Waiting for response for request {}", request_id);
    let response_opt: Option<EngineApiResponse> = web::block(move || {
        api_state.api_ch.rx.iter().find_map(|event| match event {
            PiEvent::EngineResponse(response) => {
                if response.request_id == request_id {
                    Some(response.payload)
                } else {
                    None
                }
            }
            _ => None,
        })
    })
    .await?;
    debug!("Got response for request {}", request_id);

    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineApiResponse::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn get_nodes_by_label(
    project_id: web::Path<String>,
    params: web::Query<NodesByLabelParams>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    debug!("Label request for get_nodes_by_label: {}", params.label);
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            project_id: project_id.into_inner(),
            payload: EngineRequest::GetNodesWithLabel(params.label.clone()),
        }))?;

    debug!("Waiting for response for request {}", request_id);
    let response_opt: Option<EngineApiResponse> = web::block(move || {
        api_state.api_ch.rx.iter().find_map(|event| match event {
            PiEvent::EngineResponse(response) => {
                if response.request_id == request_id {
                    Some(response.payload)
                } else {
                    None
                }
            }
            _ => None,
        })
    })
    .await?;
    debug!("Got response for request {}", request_id);

    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineApiResponse::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub async fn create_node(
    project_id: web::Path<String>,
    node: web::Json<NodeWrite>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    debug!(
        "Create node request for node with label: {}",
        node.to_string()
    );
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            project_id: project_id.into_inner(),
            payload: EngineRequest::CreateNode(node.into_inner()),
        }))?;

    debug!("Waiting for response for request {}", request_id);
    let response_opt: Option<EngineApiResponse> = web::block(move || {
        api_state.api_ch.rx.iter().find_map(|event| match event {
            PiEvent::EngineResponse(response) => {
                if response.request_id == request_id {
                    Some(response.payload)
                } else {
                    None
                }
            }
            _ => None,
        })
    })
    .await?;
    debug!("Got response for request {}", request_id);

    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineApiResponse::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

pub fn handle_engine_api_request(
    request: EngineRequestMessage,
    engine: &LockedEngine,
    api_ch: CommsChannel,
) -> PiResult<()> {
    debug!("Got an engine API request");
    let response: EngineApiResponse = match request.payload {
        EngineRequest::GetLabels => match engine.read() {
            Ok(engine) => match engine.node_ids_by_label.read() {
                Ok(node_ids_by_label) => {
                    let labels = node_ids_by_label.keys().cloned().collect();
                    EngineApiResponse::Results(EngineApiData {
                        labels,
                        ..Default::default()
                    })
                }
                Err(err) => {
                    error!("Error reading nodes_by_label: {}", err);
                    EngineApiResponse::Error(format!("Error reading nodes_by_label: {}", err))
                }
            },
            Err(_err) => EngineApiResponse::Error("Could not read engine".to_string()),
        },
        EngineRequest::GetNodesWithLabel(label) => match engine.read() {
            Ok(engine) => match engine.node_ids_by_label.read() {
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

                        EngineApiResponse::Results(EngineApiData {
                            node_ids_by_label: HashMap::from([(
                                label,
                                nodes.iter().map(|x| *x.id).collect(),
                            )]),
                            nodes,
                            ..Default::default()
                        })
                    }
                    None => {
                        EngineApiResponse::Error(format!("No node IDs found for label {}", label))
                    }
                },
                Err(err) => {
                    error!("Error reading nodes_by_label: {}", err);
                    EngineApiResponse::Error(format!("Error reading nodes_by_label: {}", err))
                }
            },
            Err(_err) => EngineApiResponse::Error("Could not read engine".to_string()),
        },
        EngineRequest::CreateNode(node_write) => {
            match node_write {
                NodeWrite::Link(link_write) => {
                    Link::add(&link_write.url, &engine)?;
                }
            }

            EngineApiResponse::Success
        }
        _ => EngineApiResponse::Error("Could not understand request".to_string()),
    };

    api_ch
        .tx
        .send(PiEvent::EngineResponse(EngineResponseMessage {
            request_id: request.request_id,
            payload: response,
        }))?;

    Ok(())
}
