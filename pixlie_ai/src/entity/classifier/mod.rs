// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
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

impl LLMSchema for ClassifierSettings {
    fn get_schema_for_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
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
