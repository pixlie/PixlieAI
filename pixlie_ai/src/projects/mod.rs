use std::{path::PathBuf, sync::Arc};

use crate::{
    error::{PiError, PiResult},
    utils::crud::{Crud, CrudItem},
};
use log::{debug, error, info};
use rocksdb::DB;
use serde::{Deserialize, Serialize};
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
    pub uuid: String,
    pub name: Option<String>,
    pub description: Option<String>,
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
        Project {
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            owner,
        }
    }
    pub fn from_id(uuid: &str) -> Project {
        Project {
            uuid: uuid.to_string(),
            name: None,
            description: None,
            owner: ProjectOwner::Myself,
        }
    }
}

pub struct ProjectForEngine {
    pub project: Project,
    pub path_to_storage_dir: PathBuf,
    pub arced_db: Option<Arc<DB>>,
}

pub struct EngineProjectValidationResult {
    pub error_response_message_for_api: Option<String>,
    pub should_unload_engine: bool,
}

impl ProjectForEngine {
    pub fn new(project: Project, path_to_storage_dir: PathBuf) -> PiResult<Self> {
        let mut project = Self {
            project,
            path_to_storage_dir,
            arced_db: None,
        };
        project.init_db()?;
        // TODO: Ideally this should be called here, but tests are failing
        // if it is called here.
        // We can move this call here once Settings is refactored to be
        // cleanly testable and functions dependent on Settings are
        // moved to impl Settings
        // ProjectCollection::create(project.project.clone())?;
        Ok(project)
    }
    pub fn open(uuid: &str, path_to_storage_dir: PathBuf) -> Self {
        let project = Self {
            project: Project::from_id(uuid),
            path_to_storage_dir,
            arced_db: None,
        };
        project
    }
    pub fn get_db_path(&self) -> PathBuf {
        self.path_to_storage_dir
            .join(format!("{}.rocksdb", self.project.uuid))
    }
    pub fn get_arced_db(&self) -> PiResult<Arc<DB>> {
        match self.arced_db.as_ref() {
            Some(db) => Ok(db.clone()),
            None => Err(PiError::InternalError(
                "Cannot get DB: DB is not open".to_string(),
            )),
        }
    }
    fn init_db(&mut self) -> PiResult<()> {
        let db_path = self.get_db_path();
        match DB::open_default(db_path.as_os_str()) {
            Ok(_) => Ok(()),
            Err(error) => Err(PiError::InternalError(format!(
                "Cannot open DB for project: {}",
                error
            ))),
        }
    }
    pub fn load_db(&mut self) -> PiResult<()> {
        let db_path = self.get_db_path();
        if db_path.exists() && db_path.is_dir() {
            let mut opts = rocksdb::Options::default();
            opts.create_if_missing(false);
            let db = DB::open(&opts, db_path.as_os_str())?;
            self.arced_db = Some(Arc::new(db));
            Ok(())
        } else {
            Err(PiError::InternalError(format!(
                "DB does not exist at path: {}",
                db_path.display()
            )))
        }
    }
    pub fn db_exists(&self) -> bool {
        self.get_db_path().exists()
    }

    pub fn validate_record_and_db(&self) -> PiResult<EngineProjectValidationResult> {
        let project_in_collection = ProjectCollection::read_item(&self.project.uuid);
        let mut validation_result = EngineProjectValidationResult {
            error_response_message_for_api: None,
            should_unload_engine: false,
        };
        let db_exists = self.db_exists();
        match project_in_collection.as_ref() {
            Ok(project) => {
                if self.project.uuid != project.uuid {
                    return Err(PiError::InternalError(format!(
                        "Project ID mismatch: {} != {}",
                        self.project.uuid, project.uuid
                    )));
                }
                if !db_exists {
                    error!(
                        "Project DB for {} does not exist, deleting project",
                        project.uuid
                    );
                    if ProjectCollection::delete(&project.uuid).is_err() {
                        error!("Error deleting project {} from collection", project.uuid);
                    } else {
                        info!(
                            "Project {} deleted from collection as DB was not found",
                            project.uuid
                        );
                    }
                }
            }
            Err(PiError::CrudNotFoundError(item, msg)) => {
                debug!("{msg}: {item:?}");
                validation_result.error_response_message_for_api = Some(format!(
                    "Project {} does not exist: {msg}",
                    self.project.uuid
                ));
                validation_result.should_unload_engine = true;
                if db_exists {
                    info!(
                        "{msg}: {item:?}, DB found at {} can be deleted",
                        self.get_db_path().display()
                    );
                    // TODO: delete the dangling DB folder, if it is safe to do so
                    // std::fs::remove_dir_all(self.get_db_path()).unwrap_or_else(|_| {
                    //     error!(
                    //         "Failed to delete DB folder at {}",
                    //         self.get_db_path().display()
                    //     );
                    // });
                }
            }
            Err(err) => {
                debug!("Error reading project {}: {err}", self.project.uuid);
                validation_result.error_response_message_for_api = Some(format!(
                    "Error while validating project {}: {err}",
                    self.project.uuid
                ));
            }
        };
        Ok(validation_result)
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
