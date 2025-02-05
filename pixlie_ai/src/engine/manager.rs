use super::{api::handle_engine_api_request, Engine};
use crate::engine::engine::LockedEngine;
use crate::{
    config::{gliner::setup_gliner, Settings},
    error::PiResult,
    CommsChannel, PiEvent,
};
use log::{debug, error};
use std::sync::RwLock;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    thread::{self},
};

#[derive(PartialEq, Eq, Hash)]
pub enum JobType {
    SetupGliner,
}

pub fn engine_manager(engine_ch: CommsChannel, api_ch: CommsChannel) -> PiResult<()> {
    // The engine manager runs the engine for each open project that needs processing or API response
    let mut jobs: HashMap<JobType, thread::JoinHandle<()>> = HashMap::new();
    // Engines for each project, key being the project ID
    let mut engines: HashMap<String, LockedEngine> = HashMap::new();

    // We loop until we receive a SIGTERM or SIGINT signals
    let is_sig_term = Arc::new(AtomicBool::new(false));
    let is_sig_int = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&is_sig_term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&is_sig_int))?;

    while !is_sig_term.load(std::sync::atomic::Ordering::Relaxed)
        && !is_sig_int.load(std::sync::atomic::Ordering::Relaxed)
    {
        for project_id in engines.keys() {
            let engine = engines.get(project_id).unwrap();
            match engine.read() {
                Ok(engine) => {
                    if engine.needs_to_tick() {
                        debug!("Ticking engine for project {}", project_id);
                        // let engine = engine.clone();
                        engine.tick();
                    }
                }
                Err(_err) => {
                    error!("Error reading engine for project {}", project_id);
                }
            }
        }

        match engine_ch.rx.try_recv() {
            Ok(res) => match res {
                PiEvent::SettingsUpdated => {
                    let settings: Settings = Settings::get_cli_settings()?;
                    debug!("Settings changed, reloading engine");
                    // Reload each engine
                    for (project_id, engine) in engines.iter() {
                        match engine.write() {
                            Ok(mut engine) => {
                                *engine = Engine::open_project(
                                    settings.path_to_storage_dir.as_ref().unwrap(),
                                    project_id,
                                );
                            }
                            Err(_err) => {}
                        }
                    }
                }
                PiEvent::SetupGliner => {
                    // Run setup_gliner only if it is not already running
                    if jobs.contains_key(&JobType::SetupGliner) {
                        continue;
                    }
                    let job_tx = engine_ch.tx.clone();
                    jobs.insert(
                        JobType::SetupGliner,
                        thread::spawn(move || {
                            setup_gliner(job_tx).unwrap();
                        }),
                    );
                }
                PiEvent::FinishedSetupGliner => {
                    if let Some(job) = jobs.remove(&JobType::SetupGliner) {
                        job.join().unwrap();
                    }
                }
                PiEvent::EngineRequest(api_request) => {
                    debug!("Got an API request for project {}", api_request.project_id);
                    match engines.get(&api_request.project_id) {
                        Some(engine) => {
                            let api_ch1 = api_ch.clone();
                            handle_engine_api_request(api_request, engine, api_ch1)?;
                        }
                        None => {
                            // Project is not loaded, let's load it into an engine
                            {
                                debug!(
                                    "Project {} is not loaded, loading it",
                                    api_request.project_id
                                );
                                let settings: Settings = Settings::get_cli_settings()?;
                                let engine = RwLock::new(Engine::open_project(
                                    settings.path_to_storage_dir.as_ref().unwrap(),
                                    &api_request.project_id,
                                ));
                                engines.insert(api_request.project_id.clone(), engine);
                            }
                            let api_ch1 = api_ch.clone();
                            let engine = engines.get(&api_request.project_id).unwrap();
                            handle_engine_api_request(api_request, &engine, api_ch1)?;
                        }
                    };
                }
                _ => {}
            },
            Err(_) => {}
        }
    }

    Ok(())
}
