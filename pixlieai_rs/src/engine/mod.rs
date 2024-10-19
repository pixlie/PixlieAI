// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::entity::LabelId;
use chrono::{DateTime, Utc};
// use petgraph::graph::NodeIndex;

pub mod api;
// pub mod executor;
pub mod state;

pub type NodeId = u32;
pub struct Node {
    pub id: NodeId,
    pub payload: Box<dyn Send>,
    pub written_at: DateTime<Utc>,
}
