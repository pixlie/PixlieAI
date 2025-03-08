use crate::engine::{get_chunk_id_and_node_ids, ArcedEdgeLabel, ArcedNodeId, EdgeLabel, NodeId};
use crate::error::{PiError, PiResult};
use log::{error, info};
use postcard::{from_bytes, to_allocvec};
use rocksdb::{ErrorKind, Options, SliceTransform, DB};
use std::collections::HashMap;
use std::path::PathBuf;
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

    pub(super) fn save_item_chunk_to_disk(&self, db: Arc<DB>, node_id: &NodeId) -> PiResult<()> {
        // We store this (and all other edges in its chunk) edge to DB
        // in the chunk corresponding to the node ID divided by 100
        // Create a chunk of data from the start node ID to the end node ID of this chunk
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(node_id);
        let chunk: Vec<(NodeId, Vec<(NodeId, EdgeLabel)>)> = node_ids
            .iter()
            .filter_map(|x_node_id| match self.data.get(x_node_id) {
                Some(edges) => Some((
                    *x_node_id,
                    edges
                        .iter()
                        .map(|(y_node_id, y_label)| (**y_node_id, y_label.to_string()))
                        .collect(),
                )),
                None => None,
            })
            .collect();
        db.put(
            format!("{}{}", EDGES_CHUNK_PREFIX, chunk_id),
            to_allocvec(&chunk)?,
        )?;
        Ok(())
    }

    pub(super) fn load_all_from_disk(&mut self, db_path: &PathBuf) -> PiResult<()> {
        let prefix_extractor = SliceTransform::create_fixed_prefix(EDGES_CHUNK_PREFIX.len());
        let mut opts = Options::default();
        opts.create_if_missing(false);
        opts.set_prefix_extractor(prefix_extractor);
        let db = match DB::open(&opts, db_path) {
            Ok(db) => db,
            Err(err) => {
                return if err.kind() == ErrorKind::InvalidArgument
                    && err.to_string().contains("does not exist")
                {
                    Ok(())
                } else {
                    Err(PiError::InternalError(
                        "Database does not exist".to_string(),
                    ))
                }
            }
        };
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
        let db_path = PathBuf::from(temp_dir.path());

        {
            let db = DB::open_default(db_path.clone()).unwrap();
            let arced_db = Arc::new(db);
            let mut db_edges: Edges = Edges::new();
            // Insert all edges into the DB
            for (node_id, edges) in edges.iter() {
                db_edges
                    .data
                    .insert(Arc::new(node_id.clone()), edges.clone());
            }

            // Save the edges to disk
            db_edges
                .save_item_chunk_to_disk(arced_db.clone(), &node_id)
                .unwrap();
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
