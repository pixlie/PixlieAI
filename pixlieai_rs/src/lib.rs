// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

pub mod admin;
pub mod api;
pub mod config;
pub mod engine;
pub mod entity;
pub mod error;
pub mod services;

pub enum PiEvent {
    SettingsUpdated,
    SetupGliner,
    FinishedSetupGliner,
}
