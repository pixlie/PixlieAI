use super::{Project, ProjectCollection, ProjectCreate, ProjectOwner};
use crate::{error::PiResult, utils::crud::Crud};
use actix_web::{get, post, web};
use log::{error, info};

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
pub async fn read_projects() -> PiResult<web::Json<Vec<Project>>> {
    let projects = ProjectCollection::read_list()?;
    info!(
        "Projects {}",
        projects
            .iter()
            .map(|project| project.uuid.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    // Check that DB exists for each project
    projects
        .iter()
        .for_each(|project| match Project::check_project_db(&project.uuid) {
            Ok(_) => {}
            Err(_) => {
                error!("DB for project {} does not exist", &project.uuid);
            }
        });
    Ok(web::Json(ProjectCollection::read_list()?))
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
pub async fn create_project(project: web::Json<ProjectCreate>) -> PiResult<web::Json<Project>> {
    let project = Project::new(
        project.name.clone(),
        project.description.clone(),
        ProjectOwner::Myself,
    );

    // When a project is created, we create a DB for it
    // If a DB is missing, we assume that the project was deleted
    let (_, path_to_db) = Project::get_default_path_to_db(&project.uuid)?;
    ProjectCollection::create(project.clone())?;
    Project::create_project_db(&path_to_db)?;
    Ok(web::Json(project))
}

pub fn configure_api_projects(app_config: &mut utoipa_actix_web::service_config::ServiceConfig) {
    app_config.service(
        utoipa_actix_web::scope::scope("/projects")
            .service(read_projects)
            .service(create_project),
    );
}
