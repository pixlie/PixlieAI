use super::{APIProvider, Workspace, WorkspaceCollection, WorkspaceUpdate};
use crate::utils::crud::CrudItem;
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{get, put, web, Responder};
use std::collections::HashMap;

#[utoipa::path(
    path = "/workspace",
    responses(
        (status = 200, description = "Workspace(default) retrieved successfully", body = Workspace),
        (status = 500, description = "Internal server error"),
    ),
    tag = "workspace",
)]
#[get("")]
pub async fn read_default_workspace() -> PiResult<impl Responder> {
    let mut item = WorkspaceCollection::get_default()?;
    // Keep only first 10 characters of the API keys
    item.api_keys.iter_mut().for_each(|(_, api_key)| {
        if api_key.len() > 10 {
            *api_key = format!("{}******", api_key[..10].to_string());
        }
    });
    Ok(web::Json(item))
}

/// Update the workspace
#[utoipa::path(
    path = "/workspace/{workspace_id}",
    request_body = WorkspaceUpdate,
    responses(
        (status = 200, description = "Workspace updated successfully", body = String),
        (status = 500, description = "Internal server error"),
    ),
    tag = "workspace",
)]
#[put("{workspace_id}")]
pub async fn update_workspace(
    workspace_id: web::Path<String>,
    update: web::Json<WorkspaceUpdate>,
) -> PiResult<impl Responder> {
    let item = WorkspaceCollection::read_item(&workspace_id)?;
    let item_id = item.get_id();
    let mut api_keys: HashMap<APIProvider, String> = item.api_keys.clone();
    if let Some(anthropic_api_key) = &update.anthropic_api_key {
        api_keys.insert(APIProvider::Anthropic, anthropic_api_key.clone());
    }
    if let Some(brave_search_api_key) = &update.brave_search_api_key {
        api_keys.insert(APIProvider::BraveSearch, brave_search_api_key.clone());
    }
    WorkspaceCollection::update(&item.get_id(), Workspace { api_keys, ..item })?;
    Ok(web::Json(item_id))
}

pub fn configure_api_workspace(app_config: &mut utoipa_actix_web::service_config::ServiceConfig) {
    app_config.service(
        utoipa_actix_web::scope::scope("/workspace")
            .service(read_default_workspace)
            .service(update_workspace),
    );
}
