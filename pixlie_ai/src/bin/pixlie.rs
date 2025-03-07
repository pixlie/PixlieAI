use log::{debug, error, info};
use pixlie_ai::api::APIChannel;
use pixlie_ai::config::{download_admin_site, Settings};
use pixlie_ai::engine::Engine;
use pixlie_ai::utils::fetcher::fetcher_runtime;
use pixlie_ai::{api::api_manager, config::check_cli_settings, FetchResponse, PiChannel, PiEvent};
use std::collections::HashMap;
use std::env::var;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

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

    let pool: Arc<ThreadPool> = Arc::new(
        threadpool::Builder::new()
            .thread_name("pixlie_ai_thread".to_string())
            .build(),
    );

    // This channel is used by the CLI (this main function)
    let main_channel = PiChannel::new();
    // Each engine has its own communication channel
    let channels_per_project: Arc<Mutex<HashMap<String, PiChannel>>> =
        Arc::new(Mutex::new(HashMap::new()));
    // The API channel is used by the API server and the CLI
    let api_channel = APIChannel::new();
    let main_channel_tx = main_channel.tx.clone();
    match download_admin_site() {
        Ok(_) => {}
        Err(err) => {
            error!("Error downloading admin site: {}", err);
            return;
        }
    }
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

    fn load_project(
        project_id: String,
        channels_per_project: Arc<Mutex<HashMap<String, PiChannel>>>,
        pi_channel_tx: crossbeam_channel::Sender<PiEvent>,
        fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
        pool: Arc<ThreadPool>,
    ) {
        debug!("Loading project {} into CLI", project_id);

        match channels_per_project.try_lock() {
            Ok(mut channels_per_project) => {
                channels_per_project.insert(project_id.clone(), PiChannel::new());
                let my_pi_channel = match channels_per_project.get(&project_id) {
                    Some(my_pi_channel) => my_pi_channel.clone(),
                    None => {
                        error!("Cannot find per engine channel for project {}", project_id);
                        return;
                    }
                };
                let project_id = project_id.clone();
                let settings: Settings = match Settings::get_cli_settings() {
                    Ok(settings) => settings,
                    Err(err) => {
                        error!("Error reading settings: {}", err);
                        exit(1);
                    }
                };
                let path_to_storage_dir = match settings.path_to_storage_dir {
                    Some(ref path) => path.clone(),
                    None => {
                        error!("Cannot find path to storage directory");
                        return;
                    }
                };

                let arced_engine = Arc::new(Engine::open_project(
                    &path_to_storage_dir,
                    &project_id,
                    my_pi_channel.clone(),
                    pi_channel_tx,
                    fetcher_tx,
                ));
                {
                    let arced_engine = arced_engine.clone();
                    pool.execute(move || {
                        arced_engine.channel_listener();
                    });
                }
                {
                    let arced_engine = arced_engine.clone();
                    pool.execute(move || {
                        arced_engine.ticker();
                    });
                }
            }
            Err(err) => {
                error!("Error locking channels_per_project: {}", err);
            }
        };
    }

    let main_channel_iter = main_channel.clone();
    let channels_per_project = channels_per_project.clone();
    for event in main_channel_iter.rx.iter() {
        match event {
            PiEvent::APIRequest(project_id, request) => {
                let channels_per_project_inner = channels_per_project.clone();
                let channel_exists: bool = match channels_per_project.try_lock() {
                    Ok(channels_per_project) => {
                        match channels_per_project.contains_key(&project_id) {
                            // Project is already loaded, we will send the API request to the engine
                            true => true,
                            // Project is not loaded, we will load it
                            false => false,
                        }
                    }
                    Err(err) => {
                        error!("Error locking channels_per_project: {}", err);
                        false
                    }
                };
                if !channel_exists {
                    load_project(
                        project_id.clone(),
                        channels_per_project_inner,
                        main_channel_iter.clone().tx,
                        fetcher_tx.clone(),
                        pool.clone(),
                    );
                }
                match channels_per_project.try_lock() {
                    Ok(channels_per_project) => {
                        // Engine is loaded, we will pass the API request to the engine's own channel
                        match channels_per_project.get(&project_id) {
                            Some(my_pi_channel) => {
                                match my_pi_channel
                                    .tx
                                    .send(PiEvent::APIRequest(project_id.clone(), request.clone()))
                                {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("Error sending PiEvent in Engine: {}", err);
                                    }
                                }
                            }
                            None => {
                                error!("Project {} is not loaded", project_id);
                            }
                        };
                    }
                    Err(err) => {
                        error!("Error locking channels_per_project: {}", err);
                    }
                }
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
            PiEvent::EngineExit(project_id) => match channels_per_project.try_lock() {
                Ok(mut channels_per_project) => {
                    channels_per_project.remove(&project_id);
                }
                Err(err) => {
                    error!("Error locking channels_per_project: {}", err);
                }
            },
            PiEvent::FetchResponse(response) => {
                match channels_per_project.try_lock() {
                    Ok(channels_per_project) => {
                        // Pass on the response to the engine's channel
                        match channels_per_project.get(&response.project_id) {
                            Some(channel) => {
                                match channel.tx.send(PiEvent::FetchResponse(FetchResponse {
                                    project_id: response.project_id.clone(),
                                    node_id: response.node_id,
                                    url: response.url.clone(),
                                    contents: response.contents.clone(),
                                })) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("Error sending PiEvent in Engine: {}", err);
                                    }
                                }
                            }
                            None => {
                                error!("Project {} is not loaded", &response.project_id);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Error locking channels_per_project: {}", err);
                    }
                }
            }
            PiEvent::FetchError(error) => match channels_per_project.try_lock() {
                Ok(channels_per_project) => match channels_per_project.get(&error.project_id) {
                    Some(channel) => match channel.tx.send(PiEvent::FetchError(error.clone())) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error sending PiEvent in Engine: {}", err);
                        }
                    },
                    None => {
                        error!("Project {} is not loaded", &error.project_id);
                    }
                },
                Err(err) => {
                    error!("Error locking channels_per_project: {}", err);
                }
            },
            PiEvent::Shutdown => {
                break;
            }
            event => {
                info!("Unhandled event: {}", event.to_string());
            }
        }
    }
}
