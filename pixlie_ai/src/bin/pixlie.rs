use log::{debug, error};
// use pixlie_ai::config::gliner::setup_gliner;
use pixlie_ai::api::APIChannel;
use pixlie_ai::config::Settings;
use pixlie_ai::engine::Engine;
use pixlie_ai::utils::fetcher::fetcher_runtime;
use pixlie_ai::{api::api_manager, config::check_cli_settings, PiChannel, PiEvent};
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();

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
                debug!("Sentry initialized for this CLI application, you can see errors at https://pixlie.sentry.io/issues/?project=4508832865648720");
            }
        }
        Err(_) => {}
    }

    match check_cli_settings() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with settings check: {}", err);
            return;
        }
    }
    let settings: Settings = match Settings::get_cli_settings() {
        Ok(settings) => settings,
        Err(err) => {
            error!("Error reading settings: {}", err);
            exit(1);
        }
    };

    let pool = threadpool::Builder::new()
        .thread_name("pixlie_ai_thread".to_string())
        .build();

    // This channel is used by the CLI (this main function)
    let main_channel = PiChannel::new();
    // Each engine has its own communication channel
    let mut channels_per_project: HashMap<String, PiChannel> = HashMap::new();
    let mut tick_requests_per_project: HashSet<String> = HashSet::new();
    // The API channel is used by the API server and the CLI
    let api_channel = APIChannel::new();
    let main_channel_tx = main_channel.tx.clone();
    {
        let api_channel_rx = api_channel.tx.clone();
        // The receiver is in async code, so we use an async channel for that
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code
        pool.execute(
            move || match api_manager(main_channel_tx.clone(), api_channel_rx) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error with api manager: {}", err);
                }
            },
        );
    }

    // https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code
    let (fetcher_tx, fetcher_rx) = tokio::sync::mpsc::channel::<PiEvent>(100);
    let main_channel_tx = main_channel.tx.clone();
    pool.execute(move || {
        fetcher_runtime(fetcher_rx, main_channel_tx);
    });

    let main_channel_tx = main_channel.tx.clone();
    pool.execute(move || {
        // We monitor for SIGTERM or SIGINT signals and send an event to the main channel
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
            // Do nothing till we receive a SIGTERM or SIGINT signal
        }
        match main_channel_tx.send(PiEvent::Shutdown) {
            Ok(_) => {}
            Err(err) => {
                error!("Error sending PiEvent::Shutdown: {}", err);
            }
        }
    });

    let main_channel_iter = main_channel.clone();
    for event in main_channel_iter.rx.iter() {
        match event {
            PiEvent::SettingsUpdated => {}
            PiEvent::APIRequest(project_id, request) => {
                match channels_per_project.contains_key(&project_id) {
                    true => {
                        // Project is already loaded, we will send the API request to the engine
                    }
                    false => {
                        // Project is not loaded, let's load it into an engine
                        debug!(
                            "Received API request for project {} which is not loaded",
                            project_id
                        );

                        channels_per_project.insert(project_id.clone(), PiChannel::new());
                        let my_pi_channel = match channels_per_project.get(&project_id) {
                            Some(my_pi_channel) => my_pi_channel.clone(),
                            None => {
                                error!("Cannot find per engine channel for project {}", project_id);
                                continue;
                            }
                        };
                        let pi_channel_tx = main_channel_iter.clone().tx;
                        let project_id = project_id.clone();
                        let path_to_storage_dir = match settings.path_to_storage_dir {
                            Some(ref path) => path.clone(),
                            None => {
                                error!("Cannot find path to storage directory");
                                continue;
                            }
                        };
                        let fetcher_tx = fetcher_tx.clone();
                        pool.execute(move || {
                            let engine = Engine::open_project(
                                &path_to_storage_dir,
                                &project_id,
                                my_pi_channel.clone(),
                                pi_channel_tx,
                                fetcher_tx,
                            );
                            engine.run();
                        });
                    }
                };
                // Engine is loaded, we will pass the API request to the engine's own channel
                match channels_per_project.get(&project_id) {
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
            PiEvent::TickMeLater(project_id) => {
                // The engine has requested to be called later
                let channels_per_project = channels_per_project.clone();
                if !tick_requests_per_project.contains(&project_id) {
                    tick_requests_per_project.insert(project_id.clone());
                    debug!("TickMeLater for engine for project {}", &project_id);
                    pool.execute(move || {
                        thread::sleep(Duration::from_millis(1000));
                        match channels_per_project.get(&project_id) {
                            Some(channel) => match channel.tx.send(PiEvent::NeedsToTick) {
                                Ok(_) => {
                                    debug!(
                                        "Sent PiEvent::NeedsToTick to engine for project {}",
                                        &project_id
                                    );
                                }
                                Err(err) => {
                                    error!("Error sending PiEvent::NeedsToTick in Engine: {}", err);
                                }
                            },
                            None => {
                                error!("Project {} is not loaded", &project_id);
                            }
                        }
                    });
                }
            }
            PiEvent::EngineExit(project_id) => {
                channels_per_project.remove(&project_id);
            }
            PiEvent::FetchResponse(project_id, node_id, url, contents) => {
                // Pass on the response to the engine's channel
                match channels_per_project.get(&project_id) {
                    Some(channel) => match channel.tx.send(PiEvent::FetchResponse(
                        project_id.clone(),
                        node_id.clone(),
                        url.clone(),
                        contents.clone(),
                    )) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error sending PiEvent in Engine: {}", err);
                        }
                    },
                    None => {
                        error!("Project {} is not loaded", &project_id);
                    }
                }
            }
            PiEvent::Shutdown => {
                break;
            }
            _ => {}
        }
    }
}
