use std::{sync::Arc, vec};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{engine::{ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeFlags, NodeId, Payload}, entity::search::SearchTerm, error::PiResult, services::anthropic::extract_search_terms, utils::crud::Crud, workspace::WorkspaceCollection, ExternalData};



#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Topic(pub String);

impl Topic {
    pub fn add_manually(engine: Arc<&Engine>, topic: &String) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::Topic(Topic(topic.to_string())),
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
            None,
        )?;
        Ok(())
    }
    pub fn find_existing(engine: Arc<&Engine>, topic: &String) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        let existing_node_ids: Vec<ArcedNodeId> = engine.get_node_ids_with_label(&Topic::get_label());
        for node_id in existing_node_ids {
            match engine.get_node_by_id(&node_id) {
                Some(node) => match &node.payload {
                    Payload::Topic(payload) => {
                        if &payload.0 == topic {
                            return Ok(Some((node, node_id)));
                        }
                    }
                    _ => {}
                },
                None => {}
            }
        }
        Ok(None)
    }
}

impl Node for Topic {
    fn get_label() -> String {
        "Topic".to_string()
    }

    fn process(
        &self,
        engine: Arc<&Engine>,
        node_id: &NodeId,
        _data_from_previous_request: Option<ExternalData>
    ) -> PiResult<()> {
        let workspaces = WorkspaceCollection::read_list()?;

        // Skip if there are no workspaces yet
        if workspaces.len() == 0 {
            debug!(
                "Skipping processing Topic node '{}': There are no workspaces yet",
                self.0
            );
            return Ok(());
        }

        // TODO: Currently we are using the first workspace
        // Later, we need to change this to use the active workspace
        let active_workspace = &workspaces[0];

        // Skip if there is no API key
        if active_workspace.anthropic_api_key.is_none() {
            debug!(
                "Skipping processing Topic node '{}': Anthropic API key isnt configured yet",
                self.0,
            );
            return Ok(());
        }

        let evaluated_node_ids = engine.get_node_ids_connected_with_label(
            node_id, &CommonEdgeLabels::EvaluatedFor.to_string()
        ).unwrap_or_else(|_| vec![]);

        let content_node_ids = engine.get_node_ids_with_label(&CommonNodeLabels::Content.to_string());

        let unprocessed_content_nodes = content_node_ids.iter().filter_map(|content_node_id| {
            if evaluated_node_ids.contains(content_node_id) {
                // Skip if the content node has already been evaluated
                return None;
            }
            match engine.get_node_by_id(content_node_id) {
                Some(content_node) => {
                    if !content_node.flags.contains(NodeFlags::IS_PROCESSED) {
                        // Skip if the content node has not been processed yet
                        debug!("Content node not procesed: {}", content_node_id);
                        return None;
                    }
                    else {
                        debug!("Content node processed: {}", content_node_id);
                        return Some(content_node);
                    }
                },
                None => {
                    debug!("Could not load content node: {}", content_node_id);
                    return None;
                }
            }
        }).collect::<Vec<ArcedNodeItem>>();

        let labels_of_interest = [
            CommonNodeLabels::BulletPoints.to_string(),
            CommonNodeLabels::Heading.to_string(),
            CommonNodeLabels::Link.to_string(),
            CommonNodeLabels::Paragraph.to_string(),
            CommonNodeLabels::OrderedPoints.to_string(),
            CommonNodeLabels::Title.to_string(),
        ];

        for content_node in unprocessed_content_nodes {
            let partial_content_nodes = match &content_node.payload {
                Payload::FileHTML(web_page) => {
                    match web_page.get_partial_content_nodes(engine.clone(), &content_node.id) {
                        Ok(partial_content_nodes) => partial_content_nodes,
                        Err(err) => {
                            error!(
                                "Skipping search term extraction for topic '{}', Webpage node {}: {}",
                                self.0,
                                content_node.id,
                                err
                            );
                            continue;
                        }
                    }
                },
                _ => {
                    // Skip if the content node is not a webpage
                    continue;
                }
            };
                    
            if partial_content_nodes.len() == 0 {
                // Skip if there is no content in the content node to evaluate
                continue;
            }
            let content = partial_content_nodes.iter().filter_map(|partial_content_node| {
                match &partial_content_node.payload {
                    Payload::Text(text) => {
                        labels_of_interest.iter().find_map(|label| {
                            if partial_content_node.labels.contains(&label) {
                                Some((label.clone(), text.to_string()))
                            } else {
                                None
                            }
                        })
                    },
                    _ => None
                }
            }).collect::<Vec<(String, String)>>();
            if content.len() == 0 {
                // Skip if there is no content in the webpage to evaluate
                continue;
            }
            let search_term_results = match extract_search_terms(
                self.0.clone(),
                &content,
                active_workspace.anthropic_api_key.as_ref().unwrap()
            ) {
                Ok(search_terms) => search_terms,
                Err(err) => {
                    error!("Error extracting search terms: {}", err);
                    continue;
                }
            };
            for search_term_result in search_term_results {
                let search_term_node_id: Option<NodeId> = match engine.get_or_add_node(
                    Payload::SearchTerm(SearchTerm(search_term_result.search_term.clone())),
                    vec![],
                    true,
                    None,
                ) {
                    Ok(node_id) => Some(node_id.get_node_id()),
                    Err(err) => {
                        error!(
                            "Error adding search term {} while processing webpage node {} for topic {}: {}",
                            search_term_result.search_term,
                            content_node.id,
                            self.0,
                            err
                        );
                        None
                    }
                };
                match search_term_node_id {
                    Some(search_term_node_id) => {
                        match engine.add_connection(
                            (node_id.clone(), search_term_node_id.clone()),
                            (CommonEdgeLabels::Suggests.to_string(), CommonEdgeLabels::SuggestedFor.to_string()),
                        ) {
                            Ok(_)=>{},
                            Err(err)=>{
                                error!(
                                    "Error adding connection between Topic node {} and SearchTerm node {}: {}",
                                    node_id,
                                    search_term_node_id,
                                    err
                                );
                            }
                        };
                        match engine.add_connection(
                            (search_term_node_id.clone(), content_node.id.clone()),
                            (CommonEdgeLabels::Suggests.to_string(), CommonEdgeLabels::SuggestedFor.to_string()),
                        ) {
                            Ok(_)=>{},
                            Err(err)=>{
                                error!(
                                    "Error adding connection between SearchTerm node {} and Content node {}: {}",
                                    search_term_node_id,
                                    content_node.id,
                                    err
                                );
                            }
                        }
                    }
                    None => {}
                }
                match engine.add_connection(
                    (node_id.clone(), content_node.id.clone()),
                    (CommonEdgeLabels::EvaluatedFor.to_string(), CommonEdgeLabels::EvaluatedFor.to_string()),
                ) {
                    Ok(_)=>{},
                    Err(err)=>{
                        error!(
                            "Error adding connection between Topic node {} and Content node {}: {}",
                            node_id,
                            content_node.id,
                            err
                        );
                    }
                }
            }

        }
        Ok(())
    }
}