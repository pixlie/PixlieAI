// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::api::{EngineRequest, EngineResponse};
use crossbeam_channel::unbounded;
use reqwest::header::HeaderMap;
use reqwest::Method;
use strum::Display;

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
    Response(FetchResponse),
    Error(FetchError),
}

#[derive(Clone)]
pub struct InternalFetchRequest {
    pub project_id: String,
    pub node_id: u32,
    pub method: Method,
    pub domain: String,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<String>,
}

impl InternalFetchRequest {
    pub fn from_request_to_engine(
        request: FetchRequest,
        project_id: String,
        domain: String,
    ) -> InternalFetchRequest {
        InternalFetchRequest {
            project_id,
            node_id: request.requesting_node_id,
            method: request.method,
            domain,
            url: request.url,
            headers: request.headers,
            body: request.body,
        }
    }
}

pub struct FetchRequest {
    pub requesting_node_id: u32,
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<String>,
}

impl FetchRequest {
    pub fn new(node_id: u32, url: &str) -> FetchRequest {
        FetchRequest {
            requesting_node_id: node_id,
            method: Method::GET,
            url: url.to_string(),
            headers: HeaderMap::new(),
            body: None,
        }
    }
}

#[derive(Clone)]
pub struct FetchResponse {
    pub project_id: String,
    pub node_id: u32,
    pub url: String,
    pub contents: String,
}

#[derive(Clone)]
pub struct FetchError {
    pub project_id: String,
    pub node_id: u32,
    pub error: String,
}

#[derive(Clone, Display)]
pub enum PiEvent {
    SettingsUpdated,
    SetupGliner,
    FinishedSetupGliner,

    APIRequest(String, EngineRequest), // Actual payload is share using PiStore
    APIResponse(String, EngineResponse),

    FetchRequest(InternalFetchRequest),
    FetchResponse(FetchResponse),
    FetchError(FetchError),

    NeedsToTick,
    // TickMeLater(String), // This is sent from engine to main thread
    EngineExit(String), // The engine has nothing else to do, so it gives up

    Shutdown,
}
