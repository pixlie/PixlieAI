// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

/*!
Here we define all the entities that we can extract from the data. Each entity is
stored in the graph as a node, and edges are created to represent the relationships
between them.

Not all entity types have a corresponding node. Some nodes have an internal type,
and therefore many entities may point to the same node.
*/

use crate::error::PiResult;
use crate::utils::llm::{clean_ts_type, LLMSchema};
use serde::{Deserialize, Serialize};
use strum::EnumString;
use ts_rs::TS;
use utoipa::ToSchema;

pub mod classifier;
pub mod content;
pub mod crawler;
pub mod email;
pub mod objective;
pub mod pixlie;
pub mod project_settings;
pub mod search;
pub mod text;
pub mod web;

#[derive(Clone, Deserialize, EnumString, Serialize, ToSchema, TS)]
pub enum EntityName {
    // We currently support the following entities to be extracted
    Person,
    Organization,
    Date,
    Place,
}

impl LLMSchema for EntityName {
    fn get_schema_for_llm(
        _node: &crate::engine::node::NodeItem,
        _engine: std::sync::Arc<&crate::engine::Engine>,
    ) -> PiResult<String> {
        Ok(clean_ts_type(&Self::export_to_string()?))
    }
}

// This is the struct used to extract entities from the data using any of the entity extraction providers
pub struct ExtractedEntity {
    pub entity_name: EntityName,
    pub matching_text: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub probability: Option<f32>,
}
