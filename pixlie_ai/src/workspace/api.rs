use super::{Workspace, WorkspaceCollection, WorkspaceUpdate};
use crate::utils::crud::CrudItem;
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{web, Responder};

pub async fn read_default_workspace() -> PiResult<impl Responder> {
    let items = WorkspaceCollection::read_list()?;
    let item = if items.len() == 0 {
        let item: Workspace = Workspace {
            uuid: uuid::Uuid::new_v4().to_string(),
            name: "Default".to_string(),
            description: None,
            anthropic_api_key: None,
        };
        WorkspaceCollection::create(item)?
    } else {
        items[0].clone()
    };
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
