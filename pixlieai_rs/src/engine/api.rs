use super::{Engine, Node};
use crate::{error::PiResult, CommsChannel, PiEvent};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub struct EngineRequestMessage {
    pub request_id: u32,
    pub payload: EngineRequest,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub enum EngineRequest {
    GetNodesWithLabel(String),
    GetRelatedNodes(u32),
    GetPartNodes(u32),
}

pub struct EngineResponseMessage {
    pub request_id: u32,
    pub payload: EngineApiResponse,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub enum EngineApiQueryType {
    NodeIdsByLabel(String, Vec<u32>),
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct EngineApiData {
    pub nodes: Vec<Node>,
    pub query_type: EngineApiQueryType,
}

#[derive(Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum EngineApiResponse {
    Results(EngineApiData),
    Error(String),
}

pub fn handle_engine_api_request(
    request: EngineRequestMessage,
    engine: &Engine,
    api_ch: CommsChannel,
) -> PiResult<()> {
    debug!("Got an engine API request");
    let response: EngineApiResponse = match request.payload {
        EngineRequest::GetNodesWithLabel(label) => match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => match node_ids_by_label.get(&label) {
                Some(node_ids) => {
                    let nodes = node_ids
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
                        nodes,
                        query_type: EngineApiQueryType::NodeIdsByLabel(
                            label,
                            node_ids.iter().map(|x| **x).collect(),
                        ),
                    })
                }
                None => EngineApiResponse::Error(format!("No node IDs found for label {}", label)),
            },
            Err(err) => {
                error!("Error reading nodes_by_label: {}", err);
                EngineApiResponse::Error(format!("Error reading nodes_by_label: {}", err))
            }
        },
        _ => EngineApiResponse::Error(format!("Could not understand request")),
    };

    api_ch
        .tx
        .send(PiEvent::EngineResponse(EngineResponseMessage {
            request_id: request.request_id,
            payload: response,
        }))?;

    Ok(())
}
