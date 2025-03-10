// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::entity::{content::TableRow, topic::Topic, workflow::WorkflowStep};
use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;
use strum::Display;
use ts_rs::TS;

pub mod api;
mod edges;
pub mod engine;
mod nodes;
pub mod setup;

use crate::engine::api::{EngineRequest, EngineResponse};
use crate::entity::search::SearchTerm;
use crate::entity::web::domain::Domain;
use crate::entity::web::link::Link;
use crate::entity::web::web_page::WebPage;
use crate::error::{PiError, PiResult};
use crate::ExternalData;
pub use engine::Engine;

#[derive(Clone, Display, Deserialize, Serialize)]
pub enum Payload {
    // StepPrompt(String),
    Step(WorkflowStep),
    Domain(Domain),
    Link(Link),
    Text(String),
    Tree, // Tree can contain nodes of any payload type, including other trees
    FileHTML(WebPage),
    TableRow(TableRow),
    SearchTerm(SearchTerm),
    Topic(Topic),
}

pub enum FindNode<'a> {
    Link(&'a str),
    Domain(&'a str),
}

pub(crate) type NodeId = u32;
pub(crate) type ArcedNodeId = Arc<NodeId>;
pub(crate) type NodeLabel = String;
pub(crate) type EdgeLabel = String;

pub(crate) type ArcedEdgeLabel = Arc<EdgeLabel>;

#[derive(Display, TS)]
#[ts(export)]
pub enum CommonNodeLabels {
    AddedByUser,
    Content,
    Domain,
    Heading,
    Link,
    ListItem,
    OrderedPoints,
    Paragraph,
    Partial,
    RobotsTxt,
    SearchTerm,
    Title,
    UnorderedPoints,
    WebPage,
}

#[derive(Display, TS)]
#[ts(export)]
pub enum CommonEdgeLabels {
    RelatedTo,

    ParentOf, // When one node is like a container of the other
    ChildOf,

    ContentOf, // When one node is the content from a file path
    PathOf,

    OwnerOf, // When one node is the root path of another (like domain and path or folder and file)
    BelongsTo,

    Suggests, // When one node is suggested based on another
    SuggestedFor,

    EvaluatedFor, // When one node is evaluated for another
}

bitflags! {
    #[derive(Clone, Deserialize, Serialize)]
    pub struct NodeFlags: u8 {
        // This is set when a node is processed and does not need to be processed unless it changes
        const IS_PROCESSED = 1;

        // This is set when a node makes an external data request which has not finished yet
        const IS_REQUESTING = 1 << 1;

        // This flag says that a node cannot be processed given the current state of the graph
        // For example, if a domain is not set to be crawled, then we cannot fetch any URLs from it
        // In that case all Link nodes belonging to that domain will have this flag set
        const IS_BLOCKED = 1 << 2;
    }
}

impl Default for NodeFlags {
    fn default() -> Self {
        NodeFlags::empty()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeItem {
    pub id: NodeId,
    pub labels: Vec<NodeLabel>, // A node can have multiple labels, like tags, indexed by relevance
    pub payload: Payload,

    pub flags: NodeFlags,
    pub written_at: DateTime<Utc>,
}

impl NodeItem {
    pub fn get_label(&self) -> String {
        self.payload.to_string()
    }
}

impl Ord for NodeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for NodeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NodeItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NodeItem {}

pub type ArcedNodeItem = Arc<NodeItem>;

pub trait Node {
    fn get_label() -> String;

    fn process(
        &self,
        _engine: Arc<&Engine>,
        _node_id: &NodeId,
        _data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()>
    where
        Self: Sized,
    {
        Err(PiError::NotAvailable(format!(
            "Process on node {} is not available",
            Self::get_label()
        )))
    }

    fn query(&self, _engine: Arc<&Engine>, _node_id: &NodeId) -> PiResult<Vec<NodeItem>>
    where
        Self: Sized,
    {
        Err(PiError::NotAvailable(format!(
            "Query on node {} is not available",
            Self::get_label()
        )))
    }
}

pub enum EngineWorkData {
    APIRequest(EngineRequest),
    APIResponse(EngineResponse),
    FetchRequest,
    FetchResponse,
}

pub enum ExistingOrNewNodeId {
    Existing(NodeId),
    New(NodeId),
}

impl ExistingOrNewNodeId {
    pub fn get_node_id(&self) -> NodeId {
        match self {
            ExistingOrNewNodeId::Existing(id) => id.clone(),
            ExistingOrNewNodeId::New(id) => id.clone(),
        }
    }
}

pub(super) fn get_chunk_id_and_node_ids(node_id: &u32) -> (u32, Vec<u32>) {
    let chunk_id = node_id / 100;
    (chunk_id, (chunk_id * 100..(chunk_id * 100 + 100)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chunk_id_and_node_ids() {
        let node_id = 0;
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(&node_id);
        assert_eq!(chunk_id, 0);
        assert_eq!(node_ids.len(), 100);
        assert_eq!(node_ids[0], 0);
        assert_eq!(node_ids[99], 99);

        let node_id = 100;
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(&node_id);
        assert_eq!(chunk_id, 1);
        assert_eq!(node_ids.len(), 100);
        assert_eq!(node_ids[0], 100);
        assert_eq!(node_ids[99], 199);

        let node_id = 10011;
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(&node_id);
        assert_eq!(chunk_id, 100);
        assert_eq!(node_ids.len(), 100);
        assert_eq!(node_ids[0], 10000);
        assert_eq!(node_ids[99], 10099);
    }
}
