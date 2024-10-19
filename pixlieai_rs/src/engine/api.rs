use super::state::PiState;
use crate::entity::LabelId;
use serde::{Deserialize, Serialize};

impl PiState {
    pub fn upsert_data_of_type<T>(&self, entity_type: LabelId, rows: Vec<T>)
    where
        T: Deserialize<'static> + Serialize,
    {
    }
}
