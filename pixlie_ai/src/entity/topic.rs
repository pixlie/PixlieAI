use std::{sync::Arc, vec};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{engine::{ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, Payload}, entity::{search::SearchTerm, web::link::Link}, error::PiResult, services::anthropic::extract_search_terms, utils::crud::Crud, workspace::WorkspaceCollection, ExternalData};

use super::web::web_page::WebPage;



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

        let webpage_node_ids = engine.get_node_ids_with_label(&WebPage::get_label());
        if webpage_node_ids.len() == 0 {
            // Skip if there are no webpages in the graph
            return Ok(());
        }

        let unprocessed_webpages: Vec<(ArcedNodeId, ArcedNodeId)> = webpage_node_ids.iter().filter_map(|webpage_node_id| {
            // Check if this webpage has already been processed for this topic
            let is_already_processed = match engine.get_node_ids_connected_with_label(
                node_id, &CommonEdgeLabels::EvaluatedFor.to_string()
            ) {
                Ok(evaluated_link_ids) => evaluated_link_ids.iter().any(|link_id| {
                    // Check if this link is connected to the current webpage
                    match engine.get_node_ids_connected_with_label(
                        &webpage_node_id, &CommonEdgeLabels::ContentOf.to_string()
                    ) {
                        Ok(webpage_link_ids) => webpage_link_ids.contains(link_id),
                        Err(_) => false
                    }
                }),
                Err(_) => false
            };

            if is_already_processed {
                return None;
            }

            // Find the link connected to this webpage
            match engine.get_node_ids_connected_with_label(
                &webpage_node_id, &CommonEdgeLabels::ContentOf.to_string()
            ) {
                Ok(link_ids) => {
                    if link_ids.is_empty() {
                        error!(
                            "Skipping search term extraction for topic '{}', Webpage node {}: No link connected to web page",
                            self.0,
                            webpage_node_id
                        );
                        return None;
                    }
                    Some((webpage_node_id.clone(), link_ids[0].clone()))
                },
                Err(err) => {
                    error!(
                        "Skipping search term extraction for topic '{}', Webpage node {}: {}",
                        self.0,
                        webpage_node_id,
                        err
                    );
                    return None;
                }
            }
        }).collect::<Vec<(ArcedNodeId, ArcedNodeId)>>();

        if unprocessed_webpages.len() > 0 {
            info!(
                "Extracting search terms for {} unprocessed Web Pages for topic '{}'",
                unprocessed_webpages.len(),
                self.0
            );
        }

        for (webpage_node_id, link_node_id) in unprocessed_webpages {
            match engine.get_node_by_id(&webpage_node_id) {
                Some(webpage_node) => {
                    match &webpage_node.payload {
                        Payload::FileHTML(web_page) => {
                            if web_page.is_scraped {
                                debug!(
                                    "Processing Topic node '{}': Extracting search terms from Webpage node {}",
                                    self.0,
                                    webpage_node_id
                                );
                                // Only process content of scraped webpages
                                let partial_content_node_ids: Vec<ArcedNodeId>
                                = match engine.get_node_ids_connected_with_label(
                                    &webpage_node_id,
                                    &CommonEdgeLabels::ParentOf.to_string()
                                ) {
                                    Ok(node_ids) => node_ids,
                                    Err(err) => {
                                        // Skip if there was an error in accessing any partial content node
                                        debug!(
                                            "Skipping search term extraction for topic '{}', Webpage node {}: {}",
                                            self.0,
                                            webpage_node_id,
                                            err
                                        );
                                        continue;
                                    }
                                };
                                let content = partial_content_node_ids.iter().filter_map(|node_id| {
                                    match engine.get_node_by_id(node_id) {
                                        Some(partial_content_node) => {
                                            match &partial_content_node.payload {
                                                Payload::Title(title) => {
                                                    Some(("webpage_title".to_string(), title.0.clone()))
                                                }
                                                Payload::Heading(heading) => {
                                                    Some(("webpage_heading".to_string(), heading.0.clone()))
                                                }
                                                Payload::Paragraph(paragraph) => {
                                                    Some(("webpage_paragraph".to_string(), paragraph.0.clone()))
                                                }
                                                _ => None
                                            }
                                        }
                                        None => None
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
                                            // TODO: Need to determine how to handle instances of search terms
                                            // failing to be saved if new / retrieved if existing
                                            error!(
                                                "Error adding search term {} while processing webpage node {} for topic {}: {}",
                                                search_term_result.search_term,
                                                webpage_node_id,
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
                                                (search_term_node_id.clone(), *link_node_id.clone()),
                                                (CommonEdgeLabels::Suggests.to_string(), CommonEdgeLabels::SuggestedFor.to_string()),
                                            ) {
                                                Ok(_)=>{},
                                                Err(err)=>{
                                                    error!(
                                                        "Error adding connection between SearchTerm node {} and Link node {}: {}",
                                                        search_term_node_id,
                                                        link_node_id,
                                                        err
                                                    );
                                                }
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                match engine.add_connection(
                                    (node_id.clone(), *link_node_id.clone()),
                                    (CommonEdgeLabels::EvaluatedFor.to_string(), CommonEdgeLabels::EvaluatedFor.to_string()),
                                ) {
                                    Ok(_)=>{},
                                    Err(err)=>{
                                        error!(
                                            "Error adding connection between Topic node {} and Link node {}: {}",
                                            node_id,
                                            link_node_id,
                                            err
                                        );
                                    }
                                    
                                }
                            }
                        }
                        _ => {}
                    }
                },
                None => {}
            };
        }
        Ok(())
    }
}