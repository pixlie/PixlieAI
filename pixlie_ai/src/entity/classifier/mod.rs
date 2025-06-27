// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::entity::pixlie::{is_tool_enabled, ToolEnabled};
use crate::error::{PiError, PiResult};
use crate::services::anthropic::Anthropic;
use crate::utils::llm::LLMProvider;
use crate::utils::llm::{clean_ts_type, LLMSchema};
use crate::ExternalData;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ClassifierSettings {
    #[serde(default)]
    pub is_enabled: ToolEnabled,
    pub prompt_to_classify_content_as_relevant_to_objective_or_not: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct Classification {
    pub is_relevant: bool,
    pub reason: String,
    pub insight_if_classified_as_relevant: Option<String>,
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
                            "is_enabled: ToolEnabled | null,",
                            "is_enabled: ToolEnabled,",
                        )
                        .replace(
                            "prompt_to_classify_content_as_relevant_to_objective_or_not: string | null,",
                            "prompt_to_classify_content_as_relevant_to_objective_or_not: string,",
                        ));
                }
            }
            _ => {}
        }

        Ok(ts_self)
    }
}

impl LLMSchema for Classification {
    fn get_schema_for_llm(_node: &NodeItem, _engine: Arc<&Engine>) -> PiResult<String> {
        Ok(clean_ts_type(&Self::export_to_string()?))
    }
}

pub struct Classifier;

impl Classifier {
    pub fn get_llm_prompt(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
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

        let prompt_for_classification = engine
            .get_node_ids_with_label(&NodeLabel::ClassifierSettings)
            .first()
            .ok_or_else(|| PiError::InternalError("No ClassifierSettings found".to_string()))
            .and_then(|id| {
                match &engine
                    .get_node_by_id(id)
                    .ok_or_else(|| PiError::InternalError(format!("ClassifierSettings node with id {} not found", id)))?
                    .payload
                {
                    Payload::ClassifierSettings(settings) => {
                        let prompt = settings.prompt_to_classify_content_as_relevant_to_objective_or_not.clone()
                                    .ok_or_else(|| PiError::GraphError("Missing prompt_to_classify_content_as_relevant_to_objective_or_not in ClassifierSettings".to_string()))?;
                                Ok(prompt
                                    .split(": ")
                                    .nth(1)
                                    .unwrap_or(&prompt)
                                    .to_string())
                    },
                    _ => Err(PiError::GraphError("Invalid payload type for ClassifierSettings".to_string()))
                }
            })?;

        // log::info!("❓ Prompt for classification: {}", prompt_for_classify: {}", query.clone());

        Ok(format!(
            r#"{}

        Content to be classified:
        {}

        Using the following schema, respond in JSON format:
        {}
        ```"#,
            prompt_for_classification,
            content,
            format!(
                "{}",
                Classification::get_schema_for_llm(node, engine.clone())?.as_str()
            )
        ))
    }

    pub fn send_llm_request(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
        let llm_prompt = Self::get_llm_prompt(node, engine.clone())?;
        let engine_request = Anthropic::get_request(&llm_prompt, node.id)?;
        engine.fetch_api(engine_request)
    }

    pub fn parse_llm_response(response: &str) -> PiResult<Classification> {
        if response.trim().is_empty() {
            log::error!("LLM response for Classification is empty");
            return Err(PiError::InternalError(
                "LLM response is empty or contains only whitespace".to_string(),
            ));
        }
        Ok(Anthropic::parse_response::<Classification>(response)?)
    }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        if !is_tool_enabled(engine.clone(), NodeLabel::ClassifierSettings)? {
            debug!(
                "ClassifierSettings is disabled, skipping Classifier processing for node {}",
                node.id
            );
            return Ok(());
        }
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    let parsed_response = &Self::parse_llm_response(&response.contents)?;
                    if parsed_response.is_relevant {
                        log::info!("🟢 WebPage node {} is relevant.", node.id);
                    } else {
                        log::info!("🔴 WebPage node {} is not relevant.", node.id);
                    }
                    let classification_node_id = engine
                        .get_or_add_node(
                            Payload::Classification(Classification {
                                is_relevant: parsed_response.is_relevant.clone(),
                                reason: parsed_response.reason.clone(),
                                insight_if_classified_as_relevant: parsed_response
                                    .insight_if_classified_as_relevant
                                    .clone(),
                            }),
                            vec![NodeLabel::Classification, NodeLabel::AddedByAI],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node.id.clone(), classification_node_id),
                        (EdgeLabel::Classifies, EdgeLabel::ClassifiedFor),
                    )?;

                    // Send insight notification when webpage is classified as relevant with an insight
                    if parsed_response.is_relevant {
                        if let Some(insight) = &parsed_response.insight_if_classified_as_relevant {
                            // Get the URL from the WebPage node using the same pattern as scraper
                            if let Ok((current_link, current_link_node_id)) =
                                crate::entity::web::web_page::get_link_of_webpage(
                                    engine.clone(),
                                    &node.id,
                                )
                            {
                                if let Ok(Some(existing_domain)) = crate::entity::web::domain::Domain::find_existing(
                                    engine.clone(),
                                    crate::entity::web::domain::FindDomainOf::Node(current_link_node_id),
                                ) {
                                    if let Ok(domain_name) = crate::entity::web::domain::Domain::get_domain_name(&existing_domain) {
                                        let full_url = format!("https://{}{}", domain_name, current_link.get_full_link());
                                        log::info!("🎯 Sending insight notification for URL: {}", full_url);
                                        if let Err(e) = engine.send_insight_notification(
                                            &full_url,
                                            insight,
                                            &parsed_response.reason,
                                        ) {
                                            log::error!(
                                                "📧 Failed to send insight notification: {}",
                                                e
                                            );
                                        }
                                    } else {
                                        log::warn!("🎯 Could not get domain name for Link node {}, skipping insight notification", current_link_node_id);
                                    }
                                } else {
                                    log::warn!("🎯 Could not find domain for Link node {}, skipping insight notification", current_link_node_id);
                                }
                            } else {
                                log::warn!("🎯 Could not get Link for WebPage node {}, skipping insight notification", node.id);
                            }
                        }
                    }
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(_error) => {}
            },
            None => {
                Self::send_llm_request(node, engine.clone())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
