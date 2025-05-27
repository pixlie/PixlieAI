use super::{Project, ProjectCollection, ProjectCreate, ProjectOwner};
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{get, post, web, Responder};

/// Get a list of all projects
#[utoipa::path(
    path = "/projects",
    responses(
        (status = 200, description = "Projects retrieved successfully", body = Vec<Project>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "projects",
)]
#[get("")]
pub async fn read_projects() -> PiResult<impl Responder> {
    let projects = ProjectCollection::read_list()?;
    Ok(web::Json(projects))
}

/// Create a new project
#[utoipa::path(
    path = "/projects",
    request_body = ProjectCreate,
    responses(
        (status = 200, description = "Project created successfully", body = Project),
        (status = 500, description = "Internal server error"),
    ),
    tag = "projects",
)]
#[post("")]
pub async fn create_project(project: web::Json<ProjectCreate>) -> PiResult<impl Responder> {
    let project = Project::new(
        project.name.clone(),
        project.description.clone(),
        ProjectOwner::Myself,
    );
    ProjectCollection::create(project.clone())?;
    Ok(web::Json(project))
}

pub fn configure_api_projects(app_config: &mut utoipa_actix_web::service_config::ServiceConfig) {
    app_config.service(
        utoipa_actix_web::scope::scope("/projects")
            .service(read_projects)
            .service(create_project),
    );
}
