// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::Engine;
use crate::entity::classifier::ClassifierSettings;
use crate::entity::crawler::CrawlerSettings;
use crate::entity::named_entity::EntityName;
use crate::entity::project_settings::ProjectSettings;
use crate::utils::llm::LLMPrompt;
use crate::{
    error::{PiError, PiResult},
    utils::llm::{clean_ts_type, LLMSchema},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utoipa::ToSchema;

/// Generic enum that can be used by any tool setting to indicate if it's enabled or disabled
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub enum ToolEnabled {
    Yes,
    No,
}

impl Default for ToolEnabled {
    fn default() -> Self {
        ToolEnabled::Yes
    }
}

/// Check if a tool is enabled by looking up nodes in the engine
pub fn is_tool_enabled(engine: Arc<&Engine>, node_label: NodeLabel) -> PiResult<bool> {
    let settings_node_ids = engine.get_node_ids_with_label(&node_label);
    let settings_node = settings_node_ids
        .first()
        .ok_or_else(|| PiError::InternalError(format!("No {:?} nodes found in engine", node_label)))
        .and_then(|id| {
            engine.get_node_by_id(id).ok_or_else(|| {
                PiError::InternalError(format!(
                    "Failed to retrieve {:?} node with id: {}",
                    node_label, id
                ))
            })
        })?;

    is_tool_enabled_from_payload(&settings_node.payload, node_label)
}

/// Macro to check if any tool with is_enabled field is enabled
macro_rules! check_tool_enabled {
    ($payload:expr, $($variant:ident),+) => {
        match $payload {
            $(Payload::$variant(settings) => matches!(settings.is_enabled, ToolEnabled::Yes),)+
            _ => return Err(PiError::InternalError(format!("Unsupported payload type for tool settings check"))),
        }
    };
}

/// Check if a tool is enabled from a payload
pub fn is_tool_enabled_from_payload(payload: &Payload, _node_label: NodeLabel) -> PiResult<bool> {
    let is_enabled = check_tool_enabled!(payload, CrawlerSettings, ClassifierSettings);
    Ok(is_enabled)
}

/// Macro to disable a specific tool
macro_rules! disable_tool {
    ($engine:expr, $node_label:expr, $variant:ident) => {{
        let node_ids = $engine.get_node_ids_with_label(&$node_label);
        for node_id in node_ids {
            if let Some(node) = $engine.get_node_by_id(&node_id) {
                if let Payload::$variant(settings) = &node.payload {
                    // Only update if currently enabled
                    if matches!(settings.is_enabled, ToolEnabled::Yes) {
                        let mut updated_settings = settings.clone();
                        updated_settings.is_enabled = ToolEnabled::No;
                        // Update the node in the engine
                        $engine.update_node(&node_id, Payload::$variant(updated_settings))?;
                    }
                }
            }
        }
        Ok::<(), crate::error::PiError>(())
    }};
}

/// Disable multiple tools by their NodeLabels
pub fn disable_tools_by_labels(engine: Arc<&Engine>, node_labels: Vec<NodeLabel>) -> PiResult<()> {
    for node_label in node_labels {
        match node_label {
            NodeLabel::CrawlerSettings => disable_tool!(engine, node_label, CrawlerSettings)?,
            NodeLabel::ClassifierSettings => disable_tool!(engine, node_label, ClassifierSettings)?,
            _ => {
                return Err(PiError::InternalError(format!(
                    "Tool disabling not supported for node label: {:?}",
                    node_label
                )))
            }
        }
    }
    Ok(())
}

/// Disable a specific tool by its NodeLabel (convenience function for single tool)
pub fn disable_tool_by_label(engine: Arc<&Engine>, node_label: NodeLabel) -> PiResult<()> {
    disable_tools_by_labels(engine, vec![node_label])
}

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
