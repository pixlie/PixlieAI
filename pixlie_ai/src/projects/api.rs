use std::path::PathBuf;

use super::{Project, ProjectCollection, ProjectCreate, ProjectForEngine, ProjectOwner};
use crate::{
    config::Settings,
    error::{PiError, PiResult},
    utils::crud::Crud,
};
use actix_web::{get, post, web, Responder, Scope};

#[get("")]
pub async fn read_projects() -> PiResult<impl Responder> {
    let projects = ProjectCollection::read_list()?;
    Ok(web::Json(projects))
}

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

pub fn api_projects_scope() -> Scope {
    web::scope("/projects")
        .service(read_projects)
        .service(create_project)
}
