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
pub mod workspace;

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

#[derive(Clone)]
pub enum ExternalData {
    Text(String),
}

#[derive(Clone)]
pub struct FetchRequest {
    pub project_id: String,
    pub node_id: u32,
    pub domain: String,
    pub url: String,
}

#[derive(Clone)]
pub struct FetchResponse {
    pub project_id: String,
    pub node_id: u32,
    pub url: String,
    pub contents: ExternalData,
}

#[derive(Clone)]
pub struct FetchError {
    pub project_id: String,
    pub node_id: u32,
    pub error: String,
}

#[derive(Clone)]
pub enum PiEvent {
    SettingsUpdated,
    SetupGliner,
    FinishedSetupGliner,

    APIRequest(String, EngineRequest), // Actual payload is share using PiStore
    APIResponse(String, EngineResponse),

    FetchRequest(FetchRequest),
    FetchResponse(FetchResponse),
    FetchError(FetchError),

    NeedsToTick,
    TickMeLater(String), // This is sent from engine to main thread
    EngineExit(String),  // The engine has nothing else to do, so it gives up

    Shutdown,
}
