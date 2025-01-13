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

#[derive(Serialize)]
pub enum EngineApiResponse {
    NodesWithLabel(Vec<Node>),
    RelatedNodes(Vec<Node>),
    PartNodes(Vec<Node>),
}

pub fn handle_engine_api_request(
    request: EngineRequestMessage,
    engine: &Engine,
    api_ch: CommsChannel,
) -> PiResult<()> {
    debug!("Got an engine API request");
    match request.payload {
        EngineRequest::GetNodesWithLabel(label) => match engine.nodes_by_label.read() {
            Ok(nodes_by_label) => {
                let nodes: Vec<Node> = match nodes_by_label.get(&label) {
                    Some(nodes) => nodes
                        .iter()
                        .filter_map(|node_id| match engine.nodes.get(node_id) {
                            Some(node) => match node.read() {
                                Ok(node) => Some(node.clone()),
                                Err(_) => None,
                            },
                            None => None,
                        })
                        .collect(),
                    None => vec![],
                };
                api_ch
                    .tx
                    .send(PiEvent::EngineResponse(EngineResponseMessage {
                        request_id: request.request_id,
                        payload: EngineApiResponse::NodesWithLabel(nodes),
                    }))?;
            }
            Err(err) => {
                error!("Error reading nodes_by_label: {}", err);
            }
        },
        // EngineRequest::GetRelatedNodes(node_id) => {
        //     let nodes = engine.get_related_nodes(&node_id);
        //     channel_tx.send(PiEvent::EngineResponse(EngineResponseMessage {
        //         request_id: request.request_id,
        //         payload: EngineApiResponse::RelatedNodes(nodes),
        //     }))?;
        // }
        // EngineRequest::GetPartNodes(node_id) => {
        //     let nodes = engine.get_part_nodes(&node_id);
        //     channel_tx.send(PiEvent::EngineResponse(EngineResponseMessage {
        //         request_id: request.request_id,
        //         payload: EngineApiResponse::PartNodes(nodes),
        //     }))?;
        // }
        _ => {}
    }
    Ok(())
}
