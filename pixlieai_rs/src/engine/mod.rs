// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::{
    config::{get_cli_settings, startup_funding_insights_app, Rule},
    entity::{
        content::{BulletPoints, Heading, OrderedPoints, Paragraph, Table, TableRow, Title},
        web::{Domain, Link, WebPage},
    },
    error::PiResult,
};
use chrono::{DateTime, Utc};
use log::{error, info};
use postcard::{from_bytes, to_allocvec};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
    thread::sleep,
    time::Duration,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use strum::Display;

// pub mod api;

#[derive(Display, Deserialize, Serialize)]
pub enum Payload {
    Rule(Rule),
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
    NamedEntity(String, String), // label, text
}

pub type NodeId = Arc<u32>;

#[derive(Deserialize, Serialize)]
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

// The engine keeps track of all the data nodes and their relationships
// All the entity labels are loaded in the engine
// All data may not be loaded in the engine, some of them may be on disk
pub struct Engine {
    pub labels: RwLock<HashSet<String>>,
    pub nodes: HashMap<NodeId, RwLock<Node>>, // All nodes that are in the engine
    nodes_to_write: RwLock<Vec<PendingNode>>, // Nodes pending to be written at the end of nodes.iter_mut()
    last_node_id: Mutex<u32>,
    storage_root: String,
    pub nodes_by_label: RwLock<HashMap<String, Vec<NodeId>>>,
    // pub entity_type_last_run: RwLock<HashMap<String, DateTime<Utc>>>,
    pub execute_every: u8, // Number of seconds to wait before executing the engine
}

