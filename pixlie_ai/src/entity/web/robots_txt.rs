use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub(crate) struct RobotsTxt {
    pub(crate) contents: String,
}

impl RobotsTxt {}
