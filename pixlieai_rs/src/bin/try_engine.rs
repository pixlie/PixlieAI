use log::{error, info};
use rand::Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rumqttc::{Client, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

pub type NodeId = Arc<u32>;

#[derive(Deserialize, Serialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub part_node_ids: Vec<NodeId>,
}

pub enum RelationType {
    IsPart,
    IsRelated,
}

pub struct PendingNode {
    pub label: String,
    pub creating_node_id: NodeId,
    pub related_type: RelationType,
}

pub struct Engine {
    pub nodes: HashMap<NodeId, RwLock<Node>>,
    last_node_id: Mutex<u32>,
    nodes_to_write: RwLock<Vec<PendingNode>>,
    // pub nodes_with_label: HashMap<String, Vec<NodeId>>,
}

impl Engine {
    pub fn insert_node(&mut self, label: String) -> NodeId {
        let id = Arc::new({
            let mut id = self.last_node_id.lock().unwrap();
            *id += 1;
            *id
        });
        self.nodes.insert(
            id.clone(),
            RwLock::new(Node {
                id: id.clone(),
                label: label.clone(),
                part_node_ids: vec![],
            }),
        );
        id
    }

    pub fn add_part_node(&self, parent_id: &NodeId, label: String) {
        self.nodes_to_write.write().unwrap().push(PendingNode {
            label,
            creating_node_id: parent_id.clone(),
            related_type: RelationType::IsPart,
        });
    }

    pub fn add_pending_nodes(&mut self) {
        let mut count = 0;
        let mut nodes_to_write: Vec<PendingNode> =
            self.nodes_to_write.write().unwrap().drain(..).collect();
        while let Some(pending_node) = nodes_to_write.pop() {
            let id = self.insert_node(pending_node.label);
            match self.nodes.get(&pending_node.creating_node_id) {
                Some(node) => match pending_node.related_type {
                    RelationType::IsPart => {
                        node.write().unwrap().part_node_ids.push(id);
                    }
                    // RelationType::IsRelated => {
                    //     node.write().unwrap().related_node_ids.push(id);
                    // }
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

    pub fn execute(&mut self) {
        // Loop till we have 10_000_000 nodes
        let mut count = 0;
        loop {
            // For each iteration, add random number of part nodes
            self.nodes.par_iter().for_each(|(node_id, _)| {
                process_node(&self, node_id);
            });
            self.add_pending_nodes();

            if count % 100 == 0 {
                info!("Nodes: {}", self.nodes.len());
            }
            count += 1;
            send_to_mqtt();
            if count > 1 {
                break;
            }
        }
    }
}

fn process_node(engine: &Engine, node_id: &NodeId) {
    // Add a random number of part nodes if total nodes is less than 100_000
    if engine.nodes.len() < 10_000 {
        for i in 0..rand::thread_rng().gen_range(5..10) {
            // Insert a random char as a part node
            engine.add_part_node(
                node_id,
                format!("{}/{}", rand::thread_rng().gen_range(0..26), i),
            );
        }
    } else {
        // Find all part nodes that have the same starting char
        let part_nodes = engine
            .nodes
            .get(node_id)
            .unwrap()
            .read()
            .unwrap()
            .part_node_ids
            .clone();

        let random_chars: Vec<_> = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

        part_nodes
            .iter()
            .filter_map(|nid| match engine.nodes.get(nid) {
                Some(node) => match node.read() {
                    Ok(node) => {
                        if random_chars.contains(&node.label.as_str()) {
                            Some(node.label.clone())
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
                None => None,
            })
            .count();
    }
}

fn send_to_mqtt() {
    let mut mqttoptions = MqttOptions::new("pixlieai", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut connection) = Client::new(mqttoptions, 10);
    client
        .subscribe("pixlieai/named_entities/gliner_request", QoS::AtMostOnce)
        .unwrap();
    thread::spawn(move || {
        client
            .publish(
                "pixlieai/named_entities/gliner_request",
                QoS::AtLeastOnce,
                false,
                "Hello World",
            )
            .unwrap();
    });

    // Iterate to poll the eventloop for connection progress
    for (_, notification) in connection.iter().enumerate() {
        match notification {
            Ok(message) => match message {
                rumqttc::Event::Incoming(rumqttc::Incoming::Publish(publish)) => {
                    if publish.topic == "pixlieai/named_entities/gliner_request" {
                        info!("Notification for MQTT message: {}", publish.topic);
                        break;
                    }
                }
                _ => {}
            },
            Err(err) => {
                error!("Error polling eventloop: {}", err);
            }
        };
    }
}

fn main() {
    env_logger::init();

    let mut engine = Engine {
        nodes: HashMap::new(),
        last_node_id: Mutex::new(0),
        nodes_to_write: RwLock::new(vec![]),
    };
    // Insert 26 initial nodes
    for char in "abcdefghijklmnopqrstuvwxyz".chars() {
        engine.insert_node(format!("{}/0", char));
    }
    engine.execute();
    // info!("Nodes: {}", engine.nodes.len());

    // Parallel iter
    // let count = engine
    //     .nodes
    //     .par_iter()
    //     .map(|(_, v)| v.read().unwrap().label.clone())
    //     .count();

    // info!("Count: {}", count);
}
