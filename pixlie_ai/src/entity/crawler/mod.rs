// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine};
use crate::entity::pixlie::ToolEnabled;
use crate::error::PiResult;
use crate::utils::llm::{clean_ts_type, LLMSchema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct CrawlerSettings {
    #[serde(default)]
    pub is_enabled: ToolEnabled,
    pub keywords_to_get_accurate_results_from_web_search: Option<Vec<String>>,
    pub crawl_link_if_anchor_text_has_any_of_these_keywords: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_crawler_settings_backward_compatibility() {
        // Test that old CrawlerSettings without status field can be deserialized
        let old_json = r#"{
            "keywords_to_get_accurate_results_from_web_search": ["test"],
            "crawl_link_if_anchor_text_has_any_of_these_keywords": null
        }"#;

        let settings: CrawlerSettings = serde_json::from_str(old_json).unwrap();

        // Should default to ToolEnabled::Yes
        match settings.is_enabled {
            ToolEnabled::Yes => {}
            _ => panic!("Expected default status to be Yes"),
        }

        assert_eq!(
            settings.keywords_to_get_accurate_results_from_web_search,
            Some(vec!["test".to_string()])
        );
        assert_eq!(
            settings.crawl_link_if_anchor_text_has_any_of_these_keywords,
            None
        );
    }

    #[test]
    fn test_crawler_settings_with_status() {
        // Test that new CrawlerSettings with status field works
        let new_json = r#"{
            "is_enabled": "No",
            "keywords_to_get_accurate_results_from_web_search": ["test"],
            "crawl_link_if_anchor_text_has_any_of_these_keywords": null
        }"#;

        let settings: CrawlerSettings = serde_json::from_str(new_json).unwrap();

        match settings.is_enabled {
            ToolEnabled::No => {}
            _ => panic!("Expected status to be No"),
        }
    }
}

impl LLMSchema for CrawlerSettings {
    fn get_schema_for_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        // If the payload is for an Objective and there is a ProjectSetting connected to it,
        // then we check if there is a setting for starting links
        // If starting links are to be manually provided, then we remove the web_search_keywords_for_objective field
        let mut has_web_search_keywords_to_get_starting_urls_for_crawl: bool = true;
        match node.payload {
            Payload::Text(_) => {
                if node.labels.contains(&NodeLabel::Objective) {
                    let project_settings_node_ids = engine
                        .get_node_ids_connected_with_label(&node.id, &EdgeLabel::RelatedTo)?;
                    if project_settings_node_ids.len() > 0 {
                        for project_settings_node_id in project_settings_node_ids {
                            match engine.get_node_by_id(&project_settings_node_id) {
                                Some(project_settings_node) => match &project_settings_node.payload
                                {
                                    Payload::ProjectSettings(project_settings) => {
                                        if project_settings
                                            .only_crawl_direct_links_from_specified_links
                                        {
                                            has_web_search_keywords_to_get_starting_urls_for_crawl =
                                                false;
                                        }
                                        break;
                                    }
                                    _ => {}
                                },
                                None => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        };

        let ts_self = if has_web_search_keywords_to_get_starting_urls_for_crawl {
            // Change the field web_search_keywords_for_objective to non-null
            ts_self
                .lines()
                .map(|line| {
                    line.replace(
                        "keywords_to_get_accurate_results_from_web_search: Array<string> | null,",
                        "keywords_to_get_accurate_results_from_web_search: Array<string>,",
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
                .trim()
                .to_string()
        } else {
            ts_self
                .lines()
                .map(|line| {
                    line.replace(
                        "keywords_to_get_accurate_results_from_web_search: Array<string> | null, ",
                        "",
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
                .trim()
                .to_string()
        };

        Ok(ts_self)
    }
}
