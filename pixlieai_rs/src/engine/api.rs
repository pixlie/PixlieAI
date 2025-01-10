use super::Node;
use crate::error::PiResult;

pub enum GetFromGraph {
    GetNodesWithLabel(String),
    GetRelatedNodes(u32),
    GetPartNodes(u32),
}

pub async fn get_nodes_with_label(label: String) -> PiResult<Vec<Node>> {
    Ok(vec![])
}

pub async fn get_related_nodes(node_id: u32) -> PiResult<Vec<Node>> {
    Ok(vec![])
}

pub async fn get_part_nodes(node_id: u32) -> PiResult<Vec<Node>> {
    Ok(vec![])
}
