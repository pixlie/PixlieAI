use serde::{Deserialize, Serialize};
use ts_rs::TS;

// The project settings node contains high level settings that guide the flow of a project
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct ProjectSettings {
    pub has_user_specified_starting_links: bool,
}