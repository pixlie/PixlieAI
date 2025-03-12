use crate::engine::node::{ArcedNodeItem, NodeId, NodeItem, NodeLabel, Payload};
use crate::engine::{CommonEdgeLabels, Engine, NodeFlags};
use crate::error::{PiError, PiResult};
use crate::ExternalData;
use log::error;
use std::sync::Arc;

pub struct Domain;

pub enum FindDomainOf<'a> {
    DomainName(&'a str),
    Node(NodeId),
}

impl Domain {
    pub fn find_existing(
        engine: Arc<&Engine>,
        find_type: FindDomainOf,
    ) -> PiResult<Option<ArcedNodeItem>> {
        match find_type {
            FindDomainOf::DomainName(domain_name) => {
                // TODO: Implement and use better graph query API: https://github.com/pixlie/PixlieAI/issues/90, point 2
                let domain_node_ids = engine.get_node_ids_with_label(&NodeLabel::Domain);
                for node_id in domain_node_ids {
                    match engine.get_node_by_id(&node_id) {
                        Some(node) => {
                            if node.labels.contains(&NodeLabel::Domain) {
                                match node.payload {
                                    Payload::Text(ref domain) => {
                                        if domain == domain_name {
                                            return Ok(Some(node));
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
            FindDomainOf::Node(node_id) => {
                // TODO: Implement and use better graph query API: https://github.com/pixlie/PixlieAI/issues/90, point 1
                let belongs_to = engine.get_node_ids_connected_with_label(
                    &node_id,
                    &CommonEdgeLabels::BelongsTo.to_string(),
                )?;
                let first_belongs_to = belongs_to.first().ok_or_else(|| {
                    PiError::InternalError(format!(
                        "Could not find Domain node for given node with ID {}",
                        node_id
                    ))
                })?;
                match engine.get_node_by_id(first_belongs_to) {
                    Some(node) => {
                        if node.labels.contains(&NodeLabel::Domain) {
                            match node.payload {
                                Payload::Text(_) => Ok(Some(node)),
                                _ => Err(PiError::GraphError(
                                    "Cannot find domain node for URL".to_string(),
                                )),
                            }
                        } else {
                            Err(PiError::GraphError(
                                "Cannot find domain node for URL".to_string(),
                            ))
                        }
                    }
                    None => Err(PiError::InternalError(format!(
                        "Node with id {} not found",
                        first_belongs_to
                    ))),
                }
            }
        }
    }

    pub fn get_domain_name(node: &NodeItem) -> PiResult<String> {
        if !node.labels.contains(&NodeLabel::Domain) {
            error!("Expected Domain type payload");
            return Err(PiError::GraphError(
                "Expected Domain type payload".to_string(),
            ));
        }
        match &node.payload {
            Payload::Text(data) => Ok(data.clone()),
            _ => {
                error!("Expected Domain type payload");
                Err(PiError::GraphError(
                    "Expected Domain type payload".to_string(),
                ))
            }
        }
    }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    // We have received the contents of the `robots.txt` from the previous request
                    let content_node_id = engine
                        .get_or_add_node(
                            Payload::Text(response.contents),
                            vec![NodeLabel::RobotsTxt],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node.id.clone(), content_node_id),
                        (
                            CommonEdgeLabels::OwnerOf.to_string(),
                            CommonEdgeLabels::BelongsTo.to_string(),
                        ),
                    )?;
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(_error) => {
                    // TODO: Make sure to not save timeout or other errors as blank robots.txt
                    let content_node_id = engine
                        .get_or_add_node(
                            Payload::Text("".to_string()),
                            vec![NodeLabel::RobotsTxt],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node.id.clone(), content_node_id),
                        (
                            CommonEdgeLabels::OwnerOf.to_string(),
                            CommonEdgeLabels::BelongsTo.to_string(),
                        ),
                    )?;
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
            },
            None => engine.fetch("/robots.txt", &node.id)?,
        };
        Ok(())
    }
}
