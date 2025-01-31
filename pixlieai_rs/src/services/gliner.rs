// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use super::ExtractionRequest;
use crate::entity::ExtractedEntity;
use crate::error::PiResult;
use log::{error, info};
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
pub struct GlinerEntity {
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub label: String,
    pub score: f32,
}

pub fn extract_entities(text: String, labels: &Vec<String>) -> PiResult<Vec<ExtractedEntity>> {
    let random_id = rand::random::<u32>();

    //  This is where we initiate a request with GLiNER using MQTT
    {
        let random_id = random_id.clone();
        let labels = labels.clone();
        thread::spawn(move || extract_entities_sender(text, &labels, random_id));
    };

    // let mut extracted: Vec<ExtractedEntity> = vec![];
    // This is where we listen for responses from GLiNER using MQTT
    // let mut mqtt_options = MqttOptions::new(
    //     format!("{}_receiver_{}", mqtt_topic, random_id),
    //     "localhost",
    //     1883,
    // );
    // mqtt_options.set_keep_alive(Duration::from_secs(5));

    // let (receiver, mut connection) = Client::new(mqtt_options, 10);
    // receiver
    //     .subscribe(
    //         format!("pixlieai/{}/responses/{}", mqtt_topic, random_id),
    //         QoS::ExactlyOnce,
    //     )
    //     .unwrap();

    // for notification in connection.iter() {
    //     match notification {
    //         Ok(message) => match message {
    //             Event::Incoming(Incoming::Publish(publish)) => {
    //                 match serde_json::from_slice::<Vec<GlinerEntity>>(&publish.payload) {
    //                     Ok(entities) => {
    //                         for entity in entities {
    //                             if entity.text.is_empty() {
    //                                 continue;
    //                             }
    //                             extracted.push(ExtractedEntity {
    //                                 label: entity.label,
    //                                 matching_text: entity.text,
    //                                 start: Some(entity.start),
    //                                 end: Some(entity.end),
    //                                 score: Some(entity.score),
    //                             });
    //                         }
    //                         receiver
    //                             .unsubscribe(format!(
    //                                 "pixlieai/{}/responses/{}",
    //                                 mqtt_topic, random_id
    //                             ))
    //                             .unwrap();
    //                         break;
    //                     }
    //                     Err(err) => {
    //                         error!(
    //                             "Receiver {}: Error deserializing gliner entities: {}",
    //                             random_id, err
    //                         );
    //                     }
    //                 };
    //             }
    //             _ => {}
    //         },
    //         Err(_) => {
    //             break;
    //         }
    //     };
    // }

    Ok(vec![])
}

fn extract_entities_sender(text: String, labels: &Vec<String>, random_id: u32) {
    // let mqtt_topic = "extract_named_entities_gliner";
    // let mut mqtt_options = MqttOptions::new(
    //     format!("{}_sender_{}", mqtt_topic, random_id),
    //     "localhost",
    //     1883,
    // );
    // mqtt_options.set_keep_alive(Duration::from_secs(5));

    // let (sender, mut connection) = Client::new(mqtt_options.clone(), 10);
    // let labels = labels.clone();
    // thread::spawn(move || {
    //     match sender.publish(
    //         format!("pixlieai/{}/requests/{}", mqtt_topic, random_id),
    //         QoS::ExactlyOnce,
    //         false,
    //         serde_json::to_string(&ExtractionRequest { text, labels }).unwrap(),
    //     ) {
    //         Ok(_) => {}
    //         Err(err) => {
    //             error!("Error publishing {}/requests: {}", mqtt_topic, err);
    //         }
    //     }
    // });
    // for notification in connection.iter() {
    //     match notification {
    //         Ok(message) => match message {
    //             Event::Incoming(Incoming::Publish(_)) => {
    //                 info!(
    //                     "Sender {}: Requested entity extraction with GLiNER",
    //                     random_id
    //                 );
    //             }
    //             _ => {}
    //         },
    //         Err(err) => {
    //             error!("Sender {}: {}", random_id, err);
    //             break;
    //         }
    //     };
    // }
}
