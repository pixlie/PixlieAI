// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::entity::{
    content::{BulletPoints, Heading, OrderedPoints, Paragraph, Table, TableRow, Title},
    web::{Domain, Link, WebPage},
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
pub mod manager;
pub mod setup;

// use crate::entity::content::TypedData;
pub use engine::Engine;
pub use engine::LockedEngine;

#[derive(Clone, Display, Deserialize, Serialize, TS)]
#[ts(export)]
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
}

pub type NodeId = Arc<u32>;
pub type NodeLabel = String;
pub type EdgeLabel = String;

#[derive(Display)]
pub enum CommonNodeLabels {
    AddedByUser,
}

#[derive(Display)]
pub enum CommonEdgeLabels {
    Related,
    Parent,
    Child,
    Content,
    Path,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct NodeItem {
    pub id: NodeId,
    pub labels: Vec<EdgeLabel>, // A node can have multiple labels, like tags, indexed by relevance
    pub payload: Payload,

    pub edges: HashMap<EdgeLabel, Vec<NodeId>>, // Nodes that are connected to this node
    pub written_at: DateTime<Utc>,
}

pub trait Node {
    fn get_label() -> String;

    fn process(&self, _engine: &Engine, _node_id: &NodeId) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
