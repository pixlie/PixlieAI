// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use chrono::{DateTime, Utc};
use log::info;
use strum::Display;
// use petgraph::graph::NodeIndex;
// use petgraph::{Directed, Graph};
use crate::{
    entity::{
        content::Title,
        web::{CrawledWebPage, Link},
    },
    workers,
};
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
    thread::sleep,
    time::Duration,
};

pub mod api;
// pub mod executor;
// pub mod state;

#[derive(Display)]
pub enum Payload {
    Link(Link),
    FileHTML(CrawledWebPage),
    Title(Title),
}

pub type NodeId = u32;
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
    // pub graph: RwLock<Graph<PiNode, PiEdge, Directed>>,
    pub labels: RwLock<HashSet<String>>,
    pub nodes: RwLock<Vec<Node>>, // All nodes that are in the engine
    pub nodes_to_write: RwLock<Vec<PendingNode>>, // Nodes pending to be written at the end of nodes.iter_mut()
    last_node_id: RwLock<u32>,
    // pub storage_root: String,
    pub nodes_by_label: RwLock<HashMap<String, Vec<NodeId>>>,
    // pub entity_type_last_run: RwLock<HashMap<String, DateTime<Utc>>>,
    pub execute_every: u8, // Number of seconds to wait before executing the engine
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: RwLock::new(vec![]),
            nodes_to_write: RwLock::new(vec![]),
            last_node_id: RwLock::new(0),
            nodes_by_label: RwLock::new(HashMap::new()),
            // entity_type_last_run: RwLock::new(HashMap::new()),
            execute_every: 1,
        }
    }

    pub fn execute(&self) {
        loop {
            // Execute each worker function, passing them the engine
            self.process_nodes();
            self.add_pending_nodes();
            sleep(Duration::from_secs(self.execute_every as u64));
        }
    }

    pub fn add_node(&self, payload: Payload) -> NodeId {
        // Get new ID after incrementing existing node ID
        let label = payload.to_string();
        let id = {
            let mut id = self.last_node_id.write().unwrap();
            *id += 1;
            *id
        };
        // Store the label in the engine
        {
            let mut labels = self.labels.write().unwrap();
            labels.insert(label.clone());
        };
        // Store the node in the engine
        {
            self.nodes.write().unwrap().push(Node {
                id,
                label: label.clone(),
                payload,

                parent_id: None,
                part_node_ids: vec![],
                related_node_ids: vec![],
                written_at: Utc::now(),
            });
        }
        // Store the node in nodes_by_label_id
        {
            let mut nodes_by_label = self.nodes_by_label.write().unwrap();
            nodes_by_label
                .entry(label.clone())
                .and_modify(|entries| entries.push(id))
                .or_insert(vec![id]);
        }
        info!("Added node of type {}", label);
        id
    }

    pub fn add_pending_nodes(&self) {
        while let Some(pending_node) = self.nodes_to_write.write().unwrap().pop() {
            let id = self.add_node(pending_node.payload);
            match pending_node.related_type {
                RelationType::IsPart => {
                    self.nodes
                        .write()
                        .unwrap()
                        .iter_mut()
                        .find(|x| x.id == pending_node.creating_node_id)
                        .unwrap()
                        .part_node_ids
                        .push(id);
                }
                RelationType::IsRelated => {
                    self.nodes
                        .write()
                        .unwrap()
                        .iter_mut()
                        .find(|x| x.id == pending_node.creating_node_id)
                        .unwrap()
                        .related_node_ids
                        .push(id);
                }
            }
        }
    }

    pub fn add_part_node(&self, parent_id: &NodeId, payload: Payload) {
        info!("Will add part node {}", &payload.to_string());
        self.nodes_to_write.write().unwrap().push(PendingNode {
            payload,
            creating_node_id: *parent_id,
            related_type: RelationType::IsPart,
        });
    }

    pub fn add_related_node(&self, parent_id: &NodeId, payload: Payload) {
        info!("Will add related node {}", &payload.to_string());
        self.nodes_to_write.write().unwrap().push(PendingNode {
            payload,
            creating_node_id: *parent_id,
            related_type: RelationType::IsRelated,
        });
    }

    // pub async fn get_nodes_by_label(&self, label: &str) -> Vec<&Node> {

    // }
}
