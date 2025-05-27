use crate::utils::crud::{Crud, CrudItem};
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
