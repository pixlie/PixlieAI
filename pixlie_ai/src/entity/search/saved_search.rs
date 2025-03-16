use crate::engine::node::{ArcedNodeId, ArcedNodeItem, NodeId, NodeItem, NodeLabel, Payload};
use crate::engine::Engine;
use crate::error::{PiError, PiResult};
use std::sync::Arc;

pub struct SavedSearch;

impl SavedSearch {
    pub fn add_manually(engine: Arc<&Engine>, search_term: &str) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::Text(search_term.to_string()),
            vec![NodeLabel::AddedByUser, NodeLabel::SearchTerm],
            true,
            None,
        )?;
        Ok(())
    }

    pub fn find_existing(
        engine: Arc<&Engine>,
        search_term: &str,
    ) -> PiResult<Option<ArcedNodeItem>> {
        let existing_node_ids: Vec<ArcedNodeId> =
            engine.get_node_ids_with_label(&NodeLabel::SearchTerm);
        for node_id in existing_node_ids {
            match engine.get_node_by_id(&node_id) {
                Some(node) => match &node.payload {
                    Payload::Text(text) => {
                        if text == search_term {
                            return Ok(Some(node));
                        }
                    }
                    _ => {}
                },
                None => {}
            }
        }
        Ok(None)
    }

    pub(crate) fn query(
        node: &NodeItem,
        engine: Arc<&Engine>,
        _node_id: &NodeId,
    ) -> PiResult<Vec<NodeItem>> {
        if !node.labels.contains(&NodeLabel::SearchTerm) {
            return Err(PiError::InternalError(format!(
                "Expected SearchTerm, got {}",
                node.labels
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )));
        }

        // We search all the content nodes in the engine for the search term
        let results: Vec<Option<NodeItem>> = match &node.payload {
            Payload::Text(search_term) => engine.map_nodes(|id, node| match node.payload {
                Payload::Text(ref text) => {
                    if node.labels.contains(&NodeLabel::Title)
                        || node.labels.contains(&NodeLabel::Heading)
                        || node.labels.contains(&NodeLabel::Paragraph)
                    {
                        if text.to_lowercase().contains(&search_term.to_lowercase()) {
                            Some(NodeItem {
                                id: **id,
                                labels: node.labels.clone(),
                                payload: node.payload.clone(),
                                flags: node.flags.clone(),
                                written_at: node.written_at.clone(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })?,
            _ => {
                vec![]
            }
        };
        Ok(results
            .iter()
            .filter_map(|x| match x {
                Some(node) => Some(node.clone()),
                None => None,
            })
            .collect())
    }
}
