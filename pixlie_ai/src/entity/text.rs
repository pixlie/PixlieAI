use crate::engine::node::{NodeId, NodeLabel, Payload};
use crate::engine::Engine;
use crate::error::PiResult;
use std::sync::Arc;

pub struct Text;

impl Text {
    pub fn add(engine: Arc<&Engine>, text: &String, labels: Vec<NodeLabel>) -> PiResult<NodeId> {
        let node_id = engine
            .get_or_add_node(Payload::Text(text.to_string()), labels, true, None)?
            .get_node_id();
        Ok(node_id)
    }
}
