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
use rumqttc::v5::{Client, ConnectionError, Event, Incoming, MqttOptions};
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

pub fn extract_entities(extraction_request: ExtractionRequest) -> PiResult<Vec<ExtractedEntity>> {
    // We use MQTT to call the Python code that uses GLiNER to extract entities
    let mqtt_topic = "extract_named_entities_gliner";

    //  This is where we initiate a request with GLiNER using MQTT
    thread::spawn(move || {
        let mut mqtt_options =
            MqttOptions::new(format!("{}_requests", mqtt_topic), "localhost", 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(5));

        let (pubisher, mut connection) = Client::new(mqtt_options.clone(), 10);
        match pubisher.publish(
            format!("pixlieai/{}/requests", mqtt_topic),
            QoS::ExactlyOnce,
            false,
            serde_json::to_string(&extraction_request).unwrap(),
        ) {
            Ok(_) => {}
            Err(err) => {
                error!("Error publishing {}/requests: {}", mqtt_topic, err);
            }
        }
        for notification in connection.iter() {
            match notification {
                Ok(message) => match message {
                    Event::Incoming(Incoming::Publish(_)) => {
                        info!("Published entity extraction with GLiNER request to MQTT server");
                        break;
                    }
                    _ => {}
                },
                Err(_) => {}
            };
        }
    });

    // This is where we listen for responses from GLiNER using MQTT
    let mut mqtt_options = MqttOptions::new(format!("{}_responses", mqtt_topic), "localhost", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let mut extracted: Vec<ExtractedEntity> = vec![];
    let (subscriber, mut connection) = Client::new(mqtt_options, 10);
    subscriber
        .subscribe(
            format!("pixlieai/{}/responses", mqtt_topic),
            QoS::ExactlyOnce,
        )
        .unwrap();

    for notification in connection.iter() {
        match notification {
            Ok(message) => match message {
                Event::Incoming(Incoming::Publish(publish)) => {
                    match serde_json::from_slice::<Vec<GlinerEntity>>(&publish.payload) {
                        Ok(entities) => {
                            for entity in entities {
                                extracted.push(ExtractedEntity {
                                    label: entity.label,
                                    matching_text: entity.text,
                                    start: Some(entity.start),
                                    end: Some(entity.end),
                                    score: Some(entity.score),
                                });
                            }
                        }
                        Err(err) => {
                            error!("Error deserializing gliner entities: {}", err);
                        }
                    };
                    break;
                }
                _ => {}
            },
            Err(_) => {}
        };
    }

    info!(
        "Extracted entities: {}",
        extracted
            .iter()
            .map(|x| x.label.clone())
            .collect::<Vec<String>>()
            .join("\n")
    );
    Ok(extracted)
}
