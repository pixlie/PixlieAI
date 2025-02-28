use crate::config::{Settings, WithHostname};
use crate::error::PiError;
use crate::{config, engine, error::PiResult, projects, workspace, PiEvent};
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::http::header::HeaderName;
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    http,
    middleware::Logger,
    rt, web, App, HttpServer, Responder,
};
use crossbeam_utils::atomic::AtomicCell;
use log::{debug, error, info};
use rustls::pki_types::PrivateKeyDer;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

const API_ROOT: &str = "/api";

pub struct ApiState {
    // Send the incoming API requests to the main thread
    pub main_tx: crossbeam_channel::Sender<PiEvent>,
    // Receive the API responses from the main thread
    // Responses are broadcast to all API handlers
    pub api_channel_tx: tokio::sync::broadcast::Sender<PiEvent>,
    // Set a unique request ID for each API request
    pub req_id: AtomicCell<u32>,
}

async fn hello() -> impl Responder {
    "Hello, world! I am the API of Pixlie AI."
}

fn load_rustls_config(with_hostname: &WithHostname) -> PiResult<ServerConfig> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    // init server config builder with safe defaults
    let config = ServerConfig::builder().with_no_client_auth();

    // load TLS key/cert files
    let cert_file = match File::open(&with_hostname.path_to_certificate) {
        Ok(cert_file) => &mut BufReader::new(cert_file),
        Err(err) => {
            error!(
                "Error opening certificate file at {}: {}",
                with_hostname.path_to_certificate.display(),
                err
            );
            return Err(PiError::InternalError(
                "Error opening certificate file".to_string(),
            ));
        }
    };
    let key_file = match File::open(&with_hostname.path_to_key) {
        Ok(key_file) => &mut BufReader::new(key_file),
        Err(err) => {
            error!(
                "Error opening key file at {}: {}",
                with_hostname.path_to_key.display(),
                err
            );
            return Err(PiError::InternalError("Error opening key file".to_string()));
        }
    };

    // convert files to key/cert objects
    let cert_chain = certs(cert_file).collect::<Result<Vec<_>, _>>().unwrap();
    let mut keys = pkcs8_private_keys(key_file)
        .map(|key| key.map(PrivateKeyDer::Pkcs8))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    match config.with_single_cert(cert_chain, keys.remove(0)) {
        Ok(config) => Ok(config),
        Err(err) => {
            error!("Error configuring TLS: {}", err);
            Err(PiError::InternalError("Error configuring TLS".to_string()))
        }
    }
}

fn configure_app(app_config: &mut web::ServiceConfig) {
    let static_admin_dir = match config::get_static_admin_dir() {
        Ok(static_admin_dir) => static_admin_dir,
        Err(err) => {
            error!("Error getting static admin directory: {}", err);
            return;
        }
    };
    let static_admin = Files::new("/", static_admin_dir.clone())
        .index_file("index.html")
        .default_handler(fn_service(|req: ServiceRequest| async {
            let mut static_admin_default = PathBuf::from(config::get_static_admin_dir()?.clone());
            static_admin_default.push("index.html");
            let (req, _) = req.into_parts();
            let file = NamedFile::open_async(static_admin_default.clone()).await?;
            let res = file.into_response(&req);
            Ok(ServiceResponse::new(req, res))
        }));

    app_config
        .service(web::resource(API_ROOT).route(web::get().to(hello)))
        .service(
            web::resource(format!("{}/settings", API_ROOT))
                .route(web::get().to(config::api::read_settings))
                .route(web::put().to(config::api::update_settings)),
        )
        .service(
            web::resource(format!("{}/settings/status", API_ROOT))
                .route(web::get().to(config::api::check_settings_status)),
        )
        .service(
            web::resource(format!("{}/settings/setup_gliner", API_ROOT))
                .route(web::post().to(config::api::request_setup_gliner)),
        )
        .service(
            web::resource(format!("{}/engine/{{project_id}}/labels", API_ROOT))
                .route(web::get().to(engine::api::get_labels)),
        )
        .service(
            web::resource(format!("{}/engine/{{project_id}}/nodes", API_ROOT))
                .route(web::get().to(engine::api::get_nodes))
                .route(web::post().to(engine::api::create_node)),
        )
        .service(
            web::resource(format!("{}/engine/{{project_id}}/edges", API_ROOT))
                .route(web::get().to(engine::api::get_edges)),
        )
        .service(
            web::resource(format!(
                "{}/engine/{{project_id}}/query/{{node_id}}",
                API_ROOT
            ))
            .route(web::get().to(engine::api::search_results)),
        )
        .service(
            web::resource(format!("{}/projects", API_ROOT))
                .route(web::get().to(projects::api::read_projects))
                .route(web::post().to(projects::api::create_project)),
        )
        .service(
            web::resource(format!("{}/workspace", API_ROOT))
                .route(web::get().to(workspace::api::read_default_workspace)),
        )
        .service(
            web::resource(format!("{}/workspace/{{workspace_id}}", API_ROOT))
                .route(web::put().to(workspace::api::update_workspace)),
        )
        // This is the admin UI and should be the last service
        .service(static_admin);
}

