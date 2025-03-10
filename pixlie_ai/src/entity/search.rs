use crate::engine::{ArcedNodeId, ArcedNodeItem, CommonNodeLabels, Engine, Node, NodeId, NodeItem, Payload};
use crate::error::PiResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct SearchTerm(pub String);

impl SearchTerm {
    pub fn add_manually(engine: Arc<&Engine>, search_term: &str) -> PiResult<()> {
        engine.get_or_add_node(
            Payload::SearchTerm(Self(search_term.to_string())),
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
            None,
        )?;
        Ok(())
    }
    pub fn find_existing(engine: Arc<&Engine>, search_term: &str) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        let existing_node_ids: Vec<ArcedNodeId> = engine.get_node_ids_with_label(&SearchTerm::get_label());
        for node_id in existing_node_ids {
            match engine.get_node_by_id(&node_id) {
                Some(node) => match &node.payload {
                    Payload::SearchTerm(node_search_term) => {
                        if node_search_term.0 == search_term {
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

impl Node for SearchTerm {
    fn get_label() -> String {
        "SearchTerm".to_string()
    }

    fn query(&self, engine: Arc<&Engine>, _node_id: &NodeId) -> PiResult<Vec<NodeItem>> {
        // We search all the content nodes in the engine for the search term
        let results: Vec<Option<NodeItem>> = engine.map_nodes(|id, node| match node.payload {
            Payload::Text(ref text) => {
                if node.labels.contains(&CommonNodeLabels::Title.to_string())
                    || node.labels.contains(&CommonNodeLabels::Heading.to_string())
                    || node.labels.contains(&CommonNodeLabels::Paragraph.to_string())
                {
                    if text.to_lowercase().contains(&self.0.to_lowercase()) {
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
        })?;
        Ok(results
            .iter()
            .filter_map(|x| match x {
                Some(node) => Some(node.clone()),
                None => None,
            })
            .collect())
    }
}
