// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::{
    config::Rule,
    entity::{
        content::{Heading, Paragraph, Table, TableRow, Title},
        web::{Domain, Link, WebPage},
    },
};
use chrono::{DateTime, Utc};
use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
    thread::sleep,
    time::Duration,
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
    Table(Table),
    TableRow(TableRow),
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
    // pub storage_root: String,
    pub nodes_by_label: RwLock<HashMap<String, Vec<NodeId>>>,
    // pub entity_type_last_run: RwLock<HashMap<String, DateTime<Utc>>>,
    pub execute_every: u8, // Number of seconds to wait before executing the engine
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: HashMap::new(),
            nodes_to_write: RwLock::new(vec![]),
            last_node_id: Mutex::new(0),
            nodes_by_label: RwLock::new(HashMap::new()),
            execute_every: 1,
        }
    }

    pub fn execute(&mut self) {
        loop {
            // Execute each worker function, passing them the engine
            self.process_nodes();
            self.add_pending_nodes();
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
            let id = self.add_node(pending_node.payload);
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
}

pub trait NodeWorker {
    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<Self>
    where
        Self: Sized;
}
