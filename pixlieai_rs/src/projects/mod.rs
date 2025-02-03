use crate::utils::crud::{Crud, CrudItem};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod api;

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub enum ProjectOwner {
    Myself,
    User(String),
    Organization(String),
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Project {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub owner: ProjectOwner,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ProjectCreate {
    pub name: String,
    pub description: Option<String>,
    pub starting_urls: Vec<String>,
    pub webpage_keywords: Vec<String>,
}

impl Project {
    pub fn new(name: String, description: Option<String>, owner: ProjectOwner) -> Project {
        Project {
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            owner,
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

    fn get_collection_name() -> String {
        "project".to_string()
    }
}
