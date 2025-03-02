use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};

use crate::{engine::{ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, CommonNodeLabels, Engine, ExistingOrNewNodeId, Node, NodeId, Payload}, error::{PiError, PiResult}};

#[derive(Clone, Deserialize, Serialize)]
pub struct TopicLinkSearchTerms {
    processed: bool,
}

impl Node for TopicLinkSearchTerms {
    fn get_label() -> String {
        "TopicLinkSearchTerms".to_string()
    }
}

impl TopicLinkSearchTerms {
    pub fn add(engine: Arc<&Engine>, topic_id: NodeId, link_id: NodeId) {
        // Add a new TopicLinkSearchTerms node
        match TopicLinkSearchTerms::find_existing(&engine, topic_id, link_id) {
            Ok(Some((_, _))) => {
                error!("TopicLinkSearchTerms already exists for topic_id {} and link_id {}", topic_id, link_id);
                return;
            },
            Ok(None) => {
                // Create a new TopicLinkSearchTerms node
                match engine.get_or_add_node(
                    Payload::TopicLinkSearchTerms( TopicLinkSearchTerms {
                        processed: false
                    }),
                    vec![CommonNodeLabels::Action.to_string()],
                    true
                ) {
                    Ok(action_node_id) => {
                        let action_node_id: NodeId = match action_node_id {
                            ExistingOrNewNodeId::Existing(existing_node_id) => {
                                error!(
                                    "Error adding TopicLinkSearchTerms for topic_id {}, link_id {}: Node already exists with ID {}",
                                    topic_id,
                                    link_id,
                                    existing_node_id
                                );
                                return;
                            },
                            ExistingOrNewNodeId::New(new_node_id) => new_node_id
                        };

                        // Connect the new node to the topic and link
                        match engine.add_connection(
                            (topic_id, action_node_id),
                            (
                                CommonEdgeLabels::Ran.to_string(),
                                CommonEdgeLabels::RanBy.to_string()
                            )
                        ) {
                            Ok(_) => {
                                match engine.add_connection(
                                    (link_id, action_node_id),
                                    (
                                        CommonEdgeLabels::Ran.to_string(),
                                        CommonEdgeLabels::RanBy.to_string()
                                    )
                                ) {
                                    Ok(_) => {},
                                    Err(err) => {
                                        // TODO: Delete the newly creaated `Topic -> Action` edge
                                        // & TopicLinkSearchTerms `Action` node for atomicity
                                        // For this, we will require add_connection to return the edge ID
                                        // and a function to delete the edge by ID
                                        // Alternates:
                                        // 1. Implement atomicity by combining the two operations in the add_or_get_node function
                                        //    by accepting a vector of edges to add
                                        // 2. Implement a transaction manager in the engine
                                        error!("Error connecting TopicLinkSearchTerms to link: {}", err);
                                        return;
                                    }
                                }
                            },
                            Err(err) => {
                                // TODO: Delete the newly created TopicLinkSearchTerms node for atomicity
                                // For this, we will require a function to delete a node by ID
                                error!("Error connecting TopicLinkSearchTerms to topic: {}", err);
                                return;
                            }
                        }
                    },
                    Err(err) => {
                        error!("Error adding new TopicLinkSearchTerms: {}", err);
                        return;
                    }
                }
            },
            Err(err) => {
                error!("Error finding existing TopicLinkSearchTerms: {}", err);
                return;
            },
            _ => {}
        }
        // Connect the new node to the topic and link
        // Connect the new node to the actions run by the topic and link
    }

    pub fn find_existing(engine: &Arc<&Engine>, topic_id: NodeId, link_id: NodeId) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        // Check if there is a TopicLinkSearchTerms with edges to the given topic_id and link_id
        // Get actions run by the topic
        let topic_actions = match engine.get_node_ids_connected_with_label(
            &topic_id, &CommonEdgeLabels::Ran.to_string()
        ) {
            Ok(actions) => actions,
            Err(err) => {
                error!("Error getting connected node IDs: {}", err);
                return Err(err);
            }
        };
        // Get actions run by the link
        let link_actions = match engine.get_node_ids_connected_with_label(
            &link_id, &CommonEdgeLabels::Ran.to_string()
        ) {
            Ok(actions) => actions,
            Err(err) => {
                error!("Error getting connected node IDs: {}", err);
                return Err(err);
            }
        };
        // Check if the topic and link have any actions in common
        let common_actions: Vec<&ArcedNodeId> = topic_actions.iter().filter(|&action| link_actions.contains(action)).collect::<Vec<&ArcedNodeId>>();
        // Check if any of the common actions are TopicLinkSearchTerms
        for action_id in common_actions {
            match engine.get_node_by_id(action_id) {
                Some(node) => {
                    if node.labels.contains(&TopicLinkSearchTerms::get_label()) {
                        return Ok(Some((node, action_id.clone())));
                    }
                },
                None => {}
            }
        }
        Ok(None)
    }
}