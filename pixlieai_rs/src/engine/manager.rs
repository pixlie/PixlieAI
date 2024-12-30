use super::Engine;
use crate::{
    config::{gliner::setup_gliner, Settings},
    error::PiResult,
    PiEvent,
};
use log::info;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread::{self, sleep},
    time::Duration,
};

#[derive(PartialEq, Eq, Hash)]
pub enum JobType {
    SetupGliner,
}

pub fn engine_manager(tx: mpsc::Sender<PiEvent>, rx: mpsc::Receiver<PiEvent>) -> PiResult<()> {
    let mut settings: Settings = Settings::get_cli_settings()?;
    let mut engine: Option<Engine> = None;
    let mut jobs: HashMap<JobType, thread::JoinHandle<()>> = HashMap::new();

    // We loop until we receive a SIGTERM or SIGINT signals
    let is_sig_term = Arc::new(AtomicBool::new(false));
    let is_sig_int = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&is_sig_term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&is_sig_int))?;

    while !is_sig_term.load(std::sync::atomic::Ordering::Relaxed)
        && !is_sig_int.load(std::sync::atomic::Ordering::Relaxed)
    {
        if settings.path_to_storage_dir.is_some() && settings.current_project.is_some() {
            engine = {
                let mut storage_dir =
                    PathBuf::from(&settings.path_to_storage_dir.as_ref().unwrap());
                storage_dir.push(format!(
                    "{}.rocksdb",
                    settings.current_project.as_ref().unwrap()
                ));
                Some(Engine::new(storage_dir))
            }
            // startup_funding_insights_app(&mut engine);
        }
        if engine.is_some() {
            // engine.as_mut().unwrap().execute();
        }

        match rx.try_recv() {
            Ok(res) => match res {
                PiEvent::SettingsUpdated => {
                    let new_settings: Settings = Settings::get_cli_settings()?;
                    if new_settings.path_to_storage_dir.is_some()
                        && new_settings.current_project.is_some()
                    {
                        // Check that new settings values are different from old ones
                        if new_settings.path_to_storage_dir.as_ref().unwrap()
                            != settings.path_to_storage_dir.as_ref().unwrap()
                            || new_settings.current_project.as_ref().unwrap()
                                != settings.current_project.as_ref().unwrap()
                        {
                            info!("Settings changed, reloading engine");
                            // TODO: Reload the engine
                            settings = new_settings;
                            engine = Some(Engine::new(PathBuf::from(
                                &settings.path_to_storage_dir.as_ref().unwrap(),
                            )));
                        }
                    } else {
                        // TODO: Stop the engine
                        settings = new_settings;
                        engine = None;
                    }
                }
                PiEvent::SetupGliner => {
                    // Run setup_gliner only if it is not already running
                    if jobs.contains_key(&JobType::SetupGliner) {
                        continue;
                    }
                    let job_tx = tx.clone();
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
            },
            Err(_) => {}
        }

        sleep(Duration::from_secs(1));
    }
    Ok(())
}
