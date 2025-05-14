// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeItem, NodeLabel};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::entity::pixlie::{LLMResponse, ProjectState, Tool};
use crate::entity::project_settings::ProjectSettings;
use crate::error::PiError;
use crate::projects::{Project, ProjectCollection};
use crate::services::anthropic::Anthropic;
use crate::utils::crud::Crud;
use crate::utils::llm::LLMSchema;
use crate::utils::llm::{LLMPrompt, LLMProvider};
use crate::{
    engine::node::{ArcedNodeId, ArcedNodeItem, Payload},
    error::PiResult,
    ExternalData,
};
use std::{sync::Arc, vec};

pub struct Objective;

impl Objective {
    pub fn find_existing(
        engine: Arc<&Engine>,
        topic: &String,
    ) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        let existing_node_ids: Vec<ArcedNodeId> =
            engine.get_node_ids_with_label(&NodeLabel::Objective);
        for node_id in existing_node_ids {
            match engine.get_node_by_id(&node_id) {
                Some(node) => {
                    if node.labels.contains(&NodeLabel::Objective) {
                        match &node.payload {
                            Payload::Text(payload) => {
                                if payload == topic {
                                    return Ok(Some((node, node_id)));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None => {}
            }
        }
        Ok(None)
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
                    let humanized_short_project_name =
                        parsed_response.short_project_name_with_spaces;
                    let project = ProjectCollection::read_item(&engine.get_project_id())?;
                    ProjectCollection::update(
                        &engine.get_project_id(),
                        Project {
                            name: Some(humanized_short_project_name),
                            ..project
                        },
                    )?;

                    for feature in parsed_response.tools_needed_to_accomplish_objective {
                        match feature {
                            Tool::Crawler(crawler_settings) => {
                                let crawler_settings_node_id = engine
                                    .get_or_add_node(
                                        Payload::CrawlerSettings(crawler_settings.clone()),
                                        vec![NodeLabel::AddedByAI, NodeLabel::CrawlerSettings],
                                        true,
                                        None,
                                    )?
                                    .get_node_id();

                                engine.add_connection(
                                    (node.id, crawler_settings_node_id),
                                    (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                )?;

                                match crawler_settings
                                    .keywords_to_search_the_web_to_get_starting_urls
                                {
                                    Some(search_terms) => {
                                        // Save the search term as a WebSearch node so they will be processed
                                        for search_term in search_terms {
                                            let search_term_node_id = engine
                                                .get_or_add_node(
                                                    Payload::Text(search_term.to_string()),
                                                    vec![
                                                        NodeLabel::AddedByAI,
                                                        NodeLabel::WebSearch,
                                                    ],
                                                    true,
                                                    None,
                                                )?
                                                .get_node_id();

                                            engine.add_connection(
                                                (node.id, search_term_node_id),
                                                (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                            )?;
                                        }
                                    }
                                    None => {}
                                }
                            }
                            Tool::Classifier(classifier_settings) => {
                                let classifier_settings_node_id = engine
                                    .get_or_add_node(
                                        Payload::ClassifierSettings(classifier_settings.clone()),
                                        vec![NodeLabel::AddedByAI, NodeLabel::ClassifierSettings],
                                        true,
                                        None,
                                    )?
                                    .get_node_id();

                                engine.add_connection(
                                    (node.id, classifier_settings_node_id),
                                    (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                )?;
                            }
                            Tool::NamedEntityExtraction(named_entity_list) => {
                                let named_entity_extraction_settings_node_id = engine
                                    .get_or_add_node(
                                        Payload::NamedEntitiesToExtract(named_entity_list),
                                        vec![
                                            NodeLabel::AddedByAI,
                                            NodeLabel::NamedEntitiesToExtract,
                                        ],
                                        true,
                                        None,
                                    )?
                                    .get_node_id();

                                engine.add_connection(
                                    (node.id, named_entity_extraction_settings_node_id),
                                    (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                )?;
                            }
                        }
                    }
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(_error) => {}
            },
            None => Objective::request_llm(node, engine)?,
        }
        Ok(())
    }

    fn get_llm_response_schema(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<String> {
        LLMResponse::get_schema_for_llm(node, engine)
    }

    fn request_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
        if node.labels.contains(&NodeLabel::Objective) {
            match &node.payload {
                Payload::Text(text) => {
                    let project_settings: Option<ProjectSettings> = {
                        let related_node_ids = engine
                            .get_node_ids_connected_with_label(&node.id, &EdgeLabel::RelatedTo)?;
                        related_node_ids.iter().find_map(|related_node_id| {
                            match engine.get_node_by_id(related_node_id) {
                                Some(node) => match &node.payload {
                                    Payload::ProjectSettings(project_settings) => {
                                        Some(project_settings.clone())
                                    }
                                    _ => None,
                                },
                                None => None,
                            }
                        })
                    };
                    let project_state = ProjectState {
                        project_settings,
                        objective: text.clone(),
                    };
                    let llm_prompt = project_state.get_prompt(
                        &Self::get_llm_response_schema(&node, engine.clone())?,
                        node,
                        engine.clone(),
                    )?;
                    let engine_request = Anthropic::get_request(&llm_prompt, node.id)?;
                    engine.fetch_api(engine_request)
                }
                _ => Err(PiError::GraphError(
                    "Expected an Objective node with Payload::Text, got".to_string(),
                )),
            }
        } else {
            Err(PiError::GraphError("Node is not an Objective".to_string()))
        }
    }

    fn parse_llm_response(response: &str) -> PiResult<LLMResponse> {
        Ok(Anthropic::parse_response::<LLMResponse>(response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::engine::get_test_engine,
        entity::{project_settings::ProjectSettings, web::link::Link},
    };

    #[test]
    fn test_llm_prompt_for_objective() {
        let test_engine = get_test_engine();
        let arced_test_engine = Arc::new(&test_engine);
        let objective =
            "Track software engineering jobs that need full-stack Python and TypeScript skills"
                .to_string();
        let objective_node_id = arced_test_engine
            .get_or_add_node(
                Payload::Text(objective.clone()),
                vec![NodeLabel::AddedByUser, NodeLabel::Objective],
                true,
                None,
            )
            .unwrap();

        let objective_node = arced_test_engine
            .get_node_by_id(&objective_node_id.get_node_id())
            .unwrap();

        let llm_schema =
            Objective::get_llm_response_schema(&*objective_node, arced_test_engine).unwrap();
        let expected_schema = r#"type CrawlerSettings = { keywords_to_search_the_web_to_get_starting_urls: Array<string>, crawl_link_if_anchor_text_has_any_of_these_keywords: Array<string> | null, };
        type ClassifierSettings = { prompt_to_classify_content_as_relevant_to_objective_or_not: string, };
        type EntityName = "Person" | "Organization" | "Date" | "Place";
        type Tool = { "Crawler": CrawlerSettings } | { "Classifier": ClassifierSettings } | { "NamedEntityExtraction": Array<EntityName> };
        type LLMResponse = { short_project_name_with_spaces: string, tools_needed_to_accomplish_objective: Array<Tool>, };"#;
        assert_eq!(
            llm_schema.split_whitespace().collect::<Vec<_>>().join(" "),
            expected_schema
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

// pub fn process(
//     node: &NodeItem,
//     engine: Arc<&Engine>,
//     _data_from_previous_request: Option<ExternalData>,
// ) -> PiResult<()> {
//     let default_workspace = WorkspaceCollection::get_default()?;
//
//     // Skip if there is no API key
//     if default_workspace.anthropic_api_key.is_none() {
//         debug!("Default workspace does not have Anthropic API key",);
//         return Err(PiError::InternalError(
//             "Default workspace does not have Anthropic API key".to_string(),
//         ));
//     }
//
//     let evaluated_node_ids = engine
//         .get_node_ids_connected_with_label(
//             &node.id,
//             &CommonEdgeLabels::EvaluatedFor.to_string(),
//         )
//         .unwrap_or_else(|_| vec![]);
//
//     let content_node_ids = engine.get_node_ids_with_label(&NodeLabel::Content);
//
//     let unprocessed_content_nodes = content_node_ids
//         .iter()
//         .filter_map(|content_node_id| {
//             if evaluated_node_ids.contains(content_node_id) {
//                 // Skip if the content node has already been evaluated
//                 return None;
//             }
//             match engine.get_node_by_id(content_node_id) {
//                 Some(content_node) => {
//                     return if !content_node.flags.contains(NodeFlags::IS_PROCESSED) {
//                         // Skip if the content node has not been processed yet
//                         None
//                     } else {
//                         Some(content_node)
//                     };
//                 }
//                 None => {
//                     return None;
//                 }
//             }
//         })
//         .collect::<Vec<ArcedNodeItem>>();
//
//     let labels_of_interest = [
//         NodeLabel::UnorderedPoints,
//         NodeLabel::Heading,
//         NodeLabel::Paragraph,
//         NodeLabel::OrderedPoints,
//         NodeLabel::Title,
//     ];
//
//     for content_node in unprocessed_content_nodes {
//         let partial_content_nodes = match &content_node.payload {
//             Payload::FileHTML(web_page) => {
//                 match web_page.get_partial_content_nodes(engine.clone(), &content_node.id) {
//                     Ok(partial_content_nodes) => partial_content_nodes,
//                     Err(err) => {
//                         error!(
//                             "Skipping search term extraction for topic '{}', Webpage node {}: {}",
//                             self.0, content_node.id, err
//                         );
//                         continue;
//                     }
//                 }
//             }
//             _ => {
//                 // Skip if the content node is not a webpage
//                 continue;
//             }
//         };
//
//         if partial_content_nodes.len() == 0 {
//             // Skip if there is no content in the content node to evaluate
//             continue;
//         }
//         let content = partial_content_nodes
//             .iter()
//             .filter_map(|partial_content_node| match &partial_content_node.payload {
//                 Payload::Text(text) => labels_of_interest.iter().find_map(|label| {
//                     if partial_content_node.labels.contains(&label) {
//                         Some((label.clone(), text.to_string()))
//                     } else {
//                         None
//                     }
//                 }),
//                 _ => None,
//             })
//             .collect::<Vec<(String, String)>>();
//         if content.len() == 0 {
//             // Skip if there is no content in the webpage to evaluate
//             continue;
//         }
//         let search_term_results = match extract_search_terms(
//             self.0.clone(),
//             &content,
//             default_workspace.anthropic_api_key.as_ref().unwrap(),
//         ) {
//             Ok(search_terms) => search_terms,
//             Err(err) => {
//                 error!("Error extracting search terms: {}", err);
//                 continue;
//             }
//         };
//         for search_term in search_term_results {
//             let search_term_node_id: Option<NodeId> = match engine.get_or_add_node(
//                 Payload::SearchTerm(SearchTerm(search_term.0.to_string())),
//                 vec![],
//                 true,
//                 None,
//             ) {
//                 Ok(node_id) => Some(node_id.get_node_id()),
//                 Err(err) => {
//                     error!(
//                             "Error adding search term {} while processing webpage node {} for topic {}: {}",
//                             search_term.0.to_string(),
//                             content_node.id,
//                             self.0,
//                             err
//                         );
//                     None
//                 }
//             };
//             match search_term_node_id {
//                 Some(search_term_node_id) => {
//                     match engine.add_connection(
//                         (node_id.clone(), search_term_node_id.clone()),
//                         (
//                             CommonEdgeLabels::Suggests.to_string(),
//                             CommonEdgeLabels::SuggestedFor.to_string(),
//                         ),
//                     ) {
//                         Ok(_) => {}
//                         Err(err) => {
//                             error!(
//                                     "Error adding connection between Topic node {} and SearchTerm node {}: {}",
//                                     node_id,
//                                     search_term_node_id,
//                                     err
//                                 );
//                         }
//                     };
//                     match engine.add_connection(
//                         (search_term_node_id.clone(), content_node.id.clone()),
//                         (
//                             CommonEdgeLabels::Suggests.to_string(),
//                             CommonEdgeLabels::SuggestedFor.to_string(),
//                         ),
//                     ) {
//                         Ok(_) => {}
//                         Err(err) => {
//                             error!(
//                                     "Error adding connection between SearchTerm node {} and Content node {}: {}",
//                                     search_term_node_id,
//                                     content_node.id,
//                                     err
//                                 );
//                         }
//                     }
//                 }
//                 None => {}
//             }
//             match engine.add_connection(
//                 (node_id.clone(), content_node.id.clone()),
//                 (
//                     CommonEdgeLabels::EvaluatedFor.to_string(),
//                     CommonEdgeLabels::EvaluatedFor.to_string(),
//                 ),
//             ) {
//                 Ok(_) => {}
//                 Err(err) => {
//                     error!(
//                         "Error adding connection between Topic node {} and Content node {}: {}",
//                         node_id, content_node.id, err
//                     );
//                 }
//             }
//         }
//     }
//     Ok(())
// }
