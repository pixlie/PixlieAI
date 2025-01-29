use super::{api::handle_engine_api_request, Engine};
use crate::{
    config::{gliner::setup_gliner, Settings},
    error::PiResult,
    CommsChannel, PiEvent,
};
use log::debug;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, sleep},
    time::Duration,
};

#[derive(PartialEq, Eq, Hash)]
pub enum JobType {
    SetupGliner,
}

pub fn engine_manager(engine_ch: CommsChannel, api_ch: CommsChannel) -> PiResult<()> {
    let settings: Settings = Settings::get_cli_settings()?;
    let mut engine: Option<Engine> = None;
    let mut jobs: HashMap<JobType, thread::JoinHandle<()>> = HashMap::new();

    // We loop until we receive a SIGTERM or SIGINT signals
    let is_sig_term = Arc::new(AtomicBool::new(false));
    let is_sig_int = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&is_sig_term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&is_sig_int))?;

    if settings.path_to_storage_dir.is_some() && settings.current_project.is_some() {
        engine = {
            Some(Engine::new_with_project(
                &settings.path_to_storage_dir.unwrap(),
                &settings.current_project.unwrap(),
            ))
        };
    }
    while !is_sig_term.load(std::sync::atomic::Ordering::Relaxed)
        && !is_sig_int.load(std::sync::atomic::Ordering::Relaxed)
    {
        if let Some(engine) = engine.as_ref() {
            // let engine = engine.clone();
            engine.execute();
        }
        match engine_ch.rx.try_recv() {
            Ok(res) => match res {
                PiEvent::SettingsUpdated => {
                    let new_settings: Settings = Settings::get_cli_settings()?;
                    if new_settings.path_to_storage_dir.is_some()
                        && new_settings.current_project.is_some()
                    {
                        debug!("Settings changed, reloading engine");
                        engine = Some(Engine::new_with_project(
                            new_settings.path_to_storage_dir.as_ref().unwrap(),
                            new_settings.current_project.as_ref().unwrap(),
                        ));
                    } else {
                        match engine {
                            Some(engine) => {
                                engine.save_to_disk();
                            }
                            None => {}
                        }
                        engine = None;
                    }
                    // settings = new_settings;
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
                    debug!("Got an API request for the engine");
                    match engine {
                        Some(ref mut engine) => {
                            let api_ch1 = api_ch.clone();
                            handle_engine_api_request(api_request, engine, api_ch1).unwrap();
                        }
                        None => {}
                    };
                }
                _ => {}
            },
            Err(_) => {}
        }
    }

    Ok(())
}
