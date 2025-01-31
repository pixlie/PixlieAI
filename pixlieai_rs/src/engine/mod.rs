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
use std::sync::Arc;
use strum::Display;
use ts_rs::TS;

pub mod api;
pub mod engine;
pub mod manager;
pub mod setup;

use crate::entity::content::TypedData;
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
    TypedData(TypedData),
    NamedEntity(String, String), // label, text
}

pub type NodeId = Arc<u32>;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub payload: Payload,

    pub parent_id: Option<NodeId>,
    pub part_node_ids: Vec<NodeId>, // These nodes make up the parts of this node
    pub related_node_ids: Vec<NodeId>, // Nodes that are related, but are not part of this node
    pub written_at: DateTime<Utc>,
}

pub enum RelationType {
    IsPart,
    IsRelated,
}

pub struct PendingNode {
    pub payload: Payload,
    pub creating_node_id: NodeId,
    pub related_type: RelationType,
}

pub trait NodeWorker {
    fn get_label() -> String;

    fn process(&self, _engine: &Engine, _node_id: &NodeId) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
