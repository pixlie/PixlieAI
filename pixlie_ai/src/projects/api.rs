use std::path::PathBuf;

use super::{Project, ProjectCollection, ProjectCreate, ProjectForEngine, ProjectOwner};
use crate::{
    config::Settings,
    error::{PiError, PiResult},
    utils::crud::Crud,
};
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
    let settings: Settings = Settings::get_cli_settings()?;
    let path_to_storage_dir = match settings.path_to_storage_dir {
        Some(path) => PathBuf::from(path),
        None => {
            return Err(PiError::InternalError(
                "Cannot find path to storage directory".to_string(),
            ));
        }
    };
    let engine_project = match ProjectForEngine::new(
        Project::new(
            project.name.clone(),
            project.description.clone(),
            ProjectOwner::Myself,
        ),
        path_to_storage_dir,
    ) {
        Ok(engine_project) => {
            // TODO: Ideally this should be called in ProjectForEngine::new,
            // but tests are failing if it is called there.
            // We can move this call there once Settings is refactored to be
            // cleanly testable and functions dependent on Settings are
            // moved to impl Settings
            ProjectCollection::create(engine_project.project.clone())?;
            engine_project
        }
        Err(err) => {
            return Err(PiError::InternalError(format!(
                "Cannot create project: {}",
                err
            )));
        }
    };

    Ok(web::Json(engine_project.project))
}

pub fn configure_api_projects(app_config: &mut utoipa_actix_web::service_config::ServiceConfig) {
    app_config.service(
        utoipa_actix_web::scope::scope("/projects")
            .service(read_projects)
            .service(create_project),
    );
}
