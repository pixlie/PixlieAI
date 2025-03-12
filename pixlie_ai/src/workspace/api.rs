use super::{Workspace, WorkspaceCollection, WorkspaceUpdate};
use crate::utils::crud::CrudItem;
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{web, Responder};

pub async fn read_default_workspace() -> PiResult<impl Responder> {
    let item = WorkspaceCollection::get_default()?;
    Ok(web::Json(item))
}

pub async fn update_workspace(
    workspace_id: web::Path<String>,
    update: web::Json<WorkspaceUpdate>,
) -> PiResult<impl Responder> {
    let item = WorkspaceCollection::read_item(&workspace_id)?;
    let item_id = item.get_id();
    WorkspaceCollection::update(
        &item.get_id(),
        Workspace {
            anthropic_api_key: update.anthropic_api_key.clone(),
            ..item
        },
    )?;
    Ok(web::Json(item_id))
}
