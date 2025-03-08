use crate::engine::{
    get_chunk_id_and_node_ids, ArcedNodeId, ArcedNodeItem, NodeFlags, NodeId, NodeItem, Payload,
};
use crate::error::{PiError, PiResult};
use chrono::Utc;
use log::error;
use postcard::{from_bytes, to_allocvec};
use rocksdb::{Options, SliceTransform, DB};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

const NODES_CHUNK_PREFIX: &str = "nodes/chunk/";

pub(super) struct Nodes {
    pub(super) data: HashMap<ArcedNodeId, ArcedNodeItem>,
}

impl Nodes {
    pub(super) fn new() -> Nodes {
        Nodes {
            data: HashMap::new(),
        }
    }

    pub(super) fn save_item_chunk_to_disk(&self, db: Arc<DB>, node_id: &NodeId) -> PiResult<()> {
        // We store this (and all other nodes in its chunk) node to DB
        // in the chunk corresponding to the node ID divided by 100
        // Create a chunk of data from the start node ID to the end node ID of this chunk
        let (chunk_id, node_ids) = get_chunk_id_and_node_ids(node_id);
        let chunk: Vec<(&NodeId, &ArcedNodeItem)> = node_ids
            .iter()
            .filter_map(|x_node_id| match self.data.get(x_node_id) {
                Some(node) => Some((x_node_id, node)),
                None => None,
            })
            .collect();
        db.put(
            format!("{}{}", NODES_CHUNK_PREFIX, chunk_id),
            to_allocvec(&chunk)?,
        )?;
        // debug!("Saved chunk {} of length {} to DB", chunk_id, chunk.len());
        Ok(())
    }

