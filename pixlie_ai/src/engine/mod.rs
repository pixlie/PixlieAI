// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::entity::{
    content::{BulletPoints, Heading, OrderedPoints, Paragraph, Table, TableRow, Title},
    workflow::WorkflowStep,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use strum::Display;
use ts_rs::TS;

pub mod api;
pub mod engine;
pub mod setup;

// use crate::entity::content::TypedData;
use crate::engine::api::{EngineRequest, EngineResponse};
use crate::entity::search::SearchTerm;
use crate::entity::topic::Topic;
use crate::entity::web::domain::Domain;
use crate::entity::web::link::Link;
use crate::entity::web::web_page::WebPage;
use crate::error::{PiError, PiResult};
pub use engine::Engine;

#[derive(Clone, Display, Deserialize, Serialize)]
pub enum Payload {
    // StepPrompt(String),
    Step(WorkflowStep),
    Domain(Domain),
    Link(Link),
    FileHTML(WebPage),
    Title(Title),
    Heading(Heading),
    Paragraph(Paragraph),
    BulletPoints(BulletPoints),
    OrderedPoints(OrderedPoints),
    Table(Table),
    TableRow(TableRow),
    Label(String),
    // TypedData(TypedData),
    NamedEntity(String, String), // label, text
    SearchTerm(SearchTerm),
    Topic(Topic),
}

pub enum FindNode<'a> {
    Link(&'a str),
    Domain(&'a str),
}

pub type NodeId = Arc<u32>;
pub type NodeLabel = String;
pub type EdgeLabel = String;

#[derive(Display, TS)]
#[ts(export)]
pub enum CommonNodeLabels {
    AddedByUser,
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

    Suggests,  // When one node is suggested based on another
    SuggestedFor,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeItem {
    pub id: NodeId,
    pub labels: Vec<NodeLabel>, // A node can have multiple labels, like tags, indexed by relevance
    pub payload: Payload,

    pub edges: HashMap<EdgeLabel, Vec<NodeId>>, // Nodes that are connected to this node
    pub written_at: DateTime<Utc>,
}

pub trait Node {
    fn get_label() -> String;

    fn process(&self, _engine: Arc<&Engine>, _node_id: &NodeId) -> PiResult<()>
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
    Pending(NodeId),
    New(NodeId),
}

impl ExistingOrNewNodeId {
    pub fn get_node_id(&self) -> NodeId {
        match self {
            ExistingOrNewNodeId::Existing(id) => id.clone(),
            ExistingOrNewNodeId::Pending(id) => id.clone(),
            ExistingOrNewNodeId::New(id) => id.clone(),
        }
    }
}
