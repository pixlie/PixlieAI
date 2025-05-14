// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use super::EntityName;
use crate::engine::node::NodeItem;
use crate::engine::Engine;
use crate::entity::classifier::ClassifierSettings;
use crate::entity::crawler::CrawlerSettings;
use crate::entity::project_settings::ProjectSettings;
use crate::utils::llm::LLMPrompt;
use crate::{
    error::PiResult,
    utils::llm::{clean_ts_type, LLMSchema},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

// Features that are available in Pixlie for an AI agent
#[derive(Deserialize, TS)]
pub enum Tool {
    Crawler(CrawlerSettings),
    Classifier(ClassifierSettings),
    NamedEntityExtraction(Vec<EntityName>),
}

impl LLMSchema for Tool {
    fn get_schema_for_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        let ts_crawl = CrawlerSettings::get_schema_for_llm(node, engine.clone())?;
        let ts_classify = ClassifierSettings::get_schema_for_llm(node, engine.clone())?;
        let ts_named_entity_extraction = EntityName::get_schema_for_llm(node, engine.clone())?;
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        Ok(format!(
            "{}\n{}\n{}\n{}",
            ts_crawl, ts_classify, ts_named_entity_extraction, ts_self
        ))
    }
}

#[derive(Deserialize, TS)]
pub struct LLMResponse {
    pub short_project_name_with_spaces: String,
    pub tools_needed_to_accomplish_objective: Vec<Tool>,
}

impl LLMSchema for LLMResponse {
    fn get_schema_for_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        let ts_tool = Tool::get_schema_for_llm(node, engine)?;
        let ts_self = clean_ts_type(&Self::export_to_string()?);

        Ok(format!("{}\n{}", ts_tool, ts_self))
    }
}

#[derive(Serialize)]
pub struct ProjectState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_settings: Option<ProjectSettings>,
    pub objective: String,
    // pub features_available: Vec<String>,
}

impl LLMPrompt for ProjectState {
    fn get_prompt(
        &self,
        llm_response_schema: &String,
        _node: &NodeItem,
        _engine: Arc<&Engine>,
    ) -> PiResult<String> {
        Ok(format!(
            r#"I am a software bot and here is my current state:
```json
{}
```

Using the following schema, please respond in JSON with `LLMResponse` to achieve the objective.
```typescript
{}
```
"#,
            serde_json::to_string(self)?,
            llm_response_schema
        ))
    }
}
