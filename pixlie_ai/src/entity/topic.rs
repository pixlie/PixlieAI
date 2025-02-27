use std::{sync::Arc, vec};

use chrono::{DateTime, TimeDelta, Utc};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{engine::{CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, Payload}, entity::{search::SearchTerm, web::link::Link}, error::{PiError, PiResult}, services::anthropic::extract_search_terms, utils::crud::Crud, workspace::WorkspaceCollection};



#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Topic {
    pub topic: String,
    pub last_processed: Option<DateTime<Utc>>
}

impl Topic {
    pub fn add_manually(engine: Arc<&Engine>, topic: &String) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::Topic(Self {
                topic: topic.to_string(),
                last_processed: None,
            }),
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
        )?;
        Ok(())
    }
    pub fn find_existing(engine: Arc<&Engine>, topic: &String) -> PiResult<Option<(Topic, NodeId)>> {
        match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => {
                match node_ids_by_label.get(&Self::get_label()) {
                    Some(topic_node_ids) => {
                        for topic_node_id in topic_node_ids {
                            match engine.get_node_by_id(topic_node_id) {
                                Ok(topic_node) => {
                                    match topic_node.payload {
                                        Payload::Topic(t) => {
                                            if t.topic == topic.to_string() {
                                                return Ok(Some((t.clone(), topic_node_id.clone())));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Err(err) => {
                                    error!("Could not read topic_node: {}", err);
                                    return Err(PiError::InternalError(format!(
                                        "Could not read topic_node: {}",
                                        err
                                    )));
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
            Err(err) => {
                error!("Could not read node_ids_by_label: {}", err);
                return Err(PiError::InternalError(format!(
                    "Could not read node_ids_by_label: {}",
                    err
                )));
            }
        }
        Ok(None)
    }
}

impl Node for Topic {
    fn get_label() -> String {
        "Topic".to_string()
    }

    fn process(&self, engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<()> {
        let workspaces = WorkspaceCollection::read_list()?;

        // Skip if there are no workspaces yet
        if workspaces.len() == 0 {
            debug!(
                "Skipping processing Topic node '{}': There are no workspaces yet",
                self.topic
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
                self.topic,
            );
            return Ok(());
        }
        const WAIT_TIME: TimeDelta = TimeDelta::hours(5);
        // If the node was processed within WAIT_TIME, skip processing
        if !self.last_processed.is_none_or(|p| p > Utc::now() - WAIT_TIME) {
            debug!(
                "Skipping processing of Topic node '{}': Isn't still {} since the last processing",
                self.topic,
                WAIT_TIME.to_string(),
            );
            return Ok(());
        }

        let topic = self.clone();
        let mut content: Vec<String> = vec![];
        match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => {
                match node_ids_by_label.get(&Link::get_label().to_string()) {
                    Some(link_node_ids) => {
                        if link_node_ids.len() == 0 {
                            debug!(
                                "Skipping processing of Topic node '{}': No link nodes present",
                                self.topic
                            );
                            return Ok(());
                        }
                        for link_node_id in link_node_ids {
                            match engine.get_node_by_id(link_node_id) {
                                Ok(link_node) => {
                                    if !link_node.labels.contains(&CommonNodeLabels::AddedByUser.to_string()) {
                                        // Only process topic for links added by the user for now
                                        // TODO: Later, we will introduce processing for all links
                                        // We may want to store the topic's processing time for
                                        // each link in a HashMap in the Topic structure
                                        continue;
                                    }
                                    match link_node.clone().payload {
                                        Payload::Link(link) => {
                                            if link.is_fetched {
                                                let web_page_node_ids 
                                                = match engine.get_node_ids_connected_with_label(
                                                    link_node_id,
                                                    &CommonEdgeLabels::PathOf.to_string(),
                                                ) {
                                                    Ok(web_page_node_ids) => web_page_node_ids,
                                                    Err(err) => {
                                                        error!(
                                                            "Error reading WebPage nodes connected to link {}({}):{}",
                                                            link_node.id,
                                                            link.get_full_link(),
                                                            err
                                                        );
                                                        return Err(PiError::InternalError(format!(
                                                            "Error reading WebPage nodes connected to link {}({}):{}",
                                                            link_node.id,
                                                            link.get_full_link(),
                                                            err
                                                        )));
                                                    }
                                                };

                                                for web_page_node_id in web_page_node_ids {
                                                    match engine.get_node_by_id(&web_page_node_id) {
                                                        Ok(html_node) => {
                                                            match html_node.payload {
                                                                Payload::FileHTML(web_page) => {
                                                                    if web_page.is_scraped {
                                                                        match engine.get_node_ids_connected_with_label(
                                                                            &web_page_node_id,
                                                                            &CommonEdgeLabels::ParentOf.to_string()
                                                                        ) {
                                                                            Ok(child_node_ids) => {
                                                                                for child_node_id in child_node_ids {
                                                                                    match engine.get_node_by_id(&child_node_id) {
                                                                                        Ok(child_node) => {
                                                                                            match child_node.payload {
                                                                                                Payload::Title(title) => {
                                                                                                    content.push(
                                                                                                        format!("<webpage_title>{}</webpage_title>", title.0)
                                                                                                    );
                                                                                                }
                                                                                                Payload::Heading(heading) => {
                                                                                                    content.push(
                                                                                                        format!("<heading>{}</heading>", heading.0)
                                                                                                    );
                                                                                                }
                                                                                                Payload::Paragraph(paragraph) => {
                                                                                                    content.push(
                                                                                                        format!("<paragraph>{}</paragraph>", paragraph.0)
                                                                                                    );
                                                                                                }
                                                                                                _ => {}
                                                                                            }
                                                                                        }
                                                                                        Err(err) => {
                                                                                            error!(
                                                                                                "Error reading content node {} for WebPage node {}: {}",
                                                                                                child_node_id,
                                                                                                web_page_node_id,
                                                                                                err
                                                                                            );
                                                                                            return Err(PiError::InternalError(format!(
                                                                                                "Error reading content node {} for WebPage node {}: {}",
                                                                                                child_node_id,
                                                                                                web_page_node_id,
                                                                                                err
                                                                                            )));
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(err) => {
                                                                                error!(
                                                                                    "Error reading child nodes of WebPage node {}: {}",
                                                                                    web_page_node_id,
                                                                                    err
                                                                                );
                                                                                return Err(PiError::InternalError(format!(
                                                                                    "Error reading child nodes of WebPage node {}: {}",
                                                                                    web_page_node_id,
                                                                                    err
                                                                                )));
                                                                            }
                                                                        }
                                                                    } else {
                                                                        debug!(
                                                                            "Skipping processing of Topic node '{}': WebPage node {} is not scraped yet",
                                                                            self.topic,
                                                                            web_page_node_id
                                                                        );
                                                                        return Ok(());
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        Err(err) => {
                                                            error!(
                                                                "Error reading WebPage node {}: {}",
                                                                web_page_node_id,
                                                                err
                                                            );
                                                            return Err(PiError::InternalError(format!(
                                                                "Error reading WebPage node {}: {}",
                                                                web_page_node_id,
                                                                err
                                                            )));
                                                        }
                                                    }
                                                }
                                            }
                                            else {
                                                debug!(
                                                    "Skipping processing of Topic node '{}': Link node {}({}) is not fetched yet",
                                                    self.topic,
                                                    link_node.id,
                                                    link.get_full_link()
                                                );
                                                return Ok(());
                                            }
                                        },
                                        _ => {},
                                    };
                                }
                                Err(err) => {
                                    error!("Error reading Link node {}: {}", link_node_id, err);
                                    return Err(PiError::InternalError(format!(
                                        "Error reading Link node {}: {}",
                                        link_node_id,
                                        err
                                    )));
                                }
                            }
                        }
                    }
                    None => {
                        debug!(
                            "Skipping processing of Topic node '{}': No link nodes found",
                            self.topic
                        );
                        return Ok(());
                    },
                    
                }
            }
            Err(err) => {
                error!("Error reading node_ids_by_label map: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error reading node_ids_by_label map: {}",
                    err
                )));
            }
        };

        if content.len() == 0 {
            debug!(
                "Skipping processing of Topic node '{}': No content found",
                self.topic
            );
            return Ok(());
        }

        let search_terms = match extract_search_terms(
            topic.topic.clone(),
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

        if search_terms[0].0 != "Topic" || search_terms[0].1 != "SearchTerm" || search_terms[0].2 != "Match" {
            debug!(
                "Skipping processing of Topic node '{}': Invalid search terms response",
                self.topic
            );
            return Ok(());
        }

        for search_term in search_terms[1..].to_vec() {
            let (_topic, search_term, _match_type) = search_term;

            let search_term_node_id = engine.get_or_add_node(
                Payload::SearchTerm(SearchTerm(search_term.clone())),
                vec![],
                true,
            )?.get_node_id();
            println!("Search term node id: {}", search_term_node_id.to_string());
            println!("Topic node id: {}", node_id.to_string());
            engine.add_connection(
                (node_id.clone(), search_term_node_id.clone()),
                (CommonEdgeLabels::Suggests.to_string(), CommonEdgeLabels::SuggestedFor.to_string()),
            );
        }

        engine.update_node(&node_id, Payload::Topic(Topic {
            last_processed: Some(Utc::now()),
            ..self.clone()
        }));
        Ok(())
    }
}