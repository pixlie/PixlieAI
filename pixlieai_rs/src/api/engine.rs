use super::ApiState;
use crate::{
    engine::api::{EngineApiResponse, EngineRequest, EngineRequestMessage},
    PiEvent,
};
use actix_web::{web, Responder, Result};
use log::debug;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NodesByLabelParams {
    label: String,
}

pub async fn get_labels(api_state: web::Data<ApiState>) -> Result<impl Responder> {
    debug!("Label request for get_labels");
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            payload: EngineRequest::GetLabels,
        }))
        .unwrap();

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
    .await
    .unwrap();
    debug!("Got response for request {}", request_id);

    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineApiResponse::Error(format!(
            "Could not get a response"
        )))),
    }
}

pub async fn get_nodes_by_label(
    params: web::Query<NodesByLabelParams>,
    api_state: web::Data<ApiState>,
) -> Result<impl Responder> {
    debug!("Label request for get_nodes_by_label: {}", params.label);
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .engine_ch
        .tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            payload: EngineRequest::GetNodesWithLabel(params.label.clone()),
        }))
        .unwrap();

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
    .await
    .unwrap();
    debug!("Got response for request {}", request_id);

    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineApiResponse::Error(format!(
            "Could not get a response"
        )))),
    }
}
