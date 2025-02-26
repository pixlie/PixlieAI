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
pub enum PiEvent {
    SettingsUpdated,
    SetupGliner,
    FinishedSetupGliner,

    APIRequest(String, EngineRequest), // Actual payload is share using PiStore
    APIResponse(String, EngineResponse),

    FetchRequest(String, u32, String),
    FetchResponse(String, u32, String, ExternalData),
    FetchError(String, u32, String),

    NeedsToTick,
    TickMeLater(String), // This is sent from engine to main thread
    EngineExit(String),  // The engine has nothing else to do, so it gives up

    Shutdown,
}