    pub(super) fn load_all_from_disk(&mut self, db_path: &PathBuf) -> PiResult<u32> {
        let prefix_extractor = SliceTransform::create_fixed_prefix(NODES_CHUNK_PREFIX.len());
        let mut opts = Options::default();
        // TODO: Remove this and make sure that loading from disk is not called for new projects
        opts.create_if_missing(true);
        opts.set_prefix_extractor(prefix_extractor);
        let db = DB::open(&opts, db_path)?;
        let mut last_node_id: NodeId = 0;
        for chunk in db.prefix_iterator(NODES_CHUNK_PREFIX) {
            match chunk {
                Ok(chunk) => {
                    let data: Vec<(NodeId, NodeItem)> = from_bytes(&chunk.1)?;
                    last_node_id = data.last().unwrap().0;
                    for (node_id, node) in data {
                        self.data.insert(Arc::new(node_id), Arc::new(node));
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
        Ok(last_node_id)
    }

    pub(super) fn update_node(&mut self, node_id: &NodeId, payload: Payload) -> PiResult<()> {
        self.data.get_mut(node_id).map(|node| {
            *node = Arc::new(NodeItem {
                id: node.id.clone(),
                payload,
                labels: node.labels.clone(),
                flags: node.flags.clone(),
                written_at: Utc::now(),
            });
        });
        Ok(())
    }

    pub(super) fn toggle_flag(&mut self, node_id: &NodeId, flag: NodeFlags) {
        self.data.get_mut(node_id).map(|node| {
            let mut flags: NodeFlags = node.flags.clone();
            flags.toggle(flag);
            *node = Arc::new(NodeItem {
                id: node.id.clone(),
                payload: node.payload.clone(),
                labels: node.labels.clone(),
                flags,
                written_at: Utc::now(),
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::nodes::Nodes;
    use crate::engine::{CommonNodeLabels, NodeFlags, Payload};
    use crate::entity::search::SearchTerm;
    use crate::entity::web::domain::Domain;
    use crate::entity::web::link::Link;
    use crate::entity::web::web_page::WebPage;
    use chrono::Utc;
    use rocksdb::DB;
    use std::default::Default;

    // #[test]
    // fn test_save_to_disk_and_load_from_disk() {
    //     let mut node_id: NodeId = 0;
    //     let payloads: Vec<(Payload, String)> = vec![
    //         (
    //             Payload::Text("Test Title".to_string()),
    //             CommonNodeLabels::Title.to_string(),
    //         ),
    //         (
    //             Payload::Text("Test Heading".to_string()),
    //             CommonNodeLabels::Heading.to_string(),
    //         ),
    //         (
    //             Payload::Text("Test Paragraph".to_string()),
    //             CommonNodeLabels::Paragraph.to_string(),
    //         ),
    //         (
    //             Payload::ArrayOfTexts(vec!["Test Bullet Point".to_string()]),
    //             CommonNodeLabels::BulletPoints.to_string(),
    //         ),
    //         (
    //             Payload::ArrayOfTexts(vec!["Test Ordered Point".to_string()]),
    //             CommonNodeLabels::OrderedPoints.to_string(),
    //         ),
    //         (
    //             Payload::Link(Link {
    //                 path: "/".to_string(),
    //                 ..Default::default()
    //             }),
    //             CommonNodeLabels::Link.to_string(),
    //         ),
    //         (
    //             Payload::FileHTML(WebPage("<html></html>".to_string())),
    //             CommonNodeLabels::WebPage.to_string(),
    //         ),
    //         (
    //             Payload::Domain(Domain {
    //                 name: "google.com".to_string(),
    //                 ..Default::default()
    //             }),
    //             CommonNodeLabels::Domain.to_string(),
    //         ),
    //         (
    //             Payload::SearchTerm(SearchTerm("test".to_string())),
    //             CommonNodeLabels::SearchTerm.to_string(),
    //         ),
    //     ];
    //
    //     let nodes: Vec<NodeItem> = payloads
    //         .iter()
    //         .map(|payload| {
    //             let node = NodeItem {
    //                 id: node_id,
    //                 payload: payload.0.clone(),
    //                 labels: vec![payload.1.clone()],
    //                 flags: NodeFlags::default(),
    //                 written_at: Utc::now(),
    //             };
    //             node_id += 1;
    //             node
    //         })
    //         .collect();
    //
    //     let temp_dir = tempfile::Builder::new()
    //         .prefix("_path_for_rocksdb_storage2")
    //         .tempdir()
    //         .expect("Failed to create temporary path for the _path_for_rocksdb_storage2.");
    //     let db_path = temp_dir.path();
    //
    //     {
    //         let db = DB::open_default(db_path).unwrap();
    //         let mut db_nodes: Nodes = Nodes::new();
    //         // Insert all nodes into the DB
    //         nodes.iter().for_each(|node| {
    //             let key = Arc::new(node.id);
    //             let value = Arc::new(node.clone());
    //             db_nodes.data.insert(key.clone(), value);
    //             db_nodes.save_item_chunk_to_disk(&db, &key).unwrap();
    //         });
    //
    //         // Save the nodes to disk
    //         // db_nodes.save_all_to_disk(&db).unwrap();
    //     }
    //
    //     {
    //         let mut db_nodes = Nodes::new();
    //         // Load data from disk and check that it is the same as the original
    //         db_nodes.load_all_from_disk(&db_path).unwrap();
    //
    //         for node in nodes.iter() {
    //             // Check the ID and payload of each node against the one in the DB
    //             let db_node = db_nodes.data.get(&node.id).unwrap();
    //             assert_eq!(node.id, db_node.id);
    //             // Match the payload data type and check the inner values
    //             match node.payload {
    //                 Payload::Text(ref text) => {
    //                     let db_text = match db_node.payload {
    //                         Payload::Text(ref text) => text,
    //                         _ => panic!("Expected Title payload"),
    //                     };
    //                     assert_eq!(text, db_text);
    //                     assert_eq!(node.labels, db_node.labels);
    //                 }
    //                 Payload::ArrayOfTexts(ref texts) => {
    //                     let db_texts = match db_node.payload {
    //                         Payload::ArrayOfTexts(ref texts) => texts,
    //                         _ => panic!("Expected MultipleTexts payload"),
    //                     };
    //                     assert_eq!(texts, db_texts);
    //                     assert_eq!(node.labels, db_node.labels);
    //                 }
    //                 Payload::Link(ref link) => {
    //                     let db_link = match db_node.payload {
    //                         Payload::Link(ref link) => link,
    //                         _ => panic!("Expected Link payload"),
    //                     };
    //                     assert_eq!(link.path, db_link.path);
    //                     assert_eq!(link.query, db_link.query);
    //                 }
    //                 Payload::Domain(ref domain) => {
    //                     let db_domain = match db_node.payload {
    //                         Payload::Domain(ref domain) => domain,
    //                         _ => panic!("Expected Domain payload"),
    //                     };
    //                     assert_eq!(domain.name, db_domain.name);
    //                     assert_eq!(domain.is_allowed_to_crawl, db_domain.is_allowed_to_crawl);
    //                 }
    //                 Payload::FileHTML(ref web_page) => {
    //                     let db_web_page = match db_node.payload {
    //                         Payload::FileHTML(ref web_page) => web_page,
    //                         _ => panic!("Expected FileHTML payload"),
    //                     };
    //                     assert_eq!(web_page.0, db_web_page.0);
    //                 }
    //                 _ => {}
    //             }
    //         }
    //
    //         // let db = DB::open_default(db_path).unwrap();
    //         // db_nodes.save_all_to_disk(&db).unwrap();
    //     }
    //
    //     // Check this again
    //     {
    //         let mut db_nodes = Nodes::new();
    //         // Load data from disk and check that it is the same as the original
    //         db_nodes.load_all_from_disk(&db_path).unwrap();
    //
    //         for node in nodes.iter() {
    //             // Check the ID and payload of each node against the one in the DB
    //             let db_node = db_nodes.data.get(&node.id).unwrap();
    //             assert_eq!(node.id, db_node.id);
    //             // Match the payload data type and check the inner values
    //             match node.payload {
    //                 Payload::Text(ref text) => {
    //                     let db_text = match db_node.payload {
    //                         Payload::Text(ref text) => text,
    //                         _ => panic!("Expected Title payload"),
    //                     };
    //                     assert_eq!(text, db_text);
    //                     assert_eq!(node.labels, db_node.labels);
    //                 }
    //                 Payload::ArrayOfTexts(ref texts) => {
    //                     let db_texts = match db_node.payload {
    //                         Payload::ArrayOfTexts(ref texts) => texts,
    //                         _ => panic!("Expected BulletPoints payload"),
    //                     };
    //                     assert_eq!(texts, db_texts);
    //                 }
    //                 Payload::Link(ref link) => {
    //                     let db_link = match db_node.payload {
    //                         Payload::Link(ref link) => link,
    //                         _ => panic!("Expected Link payload"),
    //                     };
    //                     assert_eq!(link.path, db_link.path);
    //                     assert_eq!(link.query, db_link.query);
    //                 }
    //                 Payload::Domain(ref domain) => {
    //                     let db_domain = match db_node.payload {
    //                         Payload::Domain(ref domain) => domain,
    //                         _ => panic!("Expected Domain payload"),
    //                     };
    //                     assert_eq!(domain.name, db_domain.name);
    //                     assert_eq!(domain.is_allowed_to_crawl, db_domain.is_allowed_to_crawl);
    //                 }
    //                 Payload::FileHTML(ref web_page) => {
    //                     let db_web_page = match db_node.payload {
    //                         Payload::FileHTML(ref web_page) => web_page,
    //                         _ => panic!("Expected FileHTML payload"),
    //                     };
    //                     assert_eq!(web_page.0, db_web_page.0);
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    // }
}
