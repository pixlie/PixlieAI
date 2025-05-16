// Copyright 2025 Pixlie Web Solutions Pvt. Ltd. (India)
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeId, NodeItem, Payload};
use crate::engine::{EdgeLabel, Engine};
use crate::entity::web::link::Link;
use crate::entity::web::scraper::scrape;
use crate::entity::web::web_metadata::WebMetadata;
use crate::error::{PiError, PiResult};
use crate::ExternalData;
use log::error;
use std::sync::Arc;

pub struct WebPage;

pub fn get_link_of_webpage(engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<(Link, NodeId)> {
    // Each WebPage node has a parent Link node, if not this is an error
    let related_node_ids =
        engine.get_node_ids_connected_with_label(node_id, &EdgeLabel::ContentOf)?;
    let first_related_node_id = related_node_ids.first().ok_or_else(|| {
        PiError::InternalError("No related node ids found for WebPage node".to_string())
    })?;

    match engine.get_node_by_id(first_related_node_id) {
        Some(node) => match node.payload {
            Payload::Link(ref link) => Ok((link.clone(), *first_related_node_id)),
            _ => Err(PiError::GraphError(
                "Cannot find parent Link node for WebPage node".to_string(),
            )),
        },
        None => {
            return Err(PiError::InternalError(format!(
                "Node with id {} not found",
                first_related_node_id
            )))
        }
    }
}

pub fn get_metadata_of_webpage(
    engine: Arc<&Engine>,
    node_id: &NodeId,
) -> PiResult<(WebMetadata, NodeId)> {
    // Each WebPage may have a child WebMetadata node
    let related_node_ids =
        engine.get_node_ids_connected_with_label(node_id, &EdgeLabel::ChildOf)?;
    let first_related_node_id = related_node_ids.first().ok_or_else(|| {
        PiError::InternalError("No related node ids found for WebPage node".to_string())
    })?;

    match engine.get_node_by_id(first_related_node_id) {
        Some(node) => match node.payload {
            Payload::WebMetadata(ref metadata) => Ok((metadata.clone(), *first_related_node_id)),
            _ => Err(PiError::GraphError(
                "Cannot find child WebMetadata node for WebPage node".to_string(),
            )),
        },
        None => {
            return Err(PiError::InternalError(format!(
                "Node with id {} not found",
                first_related_node_id
            )))
        }
    }
}

impl WebPage {
    fn _get_content(&self, engine: Arc<&Engine>, node_id: &NodeId) -> String {
        let part_node_ids =
            match engine.get_node_ids_connected_with_label(node_id, &EdgeLabel::ChildOf) {
                Ok(part_node_ids) => part_node_ids,
                Err(err) => {
                    error!("Error getting part node ids: {}", err);
                    return "".to_string();
                }
            };

        let mut content = String::new();
        for node_id in part_node_ids {
            let node = match engine.get_node_by_id(&node_id) {
                Some(node) => node,
                None => continue,
            };

            match node.payload {
                Payload::Text(ref text) => {
                    if text.len() > 0 {
                        content.push_str(&text);
                    }
                }
                _ => {}
            }
        }
        content
    }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        _data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        scrape(node, engine.clone())
    }
}
