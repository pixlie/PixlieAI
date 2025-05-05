// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine};
use crate::error::PiError;
use crate::error::PiResult;
use crate::services::anthropic::Anthropic;
use crate::utils::llm::LLMProvider;
use crate::utils::llm::{clean_ts_type, LLMSchema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ClassifierSettings {
    pub query_to_classify_content_as_relevant_or_irrelevant_to_objective: Option<String>,
}

#[derive(Deserialize, TS)]
pub struct LLMResponse {
    pub is_relevant: bool,
    pub reason: String,
    pub insight: String,
}

impl LLMSchema for ClassifierSettings {
    fn get_schema_for_llm(node: &NodeItem, _engine: Arc<&Engine>) -> PiResult<String> {
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        match node.payload {
            Payload::Text(_) => {
                if node.labels.contains(&NodeLabel::Objective) {
                    // Make fields required when there's content to classify
                    return Ok(ts_self
                        .replace(
                            "query_to_classify_content_as_relevant_or_irrelevant_to_objective: string | null,",
                            "query_to_classify_content_as_relevant_or_irrelevant_to_objective: string,",
                        ));
                }
            }
            _ => {}
        }

        Ok(ts_self)
    }
}

impl LLMSchema for LLMResponse {
    fn get_schema_for_llm(_node: &NodeItem, _engine: Arc<&Engine>) -> PiResult<String> {
        let ts_self = clean_ts_type(&Self::export_to_string()?);
        Ok(ts_self)
    }
}

pub fn classify(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
    let llm_prompt = get_llm_prompt(node, engine.clone())?;
    let engine_request = Anthropic::get_request(&llm_prompt, node.id)?;
    engine.fetch_api(engine_request)
}

pub fn parse_llm_response(response: &str) -> PiResult<LLMResponse> {
    Ok(Anthropic::parse_response::<LLMResponse>(response)?)
}

pub fn get_llm_prompt(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
    let content = engine
        .get_node_ids_connected_with_label(&node.id, &EdgeLabel::ParentOf)?
        .into_iter()
        .filter_map(|id| engine.get_node_by_id(&id))
        .filter(|node| {
            matches!(
                node.labels.as_slice(),
                [NodeLabel::Title | NodeLabel::Heading | NodeLabel::Paragraph, ..]
            )
        })
        .filter_map(|node| match &node.payload {
            Payload::Text(text) => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    // log::info!("✏️ Content to classify: {}", content.clone());

    let query = engine
    .get_node_ids_with_label(&NodeLabel::Objective)
    .first()
    .ok_or_else(|| PiError::InternalError("No Objective nodes found".to_string()))
    .and_then(|id| {
        engine
            .get_node_ids_connected_with_label(id, &EdgeLabel::Classifies)?
            .first()
            .ok_or_else(|| PiError::InternalError("No ClassifierSettings found for Objective".to_string()))
            .and_then(|settings_id| {
                match &engine
                    .get_node_by_id(settings_id)
                    .ok_or_else(|| PiError::InternalError(format!("ClassifierSettings node with id {} not found", settings_id)))?
                    .payload
                {
                    Payload::ClassifierSettings(settings) => {
                        let query = settings.query_to_classify_content_as_relevant_or_irrelevant_to_objective.clone()
                            .ok_or_else(|| PiError::GraphError("Missing query_to_classify_content_as_relevant_or_irrelevant_to_objective in ClassifierSettings".to_string()))?;
                        Ok(query
                            .split(": ")
                            .nth(1)
                            .unwrap_or(&query)
                            .to_string())
                    },
                    _ => Err(PiError::GraphError("Invalid payload type for ClassifierSettings".to_string()))
                }
            })
    })?;

    // log::info!("❓ Query to classify: {}", query.clone());

    Ok(format!(
        r#"Your task is to analyze the following content and classify it as either relevant or irrelevant based on the given query.

        Content:
        {}

        Query:
        {}

        Using the following schema, respond in JSON format with your classification decision, insight, and reason:
        {}
        ```"#,
        content,
        query,
        format!(
            "{}",
            LLMResponse::get_schema_for_llm(node, engine.clone())?.as_str()
        )
    ))
}
