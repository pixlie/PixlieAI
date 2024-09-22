// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::entity::EntityType;
use chrono::{DateTime, Utc};
use petgraph::graph::NodeIndex;

pub mod api;
// pub mod executor;
pub mod state;

pub type EntityNode = (EntityType, NodeIndex);

pub struct EntityTypeNodesChunk {
    // Either created or modified
    pub written_at: DateTime<Utc>,
    pub entity_type: EntityType,
    pub node_indices: Vec<NodeIndex>,
}
