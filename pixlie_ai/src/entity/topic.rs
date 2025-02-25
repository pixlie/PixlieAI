use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{engine::{CommonNodeLabels, Engine, Node, Payload}, error::PiResult};



#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Topic(pub String);

impl Topic {
    pub fn add_manually(engine: Arc<&Engine>, topic: &str) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::Topic(Self(topic.to_string())),
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
        )?;
        Ok(())
    }
}

impl Node for Topic {
    fn get_label() -> String {
        "Topic".to_string()
    }
}