use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{Engine, NodeFlags};
use crate::error::{PiError, PiResult};
use crate::services::anthropic::Anthropic;
use crate::utils::llm::{clean_ts_type, LLMProvider, LLMSchema};
use crate::ExternalData;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Clone, Debug, Default, Deserialize, Serialize, ToSchema, TS)]
pub struct Conclusion {
    conclusion_to_objective_based_on_insights: Option<String>,
}

const MINIMUM_INSIGHTS_COUNT: usize = 3;

impl LLMSchema for Conclusion {
    fn get_schema_for_llm(_node: &NodeItem, _engine: Arc<&Engine>) -> PiResult<String> {
        Ok(clean_ts_type(&Self::export_to_string()?))
    }
}

impl Conclusion {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_objective(engine: Arc<&Engine>) -> PiResult<String> {
        let node = engine
            .get_node_ids_with_label(&NodeLabel::Objective)
            .first()
            .ok_or_else(|| PiError::InternalError("No Objective nodes found in engine".to_string()))
            .and_then(|id| {
                engine.get_node_by_id(id).ok_or_else(|| {
                    PiError::InternalError(format!(
                        "Failed to retrieve Objective node with id: {}",
                        id
                    ))
                })
            })?;
        match &node.payload {
            Payload::Text(text) if !text.trim().is_empty() => Ok(text.clone()),
            Payload::Text(_) => Err(PiError::InternalError(
                "Objective node contains empty text".to_string(),
            )),
            _ => Err(PiError::InternalError(
                "Objective node payload is not text type".to_string(),
            )),
        }
    }

    fn get_insights(engine: Arc<&Engine>) -> PiResult<(String, usize)> {
        let insights: HashSet<String> = engine
            .get_node_ids_with_label(&NodeLabel::Classification)
            .into_iter()
            .filter_map(|id| engine.get_node_by_id(&id))
            .filter_map(|node| {
                if let Payload::Classification(classification) = &node.payload {
                    if classification.is_relevant {
                        classification
                            .insight_if_classified_as_relevant
                            .as_ref()
                            .filter(|insight| !insight.trim().is_empty())
                            .map(|insight| insight.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        let insights_count = insights.len();
        if insights_count >= MINIMUM_INSIGHTS_COUNT {
            Ok((
                insights
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>()
                    .join("\n\n"),
                insights_count,
            ))
        } else {
            Err(PiError::InternalError(format!(
                "Insufficient insights found: {} (minimum required: {})",
                insights_count, MINIMUM_INSIGHTS_COUNT
            )))
        }
    }

    pub fn get_llm_prompt(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        let objective = Self::get_objective(engine.clone())?;
        let (insights, _) = Self::get_insights(engine.clone())?;
        let schema = Self::get_schema_for_llm(node, engine.clone())?;
        Ok(format!(
            "Objective: {}\n\nInsights: {}\n\nUsing the following schema, respond in JSON format:\n\n{}\n```",
            objective, insights, schema
        ))
    }

    pub fn send_llm_request(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
        let llm_prompt = Self::get_llm_prompt(node, engine.clone())?;
        let engine_request = Anthropic::get_request(&llm_prompt, node.id)?;
        engine.fetch_api(engine_request)
    }

    pub fn parse_llm_response(response: &str) -> PiResult<Conclusion> {
        if response.trim().is_empty() {
            return Err(PiError::InternalError(
                "LLM response is empty or contains only whitespace".to_string(),
            ));
        }
        Anthropic::parse_response::<Conclusion>(response).map_err(|e| {
            PiError::InternalError(format!("Failed to parse LLM response as Conclusion: {}", e))
        })
    }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    let parsed_response = Self::parse_llm_response(&response.contents)?;
                    match &parsed_response.conclusion_to_objective_based_on_insights {
                        Some(conclusion) if !conclusion.trim().is_empty() => {
                            engine.update_node(&node.id, Payload::Conclusion(parsed_response))?;
                            engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                        }
                        _ => {}
                    }
                }
                ExternalData::Error(_error) => {}
            },
            None => match Self::get_insights(engine.clone()) {
                Ok((_, insights_count)) => {
                    if insights_count >= MINIMUM_INSIGHTS_COUNT {
                        Self::send_llm_request(node, engine.clone())?;
                    }
                }
                Err(_err) => {}
            },
        }
        Ok(())
    }
}
