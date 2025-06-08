use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{Engine, NodeFlags};
use crate::entity::pixlie::disable_tools_by_labels;
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
            log::error!("LLM response for Coonclusion is empty");
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
                            // Automatically disable crawler and classifier settings when conclusion is reached
                            // TODO: implement MonitorSettings to re-enable them if needed
                            disable_tools_by_labels(
                                engine.clone(),
                                vec![NodeLabel::CrawlerSettings, NodeLabel::ClassifierSettings],
                            )?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_settings_logic() {
        use crate::engine::engine::get_test_engine;
        use crate::entity::classifier::ClassifierSettings;
        use crate::entity::crawler::CrawlerSettings;
        use crate::entity::pixlie::{disable_tool_by_label, disable_tools_by_labels, ToolEnabled};
        use std::sync::Arc;

        // Test the disable_settings function with actual engine
        let test_engine = get_test_engine();
        let arced_test_engine = Arc::new(&test_engine);

        // Create enabled ClassifierSettings
        let classifier_node_id = test_engine
            .get_or_add_node(
                Payload::ClassifierSettings(ClassifierSettings {
                    is_enabled: ToolEnabled::Yes,
                    prompt_to_classify_content_as_relevant_to_objective_or_not: Some(
                        "Test prompt".to_string(),
                    ),
                }),
                vec![NodeLabel::ClassifierSettings, NodeLabel::AddedByUser],
                true,
                None,
            )
            .unwrap()
            .get_node_id();

        // Create enabled CrawlerSettings
        let crawler_node_id = test_engine
            .get_or_add_node(
                Payload::CrawlerSettings(CrawlerSettings {
                    is_enabled: ToolEnabled::Yes,
                    keywords_to_get_accurate_results_from_web_search: Some(
                        vec!["test".to_string()],
                    ),
                    crawl_link_if_anchor_text_has_any_of_these_keywords: None,
                }),
                vec![NodeLabel::CrawlerSettings, NodeLabel::AddedByUser],
                true,
                None,
            )
            .unwrap()
            .get_node_id();

        // Verify tools are initially enabled
        let classifier_node = test_engine.get_node_by_id(&classifier_node_id).unwrap();
        if let Payload::ClassifierSettings(settings) = &classifier_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::Yes));
        }

        let crawler_node = test_engine.get_node_by_id(&crawler_node_id).unwrap();
        if let Payload::CrawlerSettings(settings) = &crawler_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::Yes));
        }

        // Test disabling individual tools
        disable_tool_by_label(arced_test_engine.clone(), NodeLabel::ClassifierSettings).unwrap();

        // Verify ClassifierSettings is disabled
        let classifier_node = test_engine.get_node_by_id(&classifier_node_id).unwrap();
        if let Payload::ClassifierSettings(settings) = &classifier_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::No));
        }

        // Verify CrawlerSettings is still enabled
        let crawler_node = test_engine.get_node_by_id(&crawler_node_id).unwrap();
        if let Payload::CrawlerSettings(settings) = &crawler_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::Yes));
        }

        // Test disabling the other tool
        disable_tool_by_label(arced_test_engine.clone(), NodeLabel::CrawlerSettings).unwrap();

        // Verify CrawlerSettings is now disabled
        let crawler_node = test_engine.get_node_by_id(&crawler_node_id).unwrap();
        if let Payload::CrawlerSettings(settings) = &crawler_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::No));
        }

        // Test error case for unsupported tool
        let result = disable_tool_by_label(arced_test_engine.clone(), NodeLabel::Objective);
        assert!(result.is_err());

        // Test the new disable_tools_by_labels function
        // Re-enable tools first to test the bulk disable function
        test_engine
            .update_node(
                &classifier_node_id,
                Payload::ClassifierSettings(ClassifierSettings {
                    is_enabled: ToolEnabled::Yes,
                    prompt_to_classify_content_as_relevant_to_objective_or_not: Some(
                        "Test prompt".to_string(),
                    ),
                }),
            )
            .unwrap();

        test_engine
            .update_node(
                &crawler_node_id,
                Payload::CrawlerSettings(CrawlerSettings {
                    is_enabled: ToolEnabled::Yes,
                    keywords_to_get_accurate_results_from_web_search: Some(
                        vec!["test".to_string()],
                    ),
                    crawl_link_if_anchor_text_has_any_of_these_keywords: None,
                }),
            )
            .unwrap();

        // Verify tools are re-enabled
        let classifier_node = test_engine.get_node_by_id(&classifier_node_id).unwrap();
        if let Payload::ClassifierSettings(settings) = &classifier_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::Yes));
        }

        let crawler_node = test_engine.get_node_by_id(&crawler_node_id).unwrap();
        if let Payload::CrawlerSettings(settings) = &crawler_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::Yes));
        }

        // Test disabling multiple tools at once
        disable_tools_by_labels(
            arced_test_engine.clone(),
            vec![NodeLabel::CrawlerSettings, NodeLabel::ClassifierSettings],
        )
        .unwrap();

        // Verify both tools are disabled
        let classifier_node = test_engine.get_node_by_id(&classifier_node_id).unwrap();
        if let Payload::ClassifierSettings(settings) = &classifier_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::No));
        }

        let crawler_node = test_engine.get_node_by_id(&crawler_node_id).unwrap();
        if let Payload::CrawlerSettings(settings) = &crawler_node.payload {
            assert!(matches!(settings.is_enabled, ToolEnabled::No));
        }

        // Test error case for bulk disable with unsupported tool
        let result = disable_tools_by_labels(
            arced_test_engine,
            vec![NodeLabel::CrawlerSettings, NodeLabel::Objective],
        );
        assert!(result.is_err());

        // Test the JSON parsing works correctly (full Claude API response format)
        let response_json = r#"{
            "id": "test_id",
            "content": [{
                "type": "text",
                "text": "{\"conclusion_to_objective_based_on_insights\": \"Test conclusion reached\"}"
            }]
        }"#;
        let parsed_conclusion = Conclusion::parse_llm_response(response_json).unwrap();

        // Verify the conclusion has content
        match &parsed_conclusion.conclusion_to_objective_based_on_insights {
            Some(conclusion_text) => {
                assert_eq!(conclusion_text, "Test conclusion reached");
                assert!(!conclusion_text.trim().is_empty());
            }
            None => panic!("Conclusion should have content"),
        }

        // Test that empty conclusion doesn't trigger
        let empty_response_json = r#"{
            "id": "test_id",
            "content": [{
                "type": "text", 
                "text": "{\"conclusion_to_objective_based_on_insights\": null}"
            }]
        }"#;
        let empty_conclusion = Conclusion::parse_llm_response(empty_response_json).unwrap();

        match &empty_conclusion.conclusion_to_objective_based_on_insights {
            Some(_) => panic!("Empty conclusion should be None"),
            None => {} // Expected
        }
    }
}
