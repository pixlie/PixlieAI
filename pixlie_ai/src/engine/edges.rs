use crate::engine::node::{ArcedNodeId, NodeId};
use crate::engine::{get_chunk_id_and_node_ids, NodeEdges};
use crate::error::{PiError, PiResult};
use log::error;
use postcard::{from_bytes, to_allocvec};
use rocksdb::{Options, SliceTransform, DB};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

const EDGES_CHUNK_PREFIX: &str = "edges/chunk/";

pub(super) struct Edges {
    pub(super) data: HashMap<ArcedNodeId, NodeEdges>,
}

impl Edges {
    pub(super) fn new() -> Edges {
        Edges {
            data: HashMap::new(),
        }
    }

    pub(super) fn save_item_chunk_to_disk(&self, db: Arc<DB>, node_id: &NodeId) -> PiResult<()> {
        // We store this (and all other edges in its chunk) edge to DB
        // in the chunk corresponding to the node ID divided by 100
        // Create a chunk of data from the start node ID to the end node ID of this chunk
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(node_id);
        let chunk: Vec<(NodeId, NodeEdges)> = node_ids
            .iter()
            .filter_map(|x_node_id| match self.data.get(x_node_id) {
                Some(node_edges) => Some((*x_node_id, node_edges.clone())),
                None => None,
            })
            .collect();
        db.put(
            format!("{}{}", EDGES_CHUNK_PREFIX, chunk_id),
            to_allocvec(&chunk)?,
        )?;
        Ok(())
    }

    pub(super) fn open(db_path: &PathBuf) -> PiResult<Self> {
        let prefix_extractor = SliceTransform::create_fixed_prefix(EDGES_CHUNK_PREFIX.len());
        let mut edges = Edges::new();
        let mut opts = Options::default();
        opts.create_if_missing(false);
        opts.set_prefix_extractor(prefix_extractor);
        let db = match DB::open(&opts, db_path) {
            Ok(db) => db,
            Err(err) => {
                return Err(PiError::RocksdbError(err));
            }
        };
        for chunk in db.prefix_iterator(EDGES_CHUNK_PREFIX) {
            match chunk {
                Ok(chunk) => {
                    let data: Vec<(NodeId, NodeEdges)> = from_bytes(&chunk.1)?;
                    for (node_id, edge) in data {
                        edges.data.insert(Arc::new(node_id), edge);
                    }
                }
                Err(err) => {
                    error!("Error reading chunk from DB: {}", err);
                    return Err(PiError::RocksdbError(err));
                }
            }
        }
        Ok(edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EdgeLabel;
    use chrono::Utc;
    use rocksdb::DB;

    #[test]
    fn test_save_to_disk_and_load_from_disk() {
        let mut node_id: NodeId = 0;
        let mut edges: Vec<(NodeId, NodeEdges)> = vec![];
        edges.push((
            node_id,
            NodeEdges {
                edges: vec![(node_id + 1, EdgeLabel::ParentOf)],
                written_at: Utc::now(),
            },
        ));
        node_id += 1;
        edges.push((
            node_id,
            NodeEdges {
                edges: vec![(node_id + 1, EdgeLabel::ParentOf)],
                written_at: Utc::now(),
            },
        ));
        node_id += 1;
        edges.push((
            node_id,
            NodeEdges {
                edges: vec![
                    (node_id + 1, EdgeLabel::ParentOf),
                    (node_id + 2, EdgeLabel::ChildOf),
                ],
                written_at: Utc::now(),
            },
        ));
        node_id += 1;
        edges.push((
            node_id,
            NodeEdges {
                edges: vec![
                    (node_id + 1, EdgeLabel::ParentOf),
                    (node_id + 2, EdgeLabel::ChildOf),
                ],
                written_at: Utc::now(),
            },
        ));

        let temp_dir = tempfile::Builder::new()
            .prefix("_path_for_rocksdb_storage2")
            .tempdir()
            .expect("Failed to create temporary path for the _path_for_rocksdb_storage2.");
        let db_path = PathBuf::from(temp_dir.path());

        {
            let db = DB::open_default(db_path.clone()).unwrap();
            let arced_db = Arc::new(db);
            let mut db_edges: Edges = Edges::new();
            // Insert all edges into the DB
            for (node_id, node_edges) in edges.iter() {
                db_edges
                    .data
                    .insert(Arc::new(node_id.clone()), node_edges.clone());
            }

            // Save the edges to disk
            db_edges
                .save_item_chunk_to_disk(arced_db.clone(), &node_id)
                .unwrap();
        }

        {
            // Load data from disk and check that it is the same as the original
            let db_edges = Edges::open(&db_path).unwrap();

            for (node_id, node_edges) in edges.iter() {
                // Check the ID and payload of each node against the one in the DB
                let db_edges = db_edges.data.get(node_id).unwrap();
                assert_eq!(node_edges.edges.len(), db_edges.edges.len());
                assert_eq!(node_edges.written_at, db_edges.written_at);
                for (db_node_id, db_edge_label) in db_edges.edges.iter() {
                    assert!(node_edges
                        .edges
                        .contains(&(db_node_id.clone(), db_edge_label.clone())));
                }
            }
        }
    }
}
