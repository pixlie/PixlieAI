use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine};
use crate::entity::pixlie::is_tool_enabled;
use crate::error::{PiError, PiResult};
use crate::services::gliner::extract_entities;
use crate::utils::llm::{clean_ts_type, LLMSchema};
use crate::ExternalData;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{Display, EnumString};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Display, EnumString, Serialize, ToSchema, TS)]
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
#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ExtractedEntity {
    pub entity_name: EntityName,
    pub matching_text: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub probability: Option<f32>,
}

pub struct EntityExtraction;

impl EntityExtraction {
    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        _data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        if !is_tool_enabled(engine.clone(), NodeLabel::ClassifierSettings)? {
            debug!(
                "ClassifierSettings is disabled, skipping EntityExtraction processing for node {}",
                node.id
            );
            return Ok(());
        }
        // Get the content
        let content = engine
            .get_node_ids_connected_with_label(&node.id, &EdgeLabel::ParentOf)?
            .into_iter()
            .filter_map(|id| match engine.get_node_by_id(&id) {
                None => None,
                Some(node) => match &node.payload {
                    Payload::Text(text) => {
                        if node.labels.contains(&NodeLabel::Partial) {
                            Some(text.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        let objective_id = engine
            .get_node_ids_with_label(&NodeLabel::Objective)
            .first()
            .ok_or_else(|| PiError::InternalError("No Objective nodes found".to_string()))?
            .clone();

        let named_entities_to_extract: Option<Vec<EntityName>> = engine
            .get_node_ids_connected_with_label(&objective_id, &EdgeLabel::Suggests)?
            .iter()
            .find_map(|node_id| match &engine.get_node_by_id(node_id) {
                None => None,
                Some(node) => {
                    if node.labels.contains(&NodeLabel::NamedEntitiesToExtract) {
                        match &node.payload {
                            Payload::NamedEntitiesToExtract(named_entities) => {
                                Some(named_entities.clone())
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
            });

        match named_entities_to_extract {
            Some(named_entities) => {
                // Extract the suggested named entities
                let extracted_entities = extract_entities(content, named_entities)?;

                let extracted_entities = engine
                    .get_or_add_node(
                        Payload::ExtractedNamedEntities(extracted_entities),
                        vec![NodeLabel::AddedByGliner, NodeLabel::ExtractedNamedEntities],
                        true,
                        None,
                    )?
                    .get_node_id();

                engine.add_connection(
                    (node.id, extracted_entities),
                    (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                )?;
            }
            None => {}
        }

        Ok(())
    }
}
