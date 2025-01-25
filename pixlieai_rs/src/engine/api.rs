use super::{Engine, Node, Payload};
use crate::entity::web::Link;
use crate::{api::ApiState, error::PiResult};
use crate::{CommsChannel, PiEvent};
use actix_web::{web, Responder};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::Display;
use ts_rs::TS;

pub struct EngineRequestMessage {
    pub request_id: u32,
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
    pub nodes: Vec<Node>,
    pub labels: Vec<String>,
    pub nodes_by_label: HashMap<String, Vec<Node>>,
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

pub async fn get_labels(api_state: web::Data<ApiState>) -> PiResult<impl Responder> {
    debug!("Label request for get_labels");
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
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
    node_write: web::Json<NodeWrite>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    debug!(
        "Create node request for node with label: {}",
        node_write.to_string()
    );
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            payload: EngineRequest::CreateNode(node_write.into_inner()),
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
    engine: &mut Engine,
    api_ch: CommsChannel,
) -> PiResult<()> {
    debug!("Got an engine API request");
    let response: EngineApiResponse = match request.payload {
        EngineRequest::GetLabels => match engine.node_ids_by_label.read() {
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
        EngineRequest::GetNodesWithLabel(label) => match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => match node_ids_by_label.get(&label) {
                Some(node_ids) => {
                    let nodes: Vec<Node> = node_ids
                        .iter()
                        .filter_map(|node_id| match engine.nodes.get(node_id) {
                            Some(node) => match node.read() {
                                Ok(node) => Some(node.clone()),
                                Err(_) => None,
                            },
                            None => None,
                        })
                        .collect();

                    EngineApiResponse::Results(EngineApiData {
                        nodes_by_label: HashMap::from([(label, nodes)]),
                        ..Default::default()
                    })
                }
                None => EngineApiResponse::Error(format!("No node IDs found for label {}", label)),
            },
            Err(err) => {
                error!("Error reading nodes_by_label: {}", err);
                EngineApiResponse::Error(format!("Error reading nodes_by_label: {}", err))
            }
        },
        EngineRequest::CreateNode(node_write) => {
            match node_write {
                NodeWrite::Link(link_write) => {
                    engine.add_node(Payload::Link(Link {
                        url: link_write.url,
                        is_fetched: false,
                    }));
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
