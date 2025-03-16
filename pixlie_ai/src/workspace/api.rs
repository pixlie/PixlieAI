use super::{APIProvider, Workspace, WorkspaceCollection, WorkspaceUpdate};
use crate::utils::crud::CrudItem;
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{web, Responder};
use std::collections::HashMap;

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
