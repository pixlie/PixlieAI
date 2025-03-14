use crate::engine::node::{NodeItem, NodeLabel};
use crate::engine::{CommonEdgeLabels, Engine, NodeFlags};
use crate::entity::pixlie::LLMResponse;
use crate::error::PiError;
use crate::services::anthropic::Anthropic;
use crate::services::llm::LLM;
use crate::{
    engine::node::{ArcedNodeId, ArcedNodeItem, NodeId, Payload},
    entity::search::SearchTerm,
    error::PiResult,
    services::anthropic::extract_search_terms,
    workspace::WorkspaceCollection,
    ExternalData,
};
use std::{sync::Arc, vec};
use ts_rs::TS;

pub struct Objective;

impl Objective {
    pub fn add_manually(engine: Arc<&Engine>, topic: &String) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::Text(topic.to_string()),
            vec![NodeLabel::AddedByUser, NodeLabel::Objective],
            true,
            None,
        )?;
        Ok(())
    }

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
            Some(external_data) => {}
            None => Objective::request_llm(node, engine)?,
        }
        Ok(())
    }

    fn request_llm(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
        let pixlie_schema = LLMResponse::export_to_string()?;

        if node.labels.contains(&NodeLabel::Objective) {
            match &node.payload {
                Payload::Text(text) => {
                    let engine_request = Anthropic::get_request(&pixlie_schema, text, node.id)?;
                    engine.fetch(engine_request)
                }
                _ => Err(PiError::GraphError(
                    "Expected an Objective node with Payload::Text, got".to_string(),
                )),
            }
        } else {
            Err(PiError::GraphError("Node is not an Objective".to_string()))
        }
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