pub fn api_manager(
    main_tx: crossbeam_channel::Sender<PiEvent>,
    api_channel_tx: tokio::sync::broadcast::Sender<PiEvent>,
) -> PiResult<()> {
    info!("Starting Pixlie AI API");
    let api_state = web::Data::new(ApiState {
        main_tx,
        api_channel_tx,
        req_id: AtomicCell::new(0),
    });
    let (_path_to_config_dir, path_to_config_file) = config::get_cli_settings_path()?;
    info!("CLI settings path {}", path_to_config_file.display());
    let settings = Settings::get_cli_settings()?;
    debug!("CLI settings {:?}", settings);
    let with_hostname = settings.get_hostname()?;
    debug!("CLI settings path {}", path_to_config_file.display());
    match with_hostname {
        Some(with_hostname) => {
            debug!(
                "Starting API server on {}:{}",
                &with_hostname.hostname, 58236
            );
            match load_rustls_config(&with_hostname) {
                Ok(host_config) => {
                    rt::System::new().block_on(
                        HttpServer::new(move || {
                            // Allow for localhost, ports 5173 (development)
                            let cors = Cors::default()
                                .allowed_origin("http://localhost:5173")
                                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                                .allowed_headers(vec![
                                    http::header::ACCEPT,
                                    http::header::CONTENT_TYPE,
                                    // https://docs.sentry.io/platforms/javascript/tracing/trace-propagation/dealing-with-cors-issues/
                                    HeaderName::from_bytes(b"Baggage").unwrap(),
                                    HeaderName::from_bytes(b"Sentry-Trace").unwrap(),
                                ]);

                            App::new()
                                .wrap(cors)
                                .wrap(Logger::new("%r: %s %b %T"))
                                .app_data(api_state.clone())
                                .configure(configure_app)
                        })
                        .bind_rustls_0_23((with_hostname.hostname.clone(), 58236), host_config)?
                        .workers(1)
                        .run(),
                    )?;
                }
                Err(err) => {
                    error!("Error loading TLS config: {}", err);
                    return Err(PiError::InternalError(
                        "Error loading TLS config".to_string(),
                    ));
                }
            }
        }
        None => {
            debug!("Starting API server on localhost:{}", 58236);
            rt::System::new().block_on(
                HttpServer::new(move || {
                    // Allow for localhost, ports 5173 (development)
                    let cors = Cors::default()
                        .allowed_origin("http://localhost:5173")
                        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                        .allowed_headers(vec![
                            http::header::ACCEPT,
                            http::header::CONTENT_TYPE,
                            // https://docs.sentry.io/platforms/javascript/tracing/trace-propagation/dealing-with-cors-issues/
                            HeaderName::from_bytes(b"Baggage").unwrap(),
                            HeaderName::from_bytes(b"Sentry-Trace").unwrap(),
                        ]);

                    App::new()
                        .wrap(cors)
                        .wrap(Logger::new("%r: %s %b %T"))
                        .app_data(api_state.clone())
                        .configure(configure_app)
                })
                .bind(("localhost", 58236))?
                .workers(1)
                .run(),
            )?;
        }
    }
    Ok(())
}

// The receiver of the API channel is in async code, so we use an async channel for that
// https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code
// We use a broadcast channel because we want to send the API request to all API handlers
// The individual API handler checks if the response is for them
pub struct APIChannel {
    pub tx: tokio::sync::broadcast::Sender<PiEvent>,
    pub rx: tokio::sync::broadcast::Receiver<PiEvent>,
}

impl APIChannel {
    pub fn new() -> APIChannel {
        let (tx, rx) = tokio::sync::broadcast::channel::<PiEvent>(100);
        APIChannel { tx, rx }
    }
}