impl Engine {
    pub fn new(storage_root: PathBuf) -> Engine {
        let mut engine = Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: HashMap::new(),
            nodes_to_write: RwLock::new(vec![]),
            last_node_id: Mutex::new(0),
            storage_root: storage_root.to_str().unwrap().to_string(),
            nodes_by_label: RwLock::new(HashMap::new()),
            execute_every: 1,
        };
        // We load the graph from disk
        engine.load_from_disk();
        info!(
            "There are {} webpage nodes in the graph",
            engine
                .nodes
                .iter()
                .filter(|x| match x.1.read().unwrap().payload {
                    Payload::FileHTML(_) => true,
                    _ => false,
                })
                .count()
        );
        engine
    }

    pub fn execute(&mut self) {
        loop {
            // Execute each worker function, passing them the engine
            self.process_nodes();
            self.add_pending_nodes();
            self.save_to_disk();
            sleep(Duration::from_secs(self.execute_every as u64));
        }
    }

    pub fn process_nodes(&self) {
        let updates: Vec<(NodeId, Option<Payload>)> = self
            .nodes
            .par_iter()
            .map(|(node_id, node)| {
                let node = node.read().unwrap();
                let node_id = node_id.clone();
                match node.payload {
                    Payload::Link(ref payload) => {
                        let update = payload.process(self, &node_id);
                        match update {
                            Some(payload) => (node_id, Some(Payload::Link(payload))),
                            None => (node_id, None),
                        }
                    }
                    Payload::FileHTML(ref payload) => {
                        let update = payload.process(self, &node_id);
                        match update {
                            Some(payload) => (node_id, Some(Payload::FileHTML(payload))),
                            None => (node_id, None),
                        }
                    }
                    _ => (node_id, None),
                }
            })
            .collect();
        for (node_id, update) in updates {
            match update {
                Some(update) => {
                    self.nodes.get(&node_id).unwrap().write().unwrap().payload = update;
                }
                None => {}
            };
        }
    }

    pub fn add_node(&mut self, payload: Payload) -> NodeId {
        // Get new ID after incrementing existing node ID
        let label = payload.to_string();
        let id = Arc::new({
            let mut id = self.last_node_id.lock().unwrap();
            *id += 1;
            *id
        });
        // Store the label in the engine
        {
            let mut labels = self.labels.write().unwrap();
            labels.insert(label.clone());
        };
        // Store the node in the engine
        {
            self.nodes.insert(
                id.clone(),
                RwLock::new(Node {
                    id: id.clone(),
                    label: label.clone(),
                    payload,

                    parent_id: None,
                    part_node_ids: vec![],
                    related_node_ids: vec![],
                    written_at: Utc::now(),
                }),
            );
        }
        // Store the node in nodes_by_label_id
        {
            let mut nodes_by_label = self.nodes_by_label.write().unwrap();
            nodes_by_label
                .entry(label.clone())
                .and_modify(|entries| entries.push(id.clone()))
                .or_insert(vec![id.clone()]);
        }
        id
    }

    pub fn add_pending_nodes(&mut self) {
        let mut count = 0;
        let mut nodes_to_write: Vec<PendingNode> =
            self.nodes_to_write.write().unwrap().drain(..).collect();
        while let Some(pending_node) = nodes_to_write.pop() {
            let id = if let Some(existing_node_id) = self.find_existing(&pending_node.payload) {
                // If the payload already exists in the graph, we simply add a relation edge
                existing_node_id
            } else {
                self.add_node(pending_node.payload)
            };
            // Add a relation edge or part edge from the parent node to the new node
            match self.nodes.get(&pending_node.creating_node_id) {
                Some(node) => match pending_node.related_type {
                    RelationType::IsPart => {
                        node.write().unwrap().part_node_ids.push(id.clone());
                    }
                    RelationType::IsRelated => {
                        node.write().unwrap().related_node_ids.push(id.clone());
                    }
                },
                None => {}
            };
            // Add a relation edge from the new node to the parent node
            match self.nodes.get(&id) {
                Some(node) => match pending_node.related_type {
                    // RelationType::IsPart => {
                    //     node.write().unwrap().part_node_ids.push(pending_node.creating_node_id);
                    // }
                    RelationType::IsRelated => {
                        node.write()
                            .unwrap()
                            .related_node_ids
                            .push(pending_node.creating_node_id);
                    }
                    _ => {}
                },
                None => {}
            };
            count += 1;
        }
        if count > 0 {
            info!("Added {} nodes", count);
        }
    }

    pub fn add_part_node(&self, parent_id: &NodeId, payload: Payload) {
        self.nodes_to_write.write().unwrap().push(PendingNode {
            payload,
            creating_node_id: parent_id.clone(),
            related_type: RelationType::IsPart,
        });
    }

    pub fn add_related_node(&self, parent_id: &NodeId, payload: Payload) {
        self.nodes_to_write.write().unwrap().push(PendingNode {
            payload,
            creating_node_id: parent_id.clone(),
            related_type: RelationType::IsRelated,
        });
    }

    fn find_existing(&self, payload: &Payload) -> Option<Arc<u32>> {
        // For certain node payloads, check if there is a node with the same payload
        match payload {
            Payload::Domain(ref domain) => {
                // We do not want duplicate domains in the graph
                self.nodes.par_iter().find_map_any(|other_node| {
                    match other_node.1.read().unwrap().payload {
                        Payload::Domain(ref other_domain) => {
                            if domain == other_domain {
                                Some(other_node.0.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
            }
            Payload::Link(ref link) => {
                // We do not want duplicate links in the graph
                self.nodes.par_iter().find_map_any(|other_node| {
                    match other_node.1.read().unwrap().payload {
                        Payload::Link(ref other_link) => {
                            if link == other_link {
                                Some(other_node.0.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
            }
            Payload::Label(ref label) => {
                // We do not want duplicate labels in the graph
                self.nodes.par_iter().find_map_any(|other_node| {
                    match other_node.1.read().unwrap().payload {
                        Payload::Label(ref other_label) => {
                            if label == other_label {
                                Some(other_node.0.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
            }
            Payload::NamedEntity(ref label, ref text) => {
                // We do not want duplicate named entities in the graph
                self.nodes.par_iter().find_map_any(|other_node| {
                    match other_node.1.read().unwrap().payload {
                        Payload::NamedEntity(ref other_label, ref other_text) => {
                            if label == other_label && text == other_text {
                                Some(other_node.0.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
            }
            _ => None,
        }
    }

    pub fn save_to_disk(&self) {
        // We use RocksDB to store the graph
        let mut path = PathBuf::from(&self.storage_root);
        path.push("pixlieai.rocksdb");
        let db = DB::open_default(path).unwrap();
        for (node_id, node) in self.nodes.iter() {
            let bytes = match node.read() {
                Ok(node) => match to_allocvec(&*node) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        error!("Error serializing node: {}", err);
                        break;
                    }
                },
                Err(err) => {
                    error!("Error reading node: {}", err);
                    break;
                }
            };
            match db.put(node_id.to_le_bytes(), bytes) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error writing node: {}", err);
                    break;
                }
            }
        }
    }

    pub fn load_from_disk(&mut self) {
        let mut path = PathBuf::from(&self.storage_root);
        path.push("pixlieai.rocksdb");
        let db = DB::open_default(path).unwrap();
        let iter = db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, value) = match item {
                Ok(item) => item,
                Err(err) => {
                    error!("Error iterating over RocksDB: {}", err);
                    break;
                }
            };
            let node_id = Arc::new(read_le_u32(&mut &*key));
            let node: Node = match from_bytes(&value) {
                Ok(node) => node,
                Err(err) => {
                    error!("Error deserializing node: {}", err);
                    break;
                }
            };
            let label = node.payload.to_string().clone();
            self.nodes.insert(node_id.clone(), RwLock::new(node));

            // Store the node in nodes_by_label_id
            {
                let mut nodes_by_label = self.nodes_by_label.write().unwrap();
                nodes_by_label
                    .entry(label)
                    .and_modify(|entries| entries.push(node_id.clone()))
                    .or_insert(vec![node_id.clone()]);
            }
        }
    }
}

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}

pub trait NodeWorker {
    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<Self>
    where
        Self: Sized;
}

pub fn engine_manager() -> PiResult<()> {
    let settings = get_cli_settings()?;
    let mut storage_dir = PathBuf::from(&settings.path_to_storage_root.unwrap());
    let project_name = settings.current_project.unwrap();
    storage_dir.push(format!("{}.rocksdb", project_name));
    let mut engine = Engine::new(storage_dir);
    startup_funding_insights_app(&mut engine);
    engine.execute();
    Ok(())
}
