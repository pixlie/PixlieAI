use super::ApiState;
use crate::{
    engine::api::{EngineApiResponse, EngineRequest, EngineRequestMessage},
    PiEvent,
};
use actix_web::{web, Responder, Result};

pub async fn engine_api(
    engine_request: web::Json<EngineRequest>,
    api_state: web::Data<ApiState>,
) -> Result<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    api_state
        .channel_tx
        .send(PiEvent::EngineRequest(EngineRequestMessage {
            request_id: request_id.clone(),
            payload: engine_request.into_inner(),
        }))
        .unwrap();

    let response: Option<EngineApiResponse> = web::block(move || {
        api_state.channel_rx.iter().find_map(|event| match event {
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

    Ok(web::Json(response))
}
