use super::Engine;
use serde::{Deserialize, Serialize};

impl Engine {
    pub fn upsert_data_of_type<T>(&self, label: String, rows: Vec<T>)
    where
        T: Deserialize<'static> + Serialize,
    {
    }
}
