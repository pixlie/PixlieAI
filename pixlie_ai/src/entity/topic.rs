use std::{sync::Arc, vec};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{engine::{ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, Payload}, entity::{search::SearchTerm, web::link::Link}, error::{PiError, PiResult}, services::anthropic::extract_search_terms, utils::crud::Crud, workspace::WorkspaceCollection, ExternalData};



#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Topic(pub String);

impl Topic {
    pub fn add_manually(engine: Arc<&Engine>, topic: &String) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::Topic(Topic(topic.to_string())),
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
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
                "Skipping processing Topic node '{}': Antrhropic API key isnt configured yet",
                self.0,
            );
            return Ok(());
        }

        let topic = self.clone();
        let link_node_ids: Vec<ArcedNodeId> = engine.get_node_ids_with_label(&Link::get_label());
        if link_node_ids.len() == 0 {
            debug!(
                "Skipping processing of Topic node '{}': No link nodes present",
                self.0
            );
            return Ok(());
        }
        let mut content = vec![];
        for link_node_id in link_node_ids {
            let link_node = match engine.get_node_by_id(&link_node_id) {
                Some(arced_node) => arced_node,
                None => {
                    debug!(
                        "Skipping processing of Topic node '{}': Link node {} not found",
                        self.0,
                        link_node_id
                    );
                    return Ok(());
                }
            };
            if !link_node.labels.contains(&CommonNodeLabels::AddedByUser.to_string()) {
                // Only process topic for links added by the user for now
                // TODO: Later, we will introduce processing for all links
                // We may want to store the topic's processing time for
                // each link in a HashMap in the Topic structure
                continue;
            }

            match &link_node.payload {
                Payload::Link(link) => {
                    if link.is_fetched {
                        let webpage_node_ids: Vec<ArcedNodeId> = engine.get_node_ids_connected_with_label(
                            &link_node.id,
                            &CommonEdgeLabels::PathOf.to_string(),
                        )?;

                        for webpage_node_id in webpage_node_ids {
                            match engine.get_node_by_id(&webpage_node_id) {
                                Some(webpage_node) => {
                                    match &webpage_node.payload {
                                        Payload::FileHTML(web_page) => {
                                            if web_page.is_scraped {
                                                let child_node_ids: Vec<ArcedNodeId> = engine.get_node_ids_connected_with_label(
                                                    &webpage_node_id,
                                                    &CommonEdgeLabels::ParentOf.to_string()
                                                )?;
                                                for child_node_id in child_node_ids {
                                                    match engine.get_node_by_id(&child_node_id) {
                                                        Some(child_node) => {
                                                            match &child_node.payload {
                                                                Payload::Title(title) => {
                                                                    content.push(
                                                                        ("webpage_title".to_string(), title.0.clone())
                                                                    );
                                                                }
                                                                Payload::Heading(heading) => {
                                                                    content.push(
                                                                        ("webpage_heading".to_string(), heading.0.clone())
                                                                    );
                                                                }
                                                                Payload::Paragraph(paragraph) => {
                                                                    content.push(
                                                                        ("webpage_paragraph".to_string(), paragraph.0.clone())
                                                                    );
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        None => {}
                                                    };
                                                }   
                                            } else {
                                                debug!(
                                                    "Skipping processing of Topic node '{}': WebPage node {} is not scraped yet",
                                                    self.0,
                                                    webpage_node_id
                                                );
                                                return Ok(());
                                            }
                                        }
                                        _ => {}
                                    }
                                },
                                None => {}
                            };
                        }
                    }
                    else {
                        debug!(
                            "Skipping processing of Topic node '{}': Link node {}({}) is not fetched yet",
                            self.0,
                            link_node.id,
                            link.get_full_link()
                        );
                        return Ok(());
                    }
                },
                _ => {},
            };
        }

        if content.len() == 0 {
            debug!(
                "Skipping processing of Topic node '{}': No content found",
                self.0
            );
            return Ok(());
        }

        let search_terms = match extract_search_terms(
            topic.0.clone(),
            &content,
            active_workspace.anthropic_api_key.as_ref().unwrap()
        ) {
            Ok(search_terms) => search_terms,
            Err(err) => {
                error!("Error extracting search terms: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error extracting search terms: {}",
                    err
                )));
            }
        };

        for search_term in search_terms {
            let search_term_node_id = engine.get_or_add_node(
                Payload::SearchTerm(SearchTerm(search_term.search_term.clone())),
                vec![],
                true,
            )?.get_node_id();
            engine.add_connection(
                (node_id.clone(), search_term_node_id.clone()),
                (CommonEdgeLabels::Suggests.to_string(), CommonEdgeLabels::SuggestedFor.to_string()),
            )?;
        }

        engine.update_node(&node_id, Payload::Topic(topic))?;
        Ok(())
    }
}