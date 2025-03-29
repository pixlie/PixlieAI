use crate::engine::node::{NodeItem, NodeLabel};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::entity::pixlie::{ContinueCrawl, LLMResponse, Tool};
use crate::entity::text::Text;
use crate::error::PiError;
use crate::projects::{Project, ProjectCollection};
use crate::services::anthropic::Anthropic;
use crate::services::llm::LLM;
use crate::utils::crud::Crud;
use crate::utils::llm_schema::LLMSchema;
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
                            Tool::Crawler(crawl) => {
                                match crawl.web_search_keywords_for_objective {
                                    Some(web_search_keywords) => {
                                        for web_search_keywords in web_search_keywords {
                                            let web_search_node_id = Text::add(
                                                engine.clone(),
                                                &web_search_keywords,
                                                vec![NodeLabel::AddedByAI, NodeLabel::WebSearch],
                                            )?;
                                            engine.add_connection(
                                                (node.id, web_search_node_id),
                                                (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                            )?;
                                        }
                                    }
                                    None => {}
                                }
                                if let Some(continue_crawl) = crawl.conditions_to_continue_crawling
                                {
                                    match continue_crawl {
                                        ContinueCrawl::IfContentHasKeywords(keywords) => {
                                            for crawl_condition in keywords {
                                                let crawl_condition_node_id = Text::add(
                                                    engine.clone(),
                                                    &crawl_condition,
                                                    vec![
                                                        NodeLabel::AddedByAI,
                                                        NodeLabel::CrawlCondition,
                                                    ],
                                                )?;
                                                engine.add_connection(
                                                    (node.id, crawl_condition_node_id),
                                                    (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                                )?;
                                            }
                                        }
                                    };
                                }
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
        let pixlie_schema = Self::get_llm_response_schema(node, engine.clone())?;

        if node.labels.contains(&NodeLabel::Objective) {
            match &node.payload {
                Payload::Text(text) => {
                    let engine_request = Anthropic::get_request(&pixlie_schema, text, node.id)?;
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
        Ok(Anthropic::parse_response(response)?)
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
    fn test_llm_schema_for_crawl_without_starting_links() {
        let test_engine = get_test_engine();
        let arced_test_engine = Arc::new(&test_engine);
        let objective =
            "Track software engineeing jobs that need full-stack Python and TypeScript skills"
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
        assert_eq!(
            llm_schema,
            r#"type ContinueCrawl = { "IfContentHasKeywords": Array<string> };
type CrawlSpecification = { web_search_keywords_for_objective: Array<string>, conditions_to_continue_crawling: ContinueCrawl | null, };
type Tool = { "Crawler": CrawlSpecification };
type LLMResponse = { short_project_name_with_spaces: string, tools_needed_to_accomplish_objective: Array<Tool>, };"#
        )
    }

    #[test]
    fn test_llm_schema_for_crawl_with_starting_links() {
        // We create a project with settings to specify starting links manually
        let test_engine = get_test_engine();
        let arced_test_engine = Arc::new(&test_engine);
        let project_settings_node_id = arced_test_engine
            .get_or_add_node(
                Payload::ProjectSettings(ProjectSettings {
                    has_user_specified_starting_links: true,
                }),
                vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
                true,
                None,
            )
            .unwrap()
            .get_node_id();
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
            .unwrap()
            .get_node_id();

        // We need to add edge between ProjectSettings and Objective
        arced_test_engine
            .add_connection(
                (project_settings_node_id, objective_node_id),
                (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
            )
            .unwrap();

        // We specify a few starting links, and connect them to the ProjectSettings
        let starting_links = [
            "https://www.monster.com",
            "https://www.indeed.com/",
            "https://www.remoteok.com/",
        ];
        for link in starting_links {
            let link_node_id = Link::add(
                arced_test_engine.clone(),
                &link.to_string(),
                vec![NodeLabel::AddedByUser, NodeLabel::Link],
                vec![],
                true,
            )
            .unwrap();
            arced_test_engine
                .add_connection(
                    (project_settings_node_id, link_node_id),
                    (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
                )
                .unwrap();
        }

        // Since we specified starting links, our LLM schema for Objective should not
        // contain the web_search_keywords_for_objective field
        let objective_node = arced_test_engine
            .get_node_by_id(&objective_node_id)
            .unwrap();

        let llm_schema =
            Objective::get_llm_response_schema(&*objective_node, arced_test_engine).unwrap();
        assert_eq!(
            llm_schema,
            r#"type ContinueCrawl = { "IfContentHasKeywords": Array<string> };
type CrawlSpecification = { conditions_to_continue_crawling: ContinueCrawl | null, };
type Tool = { "Crawler": CrawlSpecification };
type LLMResponse = { short_project_name_with_spaces: string, tools_needed_to_accomplish_objective: Array<Tool>, };"#
        )
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
