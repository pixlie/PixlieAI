use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct RobotsTxt {
    pub contents: String,
}

impl RobotsTxt {}
