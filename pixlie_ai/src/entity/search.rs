use crate::engine::{CommonNodeLabels, Engine, Node, NodeId, NodeItem, Payload};
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
        )?;
        Ok(())
    }
    pub fn find_existing(engine: Arc<&Engine>, search_term: &str) -> PiResult<Option<(SearchTerm, NodeId)>> {
        match engine.node_ids_by_label.read() {
            Ok(node_ids_by_label) => {
                match node_ids_by_label.get(&Self::get_label()) {
                    Some(node_ids) => {
                        for node_id in node_ids {
                            match engine.get_node_by_id(node_id) {
                                Ok(node) => {
                                    match node.payload {
                                        Payload::SearchTerm(ref term) => {
                                            if term.0 == search_term {
                                                return Ok(Some((term.clone(), node_id.clone())));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Err(err) => {
                                    error!("Error reading SearchTerm node: {}", err);
                                    return Err(PiError::InternalError(format!(
                                        "Error reading SearchTerm node: {}",
                                        err
                                    )));
                                }
                            }
                        }
                        Ok(None)
                    }
                    None => Ok(None),
                }
            }
            Err(err) => {
                error!("Error reading node_ids_by_label: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error reading node_ids_by_label: {}",
                    err
                )));
            }
        }
    }
}

impl Node for SearchTerm {
    fn get_label() -> String {
        "SearchTerm".to_string()
    }

    fn query(&self, engine: Arc<&Engine>, _node_id: &NodeId) -> PiResult<Vec<NodeItem>> {
        // We search all the content nodes in the engine for the search term
        let results: Vec<Option<NodeItem>> = engine.map_nodes(|id, node| match node.payload {
            Payload::Title(ref title) => {
                if title.0.to_lowercase().contains(&self.0.to_lowercase()) {
                    Some(NodeItem {
                        id: **id,
                        labels: node.labels.clone(),
                        payload: node.payload.clone(),
                        written_at: node.written_at.clone(),
                    })
                } else {
                    None
                }
            }
            Payload::Heading(ref heading) => {
                if heading.0.to_lowercase().contains(&self.0.to_lowercase()) {
                    Some(NodeItem {
                        id: **id,
                        labels: node.labels.clone(),
                        payload: node.payload.clone(),
                        written_at: node.written_at.clone(),
                    })
                } else {
                    None
                }
            }
            Payload::Paragraph(ref paragraph) => {
                if paragraph.0.to_lowercase().contains(&self.0.to_lowercase()) {
                    Some(NodeItem {
                        id: **id,
                        labels: node.labels.clone(),
                        payload: node.payload.clone(),
                        written_at: node.written_at.clone(),
                    })
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
