// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use super::ExtractionRequest;
use crate::entity::ExtractedEntity;
use crate::error::PiResult;
use log::{error, info};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{Client, Event, Incoming, MqttOptions};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
pub struct GlinerEntity {
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub label: String,
    pub score: f32,
}

pub fn extract_entities(extraction_request: &ExtractionRequest) -> PiResult<Vec<ExtractedEntity>> {
    // We use MQTT to call the Python code that uses GLiNER to extract entities
    let mut mqtt_options = MqttOptions::new("pixlieai", "localhost", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let extracted: Vec<ExtractedEntity> = vec![];
    let (client, mut connetion) = Client::new(mqtt_options, 10);
    client
        .subscribe("pixlieai/extract_entities_gliner", QoS::AtMostOnce)
        .unwrap();
    client
        .publish(
            "gliner/extract_entities",
            QoS::AtMostOnce,
            true,
            serde_json::to_string(&extraction_request).unwrap(),
        )
        .unwrap();
    info!("Published entity extraction with GLiNER request to MQTT server");
    match connetion.recv() {
        Ok(received) => match received {
            Ok(message) => match message {
                Event::Incoming(Incoming::Publish(publish)) => {
                    info!("Received message: {:?}", publish);
                }
                _ => {
                    error!("Received unexpected message: {:?}", message);
                }
            },
            Err(err) => {
                error!("Error receiving message {}", err);
            }
        },
        Err(err) => {
            error!("Error receiving message: {:?}", err);
        }
    };
    Ok(extracted)
}
