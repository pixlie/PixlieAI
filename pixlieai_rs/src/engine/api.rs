use super::{Engine, Node};
use crate::{error::PiResult, PiEvent};
use serde::{Deserialize, Serialize};

pub struct EngineRequestMessage {
    pub request_id: u32,
    pub payload: EngineRequest,
}

#[derive(Deserialize)]
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
    channel_tx: crossbeam_channel::Sender<PiEvent>,
) -> PiResult<()> {
    match request.payload {
        // EngineRequest::GetNodesWithLabel(label) => {
        //     let nodes = engine.nodes_by_label.read().unwrap().get(&label).unwrap();
        //     channel_tx.send(PiEvent::EngineResponse(EngineResponseMessage {
        //         request_id: request.request_id,
        //         payload: EngineApiResponse::NodesWithLabel(nodes),
        //     }))?;
        // }
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
