use log::{debug, error};
// use pixlie_ai::config::gliner::setup_gliner;
use pixlie_ai::api::APIChannel;
use pixlie_ai::config::Settings;
use pixlie_ai::engine::Engine;
use pixlie_ai::utils::fetcher::Fetcher;
use pixlie_ai::{api::api_manager, config::check_cli_settings, PiChannel, PiEvent};
use std::collections::HashMap;
use std::env::var;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() {
    // Setup Sentry for error logging. The URL comes from environment variable
    match var("SENTRY_URL") {
        Ok(sentry_url) => {
            if sentry_url.contains("sentry.io") {
                let _sentry = sentry::init((
                    sentry_url,
                    sentry::ClientOptions {
                        release: sentry::release_name!(),
                        ..Default::default()
                    },
                ));
            }
        }
        Err(_) => {}
    }

    env_logger::init();
    match check_cli_settings() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with settings check: {}", err);
            return;
        }
    }
    let pool = threadpool::Builder::new()
        .thread_name("pixlie_ai_worker".to_string())
        .build();

    let main_channel = PiChannel::new();
    let api_channel = APIChannel::new();
    {
        let main_channel = main_channel.clone();
        let api_channel_rx = api_channel.tx.clone();
        // The receiver is in async code, so we use an async channel for that
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code
        pool.execute(move || match api_manager(main_channel.tx, api_channel_rx) {
            Ok(_) => {}
            Err(err) => {
                error!("Error with api manager: {}", err);
            }
        });
    }

    let fetcher = Fetcher::new();
    let arced_fetcher = Arc::new(fetcher);

    // Engines for each project, key being the project ID
    let mut projects: HashMap<String, PiChannel> = HashMap::new();

    // We loop until we receive a SIGTERM or SIGINT signals
    let is_sig_term = Arc::new(AtomicBool::new(false));
    let is_sig_int = Arc::new(AtomicBool::new(false));
    match signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&is_sig_term)) {
        Ok(_) => {}
        Err(err) => {
            error!("Error registering SIGTERM: {}", err);
        }
    }
    match signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&is_sig_int)) {
        Ok(_) => {}
        Err(err) => {
            error!("Error registering SIGINT: {}", err);
        }
    }

    while !is_sig_term.load(std::sync::atomic::Ordering::Relaxed)
        && !is_sig_int.load(std::sync::atomic::Ordering::Relaxed)
    {
        let pi_channel_main = main_channel.clone();
        match pi_channel_main.rx.try_recv() {
            Ok(event) => match event {
                PiEvent::SettingsUpdated => {}
                PiEvent::APIRequest(project_id, request) => {
                    match projects.contains_key(&project_id) {
                        true => {
                            // Project is already loaded, we will send the API request to the engine
                        }
                        false => {
                            // Project is not loaded, let's load it into an engine
                            debug!(
                                "Received API request for project {} which is not loaded",
                                project_id
                            );
                            let settings: Settings = match Settings::get_cli_settings() {
                                Ok(settings) => settings,
                                Err(err) => {
                                    error!("Error reading settings: {}", err);
                                    continue;
                                }
                            };

                            let my_pi_channel = PiChannel::new();
                            projects.insert(project_id.clone(), my_pi_channel.clone());
                            let pi_channel_tx = pi_channel_main.clone().tx;
                            let project_id = project_id.clone();
                            let arced_fetcher = arced_fetcher.clone();
                            pool.execute(move || {
                                let engine = Engine::open_project(
                                    settings.path_to_storage_dir.as_ref().unwrap(),
                                    &project_id,
                                    arced_fetcher,
                                );
                                engine.tick(my_pi_channel, pi_channel_tx);
                            });
                        }
                    };
                    // Engine is loaded, we will pass the API request to the engine's own channel
                    match projects.get(&project_id) {
                        Some(my_pi_channel) => {
                            match my_pi_channel
                                .tx
                                .send(PiEvent::APIRequest(project_id.clone(), request))
                            {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Error sending PiEvent in Engine: {}", err);
                                }
                            }
                        }
                        None => {
                            error!("Project {} is not loaded", project_id);
                            continue;
                        }
                    };
                }
                PiEvent::APIResponse(project_id, response) => {
                    // Pass on the response to the API broadcast channel
                    match api_channel
                        .tx
                        .send(PiEvent::APIResponse(project_id, response))
                    {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error sending PiEvent in API broadcast channel: {}", err);
                        }
                    }
                }
                PiEvent::EngineTicked(project_id) => {
                    projects.remove(&project_id);
                }
                _ => {}
            },
            Err(_) => {}
        }
    }
}

