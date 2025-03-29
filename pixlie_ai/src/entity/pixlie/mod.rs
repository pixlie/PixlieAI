use serde::Deserialize;
use std::sync::Arc;
use ts_rs::TS;

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine};
use crate::entity::project_settings::ProjectSettings;
use crate::{
    error::PiResult,
    utils::llm_schema::{clean_ts_type, LLMSchema},
};
// #[derive(Deserialize, TS)]
// #[ts(export)]
// pub enum NamedEntity {
//     BlogPost,
//     JobPost,
//     People,
//     Company,
//     Event,
//     Date,
//     Location,
//     PressRelease,
//     Product,
//     Currency,
//     Price,
// }

#[derive(Deserialize, TS)]
pub enum ContinueCrawl {
    IfContentHasKeywords(Vec<String>),
    // IfContentClassifiedAs(Vec<String>),
    // IfNamedEntityExtracted(Vec<NamedEntity>),
}

impl LLMSchema for ContinueCrawl {}

#[derive(Deserialize, TS)]
pub struct CrawlSpecification {
    pub web_search_keywords_for_objective: Option<Vec<String>>,
    pub conditions_to_continue_crawling: Option<ContinueCrawl>,
}

impl LLMSchema for CrawlSpecification {
    fn get_schema_for_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        let ts_continue_crawl = ContinueCrawl::get_schema_for_llm(node, engine.clone())?;
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        // If the payload is for an Objective and there is a ProjectSetting connected to it,
        // then we check if there is a setting for starting links
        // If starting links are to be manually provided, then we remove the web_search_keywords_for_objective field
        let mut has_web_search_keywords_for_objective: bool = true;
        match node.payload {
            Payload::Text(_) => {
                if node.labels.contains(&NodeLabel::Objective) {
                    let project_settings_node_id = engine
                        .get_node_ids_connected_with_label(&node.id, &EdgeLabel::BelongsTo)?;
                    if project_settings_node_id.len() > 0 {
                        match engine.get_node_by_id(&project_settings_node_id[0]) {
                            Some(project_settings_node) => match &project_settings_node.payload {
                                Payload::ProjectSettings(project_settings) => {
                                    if project_settings.has_user_specified_starting_links {
                                        has_web_search_keywords_for_objective = false;
                                    }
                                }
                                _ => {}
                            },
                            None => {}
                        }
                    }
                }
            }
            _ => {}
        };

        let ts_self = if has_web_search_keywords_for_objective {
            // Change the field web_search_keywords_for_objective to non-null
            ts_self
                .lines()
                .map(|line| {
                    line.replace(
                        "web_search_keywords_for_objective: Array<string> | null",
                        "web_search_keywords_for_objective: Array<string>",
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
                        "web_search_keywords_for_objective: Array<string> | null, ",
                        "",
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
                .trim()
                .to_string()
        };

        Ok(format!("{}\n{}", ts_continue_crawl, ts_self))
    }
}

// Features that are available in Pixlie for an AI agent
#[derive(Deserialize, TS)]
pub enum Tool {
    Crawler(CrawlSpecification),
    // NamedEntityExtraction(Vec<NamedEntity>),
}

impl LLMSchema for Tool {
    fn get_schema_for_llm(_node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        let ts_crawl = CrawlSpecification::get_schema_for_llm(_node, engine)?;
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        Ok(format!("{}\n{}", ts_crawl, ts_self))
    }
}

#[derive(Deserialize, TS)]
pub struct LLMResponse {
    pub short_project_name_with_spaces: String,
    pub tools_needed_to_accomplish_objective: Vec<Tool>,
}

impl LLMSchema for LLMResponse {
    fn get_schema_for_llm(_node: &NodeItem, _engine: Arc<&Engine>) -> PiResult<String> {
        let ts_tool = clean_ts_type(&Tool::get_schema_for_llm(_node, _engine)?);
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        Ok(format!("{}\n{}", ts_tool, ts_self))
    }
}
