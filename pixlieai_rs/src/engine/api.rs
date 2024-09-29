use super::state::PiState;
use crate::entity::EntityType;
use serde::{Deserialize, Serialize};

impl PiState {
    pub fn upsert_data_of_type<T>(&self, entity_type: EntityType, rows: Vec<T>)
    where
        T: Deserialize<'static> + Serialize,
    {
    }
}