// #[derive(PartialEq, Eq, Hash)]
// pub enum JobType {
//     SetupGliner,
// }

// pub fn engine_manager(engine_ch: PiChannel, api_ch: PiChannel) -> PiResult<()> {
//     // The engine manager runs the engine for each open project that needs processing or API response
//     // let mut jobs: HashMap<JobType, thread::JoinHandle<()>> = HashMap::new();
//     // Engines for each project, key being the project ID
//     let mut engines: HashMap<String, LockedEngine> = HashMap::new();
//
//     // We loop until we receive a SIGTERM or SIGINT signals
//     let is_sig_term = Arc::new(AtomicBool::new(false));
//     let is_sig_int = Arc::new(AtomicBool::new(false));
//     signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&is_sig_term))?;
//     signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&is_sig_int))?;
//
//     while !is_sig_term.load(std::sync::atomic::Ordering::Relaxed)
//         && !is_sig_int.load(std::sync::atomic::Ordering::Relaxed)
//     {
//         for project_id in engines.keys() {
//             let engine = engines.get(project_id).unwrap();
//             match engine.read() {
//                 Ok(engine) => {
//                     if engine.needs_to_tick() {
//                         debug!("Ticking engine for project {}", project_id);
//                         engine.tick();
//                     }
//                 }
//                 Err(_err) => {
//                     error!("Error reading engine for project {}", project_id);
//                 }
//             }
//         }
//
//         match engine_ch.rx.try_recv() {
//             Ok(res) => match res {
//                 PiEvent::SettingsUpdated => {
//                     let settings: Settings = Settings::get_cli_settings()?;
//                     debug!("Settings changed, reloading engine");
//                     // Reload each engine
//                     for (project_id, engine) in engines.iter() {
//                         match engine.write() {
//                             Ok(mut engine) => {
//                                 *engine = Engine::open_project(
//                                     settings.path_to_storage_dir.as_ref().unwrap(),
//                                     project_id,
//                                 );
//                             }
//                             Err(_err) => {}
//                         }
//                     }
//                 }
//                 PiEvent::SetupGliner => {
//                     // Run setup_gliner only if it is not already running
//                     if jobs.contains_key(&JobType::SetupGliner) {
//                         continue;
//                     }
//                     let job_tx = engine_ch.tx.clone();
//                     jobs.insert(
//                         JobType::SetupGliner,
//                         thread::spawn(move || {
//                             setup_gliner(job_tx).unwrap();
//                         }),
//                     );
//                 }
//                 PiEvent::FinishedSetupGliner => {
//                     if let Some(job) = jobs.remove(&JobType::SetupGliner) {
//                         job.join().unwrap();
//                     }
//                 }
//                 PiEvent::APIRequest(project_id, request) => {
//                     debug!("Got an API request for project {}", project_id);
//                     match engines.get(&project_id) {
//                         Some(engine) => {
//                             let api_ch1 = api_ch.clone();
//                             handle_engine_api_request(request, engine, api_ch1)?;
//                         }
//                         None => {
//                             // Project is not loaded, let's load it into an engine
//                             {
//                                 debug!("Project {} is not loaded, loading it", project_id);
//                                 let settings: Settings = Settings::get_cli_settings()?;
//                                 let engine = RwLock::new(Engine::open_project(
//                                     settings.path_to_storage_dir.as_ref().unwrap(),
//                                     &project_id,
//                                 ));
//                                 engines.insert(project_id.clone(), engine);
//                             }
//                             let api_ch1 = api_ch.clone();
//                             let engine = engines.get(&api_request.project_id).unwrap();
//                             handle_engine_api_request(api_request, &engine, api_ch1)?;
//                         }
//                     };
//                 }
//                 _ => {}
//             },
//             Err(_) => {}
//         }
//     }
//
//     Ok(())
// }
