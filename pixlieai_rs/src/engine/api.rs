use crate::entity::EntityType;
use serde::{Deserialize, Serialize};

pub fn upsert_data_of_type<T>(entity_type: EntityType, rows: Vec<T>)
where
    T: Deserialize<'static> + Serialize,
{
}
