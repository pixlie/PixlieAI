use crate::utils::crud::{Crud, CrudItem};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod api;

// A workspace is the set of settings that affect external services,
// permissions and other configuration.
//
// For now, there is only one workspace (created automatically).
// All projects share the same workspace.
#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, rename_all = "camelCase")]
pub struct Workspace {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,

    pub anthropic_api_key: Option<String>,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct WorkspaceUpdate {
    pub name: Option<String>,        // Not used yet
    pub description: Option<String>, // Not used yet
    pub anthropic_api_key: Option<String>,
}

impl CrudItem for Workspace {
    fn get_id(&self) -> String {
        self.uuid.clone()
    }
}

pub struct WorkspaceCollection {}

impl Crud for WorkspaceCollection {
    type Item = Workspace;

    fn get_collection_name() -> &'static str {
        "workspace"
    }
}
