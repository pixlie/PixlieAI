use crate::{
    config::Settings,
    error::{PiError, PiResult},
    utils::crud::{Crud, CrudItem},
};
use log::error;
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;
use utoipa::ToSchema;

pub mod api;

#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub enum ProjectOwner {
    Myself,
    User(String),
    Organization(String),
}

#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct Project {
    /// Project ID (UUID)
    pub uuid: String,
    /// Project name - this is assigned by AI
    pub name: Option<String>,
    /// Project description - currently unused
    pub description: Option<String>,
    /// Project owner - for future use, currently defaults to Myself
    pub owner: ProjectOwner,
}

#[derive(Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ProjectCreate {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Project {
    pub fn new(name: Option<String>, description: Option<String>, owner: ProjectOwner) -> Project {
        let project = Project {
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            owner,
        };
        project
    }
}

impl CrudItem for Project {
    fn get_id(&self) -> String {
        self.uuid.clone()
    }
}

pub struct ProjectCollection {}

impl Crud for ProjectCollection {
    type Item = Project;

    fn get_collection_name() -> &'static str {
        "project"
    }
}

impl Project {
    pub fn create_project_db(path_to_db: &PathBuf) -> PiResult<()> {
        if path_to_db.exists() || path_to_db.is_dir() {
            return Err(PiError::InternalError(
                "DB for project already exists".to_string(),
            ));
        }
        match DB::open_default(path_to_db.as_os_str()) {
            Ok(db) => {
                // db.put("__init__", "1".as_bytes())?;
                // db.flush()?;
                Ok(())
            }
            Err(error) => Err(PiError::InternalError(format!(
                "Cannot create DB for project: {}",
                error
            ))),
        }
    }

    pub fn get_default_path_to_db(project_uuid: &str) -> PiResult<(PathBuf, PathBuf)> {
        let path_to_storage_dir: PathBuf = match Settings::get_cli_settings() {
            Ok(settings) => match settings.path_to_storage_dir {
                Some(path) => PathBuf::from(path),
                None => {
                    error!("Cannot find path to storage directory");
                    return Err(PiError::InternalError(
                        "Path to storage directory not configured yet".to_string(),
                    ));
                }
            },
            Err(err) => {
                error!("Error reading settings: {}", err);
                return Err(PiError::InternalError("Error reading settings".to_string()));
            }
        };
        let path_to_db = path_to_storage_dir.join(format!("{}.rocksdb", project_uuid));
        Ok((path_to_storage_dir, path_to_db))
    }

    pub fn check_project_db(project_uuid: &str) -> PiResult<(PathBuf, PathBuf)> {
        let (path_to_storage_dir, path_to_db) = Self::get_default_path_to_db(project_uuid)?;
        let _ = ProjectCollection::read_item(&project_uuid)?;
        if !path_to_db.exists() || !path_to_db.is_dir() {
            // If project DB has been deleted, we remove the project from the collection
            match ProjectCollection::delete(project_uuid) {
                Ok(_) => {}
                Err(_) => {}
            }
            return Err(PiError::InternalError(format!(
                "DB for project {} does not exist",
                project_uuid
            )));
        }
        Ok((path_to_storage_dir, path_to_db))
    }
}
