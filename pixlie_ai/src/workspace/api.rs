use super::{Workspace, WorkspaceCollection};
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
