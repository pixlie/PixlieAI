use super::{Project, ProjectCollection, ProjectCreate, ProjectOwner};
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{get, http::StatusCode, post, web, HttpResponse, HttpResponseBuilder, Responder};
use log::error;

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
pub async fn create_project(project: web::Json<ProjectCreate>) -> PiResult<HttpResponse> {
    let project = Project::new(
        project.name.clone(),
        project.description.clone(),
        ProjectOwner::Myself,
    );

    // When a project is created, we create a DB for it
    // If a DB is missing, we assume that the project was deleted
    let (_, path_to_db) = Project::get_default_path_to_db(&project.uuid)?;
    Project::create_project_db(&path_to_db)?;
    ProjectCollection::create(project.clone())?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(project))
}

pub fn configure_api_projects(app_config: &mut utoipa_actix_web::service_config::ServiceConfig) {
    app_config.service(
        utoipa_actix_web::scope::scope("/projects")
            .service(read_projects)
            .service(create_project),
    );
}
