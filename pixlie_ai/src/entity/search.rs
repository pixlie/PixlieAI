use crate::engine::{CommonNodeLabels, Engine, Payload};
use crate::error::PiResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct SearchTerm(pub String);

impl SearchTerm {
    pub fn add_manually(engine: Arc<&Engine>, search_term: &str) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::SearchTerm(Self(search_term.to_string())),
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
        )?;
        Ok(())
    }
}
