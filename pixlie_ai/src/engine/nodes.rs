use crate::engine::{
    get_chunk_id_and_node_ids, ArcedNodeId, ArcedNodeItem, NodeFlags, NodeId, NodeItem, Payload,
};
use crate::error::{PiError, PiResult};
use chrono::Utc;
use itertools::sorted;
use log::{debug, error};
use postcard::{from_bytes, to_allocvec};
use rocksdb::{Options, SliceTransform, DB};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

// const NODES_KEY_PREFIX: &str = "nodes/";
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

    pub(super) fn save_all_to_disk(&self, db: &DB) -> PiResult<()> {
        // We store all nodes in the DB, in chunks
        // Each chunk has up to 100 nodes
        let mut chunk_id = 0;
        let mut chunk: Vec<(&ArcedNodeId, &ArcedNodeItem)> = vec![];
        for data in sorted(self.data.iter()) {
            if chunk.len() < 100 {
                chunk.push(data);
            } else if chunk.len() == 100 {
                db.put(
                    format!("{}{}", NODES_CHUNK_PREFIX, chunk_id),
                    to_allocvec(&chunk)?,
                )?;
                chunk_id += 1;
                chunk = vec![];
            }
        }
        if !chunk.is_empty() {
            db.put(
                format!("{}{}", NODES_CHUNK_PREFIX, chunk_id),
                to_allocvec(&chunk)?,
            )?;
        }
        Ok(())
    }

    pub(super) fn save_item_chunk_to_disk(&self, db: &DB, node_id: &NodeId) -> PiResult<()> {
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
        debug!("Saved chunk {} of length {} to DB", chunk_id, chunk.len());
        Ok(())
    }

    pub(super) fn load_all_from_disk(&mut self, db_path: &Path) -> PiResult<u32> {
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

    pub(super) fn update_flag(&mut self, node_id: &NodeId, flag: NodeFlags) {
        self.data.get_mut(node_id).map(|node| {
            *node = Arc::new(NodeItem {
                id: node.id.clone(),
                payload: node.payload.clone(),
                labels: node.labels.clone(),
                flags: NodeFlags::from_bits_truncate(node.flags.bits() | flag.bits()),
                written_at: Utc::now(),
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::nodes::Nodes;
    use crate::engine::{NodeFlags, Payload};
    use crate::entity::content::{BulletPoints, Heading, OrderedPoints, Paragraph, Title};
    use crate::entity::search::SearchTerm;
    use crate::entity::web::domain::Domain;
    use crate::entity::web::link::Link;
    use crate::entity::web::web_page::WebPage;
    use chrono::Utc;
    use rocksdb::DB;
    use std::default::Default;

    #[test]
    fn test_save_to_disk_and_load_from_disk() {
        let mut node_id: NodeId = 0;
        let payloads: Vec<Payload> = vec![
            Payload::Title(Title("Test Title".to_string())),
            Payload::Heading(Heading("Test Heading".to_string())),
            Payload::Paragraph(Paragraph("Test Paragraph".to_string())),
            Payload::BulletPoints(BulletPoints(vec!["Test Bullet Point".to_string()])),
            Payload::OrderedPoints(OrderedPoints(vec!["Test Ordered Point".to_string()])),
            Payload::Link(Link {
                path: "/".to_string(),
                ..Default::default()
            }),
            Payload::FileHTML(WebPage {
                contents: "<html></html>".to_string(),
                ..Default::default()
            }),
            Payload::Domain(Domain {
                name: "google.com".to_string(),
                ..Default::default()
            }),
            Payload::SearchTerm(SearchTerm("test".to_string())),
        ];

        let nodes: Vec<NodeItem> = payloads
            .iter()
            .map(|payload| {
                let node = NodeItem {
                    id: node_id,
                    payload: payload.clone(),
                    labels: vec![],
                    flags: NodeFlags::default(),
                    written_at: Utc::now(),
                };
                node_id += 1;
                node
            })
            .collect();

        let temp_dir = tempfile::Builder::new()
            .prefix("_path_for_rocksdb_storage2")
            .tempdir()
            .expect("Failed to create temporary path for the _path_for_rocksdb_storage2.");
        let db_path = temp_dir.path();

        {
            let db = DB::open_default(db_path).unwrap();
            let mut db_nodes: Nodes = Nodes::new();
            // Insert all nodes into the DB
            nodes.iter().for_each(|node| {
                let key = Arc::new(node.id);
                let value = Arc::new(node.clone());
                db_nodes.data.insert(key, value);
            });

            // Save the nodes to disk
            db_nodes.save_all_to_disk(&db).unwrap();
        }

        {
            let mut db_nodes = Nodes::new();
            // Load data from disk and check that it is the same as the original
            db_nodes.load_all_from_disk(&db_path).unwrap();

            for node in nodes.iter() {
                // Check the ID and payload of each node against the one in the DB
                let db_node = db_nodes.data.get(&node.id).unwrap();
                assert_eq!(node.id, db_node.id);
                // Match the payload data type and check the inner values
                match node.payload {
                    Payload::Title(ref title) => {
                        let db_title = match db_node.payload {
                            Payload::Title(ref title) => title,
                            _ => panic!("Expected Title payload"),
                        };
                        assert_eq!(title.0, db_title.0);
                    }
                    Payload::Heading(ref heading) => {
                        let db_heading = match db_node.payload {
                            Payload::Heading(ref heading) => heading,
                            _ => panic!("Expected Heading payload"),
                        };
                        assert_eq!(heading.0, db_heading.0);
                    }
                    Payload::Paragraph(ref paragraph) => {
                        let db_paragraph = match db_node.payload {
                            Payload::Paragraph(ref paragraph) => paragraph,
                            _ => panic!("Expected Paragraph payload"),
                        };
                        assert_eq!(paragraph.0, db_paragraph.0);
                    }
                    Payload::BulletPoints(ref bullet_points) => {
                        let db_bullet_points = match db_node.payload {
                            Payload::BulletPoints(ref bullet_points) => bullet_points,
                            _ => panic!("Expected BulletPoints payload"),
                        };
                        assert_eq!(bullet_points.0, db_bullet_points.0);
                    }
                    Payload::OrderedPoints(ref ordered_points) => {
                        let db_ordered_points = match db_node.payload {
                            Payload::OrderedPoints(ref ordered_points) => ordered_points,
                            _ => panic!("Expected OrderedPoints payload"),
                        };
                        assert_eq!(ordered_points.0, db_ordered_points.0);
                    }
                    Payload::Link(ref link) => {
                        let db_link = match db_node.payload {
                            Payload::Link(ref link) => link,
                            _ => panic!("Expected Link payload"),
                        };
                        assert_eq!(link.path, db_link.path);
                        assert_eq!(link.query, db_link.query);
                    }
                    Payload::Domain(ref domain) => {
                        let db_domain = match db_node.payload {
                            Payload::Domain(ref domain) => domain,
                            _ => panic!("Expected Domain payload"),
                        };
                        assert_eq!(domain.name, db_domain.name);
                        assert_eq!(domain.is_allowed_to_crawl, db_domain.is_allowed_to_crawl);
                    }
                    Payload::FileHTML(ref web_page) => {
                        let db_web_page = match db_node.payload {
                            Payload::FileHTML(ref web_page) => web_page,
                            _ => panic!("Expected FileHTML payload"),
                        };
                        assert_eq!(web_page.contents, db_web_page.contents);
                        assert_eq!(web_page.is_scraped, db_web_page.is_scraped);
                        assert_eq!(web_page.is_classified, db_web_page.is_classified);
                        assert_eq!(web_page.is_extracted, db_web_page.is_extracted);
                    }
                    _ => {}
                }
            }

            let db = DB::open_default(db_path).unwrap();
            db_nodes.save_all_to_disk(&db).unwrap();
        }

        // Check this again
        {
            let mut db_nodes = Nodes::new();
            // Load data from disk and check that it is the same as the original
            db_nodes.load_all_from_disk(&db_path).unwrap();

            for node in nodes.iter() {
                // Check the ID and payload of each node against the one in the DB
                let db_node = db_nodes.data.get(&node.id).unwrap();
                assert_eq!(node.id, db_node.id);
                // Match the payload data type and check the inner values
                match node.payload {
                    Payload::Title(ref title) => {
                        let db_title = match db_node.payload {
                            Payload::Title(ref title) => title,
                            _ => panic!("Expected Title payload"),
                        };
                        assert_eq!(title.0, db_title.0);
                    }
                    Payload::Heading(ref heading) => {
                        let db_heading = match db_node.payload {
                            Payload::Heading(ref heading) => heading,
                            _ => panic!("Expected Heading payload"),
                        };
                        assert_eq!(heading.0, db_heading.0);
                    }
                    Payload::Paragraph(ref paragraph) => {
                        let db_paragraph = match db_node.payload {
                            Payload::Paragraph(ref paragraph) => paragraph,
                            _ => panic!("Expected Paragraph payload"),
                        };
                        assert_eq!(paragraph.0, db_paragraph.0);
                    }
                    Payload::BulletPoints(ref bullet_points) => {
                        let db_bullet_points = match db_node.payload {
                            Payload::BulletPoints(ref bullet_points) => bullet_points,
                            _ => panic!("Expected BulletPoints payload"),
                        };
                        assert_eq!(bullet_points.0, db_bullet_points.0);
                    }
                    Payload::OrderedPoints(ref ordered_points) => {
                        let db_ordered_points = match db_node.payload {
                            Payload::OrderedPoints(ref ordered_points) => ordered_points,
                            _ => panic!("Expected OrderedPoints payload"),
                        };
                        assert_eq!(ordered_points.0, db_ordered_points.0);
                    }
                    Payload::Link(ref link) => {
                        let db_link = match db_node.payload {
                            Payload::Link(ref link) => link,
                            _ => panic!("Expected Link payload"),
                        };
                        assert_eq!(link.path, db_link.path);
                        assert_eq!(link.query, db_link.query);
                    }
                    Payload::Domain(ref domain) => {
                        let db_domain = match db_node.payload {
                            Payload::Domain(ref domain) => domain,
                            _ => panic!("Expected Domain payload"),
                        };
                        assert_eq!(domain.name, db_domain.name);
                        assert_eq!(domain.is_allowed_to_crawl, db_domain.is_allowed_to_crawl);
                    }
                    Payload::FileHTML(ref web_page) => {
                        let db_web_page = match db_node.payload {
                            Payload::FileHTML(ref web_page) => web_page,
                            _ => panic!("Expected FileHTML payload"),
                        };
                        assert_eq!(web_page.contents, db_web_page.contents);
                        assert_eq!(web_page.is_scraped, db_web_page.is_scraped);
                        assert_eq!(web_page.is_classified, db_web_page.is_classified);
                        assert_eq!(web_page.is_extracted, db_web_page.is_extracted);
                    }
                    _ => {}
                }
            }
        }
    }
}
