use crate::engine::{get_chunk_id_and_node_ids, ArcedEdgeLabel, ArcedNodeId, NodeId};
use crate::error::{PiError, PiResult};
use log::error;
use postcard::{from_bytes, to_allocvec};
use rocksdb::{Options, SliceTransform, DB};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

const EDGES_CHUNK_PREFIX: &str = "edges/chunk/";

pub(super) struct Edges {
    pub(super) data: HashMap<ArcedNodeId, Vec<(ArcedNodeId, ArcedEdgeLabel)>>,
}

impl Edges {
    pub(super) fn new() -> Edges {
        Edges {
            data: HashMap::new(),
        }
    }

    pub(super) fn save_all_to_disk(&self, db: &DB) -> PiResult<()> {
        // We store all edges in the DB, in chunks
        // Each chunk has edges for up to 100 starting nodes
        let mut chunk_id = 0;
        let mut chunk: Vec<(&ArcedNodeId, &Vec<(ArcedNodeId, ArcedEdgeLabel)>)> = vec![];
        // let mut all_node_ids_with_edges: Vec<NodeId> = vec![];
        for (node_id, edges) in self.data.iter() {
            if chunk.len() < 100 {
                chunk.push((node_id, edges));
            } else if chunk.len() == 100 {
                db.put(
                    format!("{}{}", EDGES_CHUNK_PREFIX, chunk_id),
                    to_allocvec(&chunk)?,
                )?;
                chunk_id += 1;
                chunk = vec![];
            }
        }
        if !chunk.is_empty() {
            db.put(
                format!("{}{}", EDGES_CHUNK_PREFIX, chunk_id),
                to_allocvec(&chunk)?,
            )?;
        }
        Ok(())
    }

    pub(super) fn save_item_chunk_to_disk(&self, db: &DB, node_id: &NodeId) -> PiResult<()> {
        // We store this (and all other edges in its chunk) edge to DB
        // in the chunk corresponding to the node ID divided by 100
        // Create a chunk of data from the start node ID to the end node ID of this chunk
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(node_id);
        let chunk: Vec<(&NodeId, &Vec<(ArcedNodeId, ArcedEdgeLabel)>)> = node_ids
            .iter()
            .filter_map(|x_node_id| match self.data.get(x_node_id) {
                Some(edges) => Some((x_node_id, edges)),
                None => None,
            })
            .collect();
        db.put(
            format!("{}{}", EDGES_CHUNK_PREFIX, chunk_id),
            to_allocvec(&chunk)?,
        )?;
        // debug!("Saved chunk {} of length {} to DB", chunk_id, chunk.len());
        Ok(())
    }

    pub(super) fn load_all_from_disk(&mut self, db_path: &Path) -> PiResult<()> {
        let prefix_extractor = SliceTransform::create_fixed_prefix(EDGES_CHUNK_PREFIX.len());
        let mut opts = Options::default();
        // TODO: Remove this and make sure that loading from disk is not called for new projects
        opts.create_if_missing(true);
        opts.set_prefix_extractor(prefix_extractor);
        let db = DB::open(&opts, db_path)?;
        for chunk in db.prefix_iterator(EDGES_CHUNK_PREFIX) {
            match chunk {
                Ok(chunk) => {
                    let data: Vec<(NodeId, Vec<(ArcedNodeId, ArcedEdgeLabel)>)> =
                        from_bytes(&chunk.1)?;
                    for (node_id, edges) in data {
                        self.data.insert(Arc::new(node_id), edges);
                    }
                }
                Err(err) => {
                    error!("Error reading chunk from DB: {}", err);
                    return Err(PiError::InternalError(format!(
                        "Error reading chunk from DB: {}",
                        err
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CommonEdgeLabels;
    use rocksdb::DB;

    #[test]
    fn test_save_to_disk_and_load_from_disk() {
        let mut node_id: NodeId = 0;
        let mut edges: Vec<(NodeId, Vec<(ArcedNodeId, ArcedEdgeLabel)>)> = vec![];
        edges.push((
            node_id,
            vec![(
                Arc::new(node_id + 1),
                Arc::new(CommonEdgeLabels::ParentOf.to_string()),
            )],
        ));
        node_id += 1;
        edges.push((
            node_id,
            vec![(
                Arc::new(node_id + 1),
                Arc::new(CommonEdgeLabels::ParentOf.to_string()),
            )],
        ));
        node_id += 1;
        edges.push((
            node_id,
            vec![
                (
                    Arc::new(node_id + 1),
                    Arc::new(CommonEdgeLabels::ParentOf.to_string()),
                ),
                (
                    Arc::new(node_id + 2),
                    Arc::new(CommonEdgeLabels::ChildOf.to_string()),
                ),
            ],
        ));
        node_id += 1;
        edges.push((
            node_id,
            vec![
                (
                    Arc::new(node_id + 1),
                    Arc::new(CommonEdgeLabels::ParentOf.to_string()),
                ),
                (
                    Arc::new(node_id + 2),
                    Arc::new(CommonEdgeLabels::ChildOf.to_string()),
                ),
            ],
        ));

        let temp_dir = tempfile::Builder::new()
            .prefix("_path_for_rocksdb_storage2")
            .tempdir()
            .expect("Failed to create temporary path for the _path_for_rocksdb_storage2.");
        let db_path = temp_dir.path();

        {
            let db = DB::open_default(db_path).unwrap();
            let mut db_edges: Edges = Edges::new();
            // Insert all edges into the DB
            for (node_id, edges) in edges.iter() {
                db_edges
                    .data
                    .insert(Arc::new(node_id.clone()), edges.clone());
            }

            // Save the edges to disk
            db_edges.save_all_to_disk(&db).unwrap();
        }

        {
            let mut db_edges = Edges::new();
            // Load data from disk and check that it is the same as the original
            db_edges.load_all_from_disk(&db_path).unwrap();

            for (node_id, edges) in edges.iter() {
                // Check the ID and payload of each node against the one in the DB
                let db_edges = db_edges.data.get(node_id).unwrap();
                assert_eq!(edges.len(), db_edges.len());
                for (db_node_id, db_edge_label) in db_edges.iter() {
                    assert!(edges.contains(&(db_node_id.clone(), db_edge_label.clone())));
                }
            }
        }
    }
}
