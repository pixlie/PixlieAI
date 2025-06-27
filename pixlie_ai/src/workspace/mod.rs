use crate::error::PiResult;
use crate::utils::crud::{Crud, CrudItem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use utoipa::ToSchema;

pub mod api;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize, ToSchema, TS)]
#[ts(export)]
pub enum APIProvider {
    Anthropic,
    BraveSearch,
    SendGrid,
}

// A workspace is the set of settings that affect external services,
// permissions and other configuration.
//
// For now, there is only one workspace (created automatically).
// All projects share the same workspace.
#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export, rename_all = "camelCase")]
pub struct Workspace {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,

    pub api_keys: HashMap<APIProvider, String>,

    pub sendgrid_sender_email: Option<String>,
    pub sendgrid_receiver_email: Option<String>,
}

impl Workspace {
    pub fn get_api_key(&self, provider: &APIProvider) -> Option<&String> {
        if let Some(key) = self.api_keys.get(&provider) {
            if key.is_empty() {
                None
            } else {
                Some(key)
            }
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct WorkspaceUpdate {
    pub name: Option<String>,        // Not used yet
    pub description: Option<String>, // Not used yet

    pub anthropic_api_key: Option<String>,
    pub brave_search_api_key: Option<String>,
    pub sendgrid_api_key: Option<String>,
    pub sendgrid_sender_email: Option<String>,
    pub sendgrid_receiver_email: Option<String>,
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
                api_keys: HashMap::new(),
                sendgrid_sender_email: None,
                sendgrid_receiver_email: None,
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
