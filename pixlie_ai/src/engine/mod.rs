// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::Display;
use ts_rs::TS;

pub mod api;
mod edges;
pub mod engine;
pub(crate) mod node;
mod nodes;
pub mod setup;

use crate::engine::api::{EngineRequest, EngineResponse};
pub use engine::Engine;

pub(crate) type EdgeLabel = String;

pub(crate) type ArcedEdgeLabel = Arc<EdgeLabel>;

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

pub enum EngineWorkData {
    APIRequest(EngineRequest),
    APIResponse(EngineResponse),
    FetchRequest,
    FetchResponse,
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
