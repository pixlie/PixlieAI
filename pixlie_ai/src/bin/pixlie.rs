use log::{debug, error, info};
use pixlie_ai::api::{send_api_error, APIChannel};
use pixlie_ai::engine::Engine;
use pixlie_ai::error::{PiError, PiResult};
use pixlie_ai::projects::check_project_db;
use pixlie_ai::utils::fetcher::fetcher_runtime;
use pixlie_ai::{api::api_manager, config::check_cli_settings, FetchResponse, PiChannel, PiEvent};
use std::collections::HashMap;
use std::env::var;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

fn load_project_to_engine(
    project_uuid: &str,
    path_to_storage_dir: &PathBuf,
    channels_per_project: Arc<Mutex<HashMap<String, PiChannel>>>,
    pi_channel_tx: crossbeam_channel::Sender<PiEvent>,
    fetcher_tx: tokio::sync::mpsc::Sender<PiEvent>,
    pool: Arc<ThreadPool>,
) -> PiResult<()> {
    match channels_per_project.try_lock() {
        Ok(mut channels_per_project) => {
            channels_per_project.insert(project_uuid.to_string(), PiChannel::new());
            let my_pi_channel = match channels_per_project.get(project_uuid) {
                Some(my_pi_channel) => my_pi_channel.clone(),
                None => {
                    error!(
                        "Cannot find per engine channel for project {}",
                        project_uuid
                    );
                    return Err(PiError::InternalError(format!(
                        "Cannot find per engine channel for project {}",
                        project_uuid
                    )));
                }
            };

            let engine = match Engine::open(
                project_uuid,
                &path_to_storage_dir,
                my_pi_channel.clone(),
                pi_channel_tx,
                fetcher_tx,
            ) {
                Ok(engine) => engine,
                Err(err) => {
                    channels_per_project.remove(project_uuid);
                    return Err(err);
                }
            };
            let arced_engine = Arc::new(engine);
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

    Ok(())
}

fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{style}{}{style:#} {}:{}] {}",
                record.level(),
                record.module_path().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

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
                debug!(
                    "Sentry initialized for this CLI application, you can see errors at \
                https://pixlie.sentry.io/issues/?project=4508832865648720"
                );
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

    #[cfg(not(debug_assertions))]
    {
        use pixlie_ai::config::download_admin_site;
        match download_admin_site() {
            Ok(_) => {}
            Err(err) => {
                error!("Error downloading admin site: {}", err);
                return;
            }
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

    let main_channel_iter = main_channel.clone();
    let channels_per_project = channels_per_project.clone();
    for event in main_channel_iter.rx.iter() {
        match event {
            PiEvent::APIRequest {
                project_id,
                request_id,
                payload,
            } => {
                let path_to_storage_dir = match check_project_db(&project_id) {
                    Ok(path_to_storage_dir) => path_to_storage_dir,
                    Err(err) => {
                        send_api_error(
                            api_channel.tx.clone(),
                            &project_id,
                            request_id,
                            &err.to_string(),
                        );
                        continue;
                    }
                };

                let channels_per_project_inner = channels_per_project.clone();
                let channel_exists = match channels_per_project.try_lock() {
                    Ok(channels_per_project) => channels_per_project.contains_key(&project_id),
                    Err(err) => {
                        error!("Error locking channels_per_project: {}", err);
                        send_api_error(
                            api_channel.tx.clone(),
                            &project_id,
                            request_id,
                            "Error locking channels_per_project",
                        );
                        continue;
                    }
                };

                if !channel_exists {
                    match load_project_to_engine(
                        &project_id,
                        &path_to_storage_dir,
                        channels_per_project_inner,
                        main_channel_iter.clone().tx,
                        fetcher_tx.clone(),
                        pool.clone(),
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error loading project to engine: {}", err);
                            send_api_error(
                                api_channel.tx.clone(),
                                &project_id,
                                request_id,
                                &format!("{}", err),
                            );
                            continue;
                        }
                    }
                }

                match channels_per_project.try_lock() {
                    Ok(channels_per_project) => {
                        // Engine is loaded, we will pass the API request to the engine's own channel
                        match channels_per_project.get(&project_id) {
                            Some(my_pi_channel) => {
                                match my_pi_channel.tx.send(PiEvent::APIRequest {
                                    project_id: project_id.clone(),
                                    request_id,
                                    payload,
                                }) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("Error sending PiEvent to Engine: {}", err);
                                        send_api_error(
                                            api_channel.tx.clone(),
                                            &project_id,
                                            request_id,
                                            "Error sending PiEvent to Engine",
                                        );
                                    }
                                }
                            }
                            None => {
                                error!("Could not load project {}", project_id);
                                send_api_error(
                                    api_channel.tx.clone(),
                                    &project_id,
                                    request_id,
                                    "Could not load project",
                                );
                            }
                        };
                    }
                    Err(err) => {
                        error!("Error locking channels_per_project: {}", err);
                        send_api_error(
                            api_channel.tx.clone(),
                            &project_id,
                            request_id,
                            "Error locking channels_per_project",
                        );
                    }
                }
            }
            PiEvent::APIResponse {
                project_id,
                request_id,
                payload,
            } => {
                // Pass on the response to the API broadcast channel
                match api_channel.tx.send(PiEvent::APIResponse {
                    project_id,
                    request_id,
                    payload,
                }) {
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

#[cfg(test)]
mod tests {
    use pixlie_ai::{
        utils::version_check::{get_cargo_version, get_version_from_file},
        PIXLIE_VERSION_NUMBER,
    };

    #[test]
    fn test_match_version_file_with_cargo_version() {
        assert_eq!(
            get_version_from_file().unwrap(),
            get_cargo_version().unwrap()
        );
    }

    #[test]
    fn test_match_cargo_version_with_code_version() {
        assert_eq!(get_cargo_version().unwrap(), PIXLIE_VERSION_NUMBER);
    }
}
