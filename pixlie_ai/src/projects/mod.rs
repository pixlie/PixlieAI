use std::{path::PathBuf, sync::Arc};

use crate::{
    error::{PiError, PiResult},
    utils::crud::{Crud, CrudItem},
};
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod api;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum ProjectOwner {
    Myself,
    User(String),
    Organization(String),
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Project {
    pub uuid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: ProjectOwner,
}

#[derive(Deserialize, Serialize, TS)]
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

impl ProjectForEngine {
    pub fn new(project: Project, path_to_storage_dir: PathBuf) -> PiResult<Self> {
        let mut project = Self {
            project,
            path_to_storage_dir,
            arced_db: None,
        };
        project.init_db()?;
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
        let mut path_to_storage_dir = self.path_to_storage_dir.clone();
        path_to_storage_dir.push(format!("{}.rocksdb", self.project.uuid));
        path_to_storage_dir
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
