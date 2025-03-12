use super::{Project, ProjectCollection, ProjectCreate, ProjectOwner};
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{web, Responder};
use uuid::Uuid;

pub async fn read_projects() -> PiResult<impl Responder> {
    let projects = ProjectCollection::read_list()?;
    Ok(web::Json(projects))
}

pub async fn create_project(project: web::Json<ProjectCreate>) -> PiResult<impl Responder> {
    let project: Project = Project {
        uuid: Uuid::new_v4().to_string(),
        name: project.name.clone(),
        description: project.description.clone(),
        owner: ProjectOwner::Myself,
    };
    let project = ProjectCollection::create(project)?;
    Ok(web::Json(project))
}
