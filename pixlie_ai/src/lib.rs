// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::api::{EngineRequest, EngineResponse};
use crossbeam_channel::unbounded;

pub mod api;
pub mod config;
pub mod engine;
pub mod entity;
pub mod error;
pub mod projects;
pub mod services;
pub mod utils;

#[derive(Clone)]
pub struct PiChannel {
    pub tx: crossbeam_channel::Sender<PiEvent>,
    pub rx: crossbeam_channel::Receiver<PiEvent>,
}

impl PiChannel {
    pub fn new() -> PiChannel {
        let (tx, rx) = unbounded::<PiEvent>();
        PiChannel { tx, rx }
    }
}

pub struct APIChannel {
    pub tx: tokio::sync::broadcast::Sender<PiEvent>,
    pub rx: tokio::sync::broadcast::Receiver<PiEvent>,
}

impl APIChannel {
    pub fn new() -> APIChannel {
        let (tx, rx) = tokio::sync::broadcast::channel::<PiEvent>(100);
        APIChannel { tx, rx }
    }
}

#[derive(Clone)]
pub enum PiEvent {
    NeedsToTick,
    SettingsUpdated,
    SetupGliner,
    FinishedSetupGliner,
    EngineTicked(String), // The engine has nothing else to do, so it gives up
    APIRequest(String, EngineRequest), // Actual payload is share using PiStore
    APIResponse(String, EngineResponse),
    FetchRequest(String),
    FetchResponse(String),
}
