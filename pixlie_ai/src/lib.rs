// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crossbeam_channel::unbounded;
use engine::api::{EngineRequestMessage, EngineResponseMessage};

pub mod api;
pub mod config;
pub mod engine;
pub mod entity;
pub mod error;
pub mod projects;
pub mod services;
pub mod utils;

#[derive(Clone)]
pub struct CommsChannel {
    tx: crossbeam_channel::Sender<PiEvent>,
    rx: crossbeam_channel::Receiver<PiEvent>,
}

impl CommsChannel {
    pub fn new() -> CommsChannel {
        let (tx, rx) = unbounded::<PiEvent>();
        CommsChannel { tx, rx }
    }
}

pub enum PiEvent {
    SettingsUpdated,
    SetupGliner,
    FinishedSetupGliner,
    EngineRequest(EngineRequestMessage),
    EngineResponse(EngineResponseMessage),
}
