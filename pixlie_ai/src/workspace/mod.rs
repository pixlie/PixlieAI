use crate::utils::crud::{Crud, CrudItem};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::error::PiResult;

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

impl WorkspaceCollection {
    pub fn get_default() -> PiResult<Workspace> {
        let items = Self::read_list()?;
        if items.len() == 0 {
            let item: Workspace = Workspace {
                uuid: uuid::Uuid::new_v4().to_string(),
                name: "Default".to_string(),
                description: None,
                anthropic_api_key: None,
            };
            Ok(WorkspaceCollection::create(item)?)
        } else {
            Ok(items[0].clone())
        }
    }
}

impl Crud for WorkspaceCollection {
    type Item = Workspace;

    fn get_collection_name() -> &'static str {
        "workspace"
    }
}
