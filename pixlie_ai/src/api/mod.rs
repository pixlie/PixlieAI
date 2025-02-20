use crate::{config, engine, error::PiResult, projects, PiEvent};
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::http::header::HeaderName;
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    http, rt, web, App, HttpServer, Responder,
};
use crossbeam_utils::atomic::AtomicCell;
use log::info;
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
    let static_admin_dir = config::get_static_admin_dir()?;
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

            let static_admin = Files::new("/", static_admin_dir.clone())
                .index_file("index.html")
                .default_handler(fn_service(|req: ServiceRequest| async {
                    let mut static_admin_default =
                        PathBuf::from(config::get_static_admin_dir()?.clone());
                    static_admin_default.push("index.html");
                    let (req, _) = req.into_parts();
                    let file = NamedFile::open_async(static_admin_default.clone()).await?;
                    let res = file.into_response(&req);
                    Ok(ServiceResponse::new(req, res))
                }));

            App::new()
                .wrap(cors)
                .app_data(api_state.clone())
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
                    web::resource(format!(
                        "{}/engine/{{project_id}}/query/{{node_id}}",
                        API_ROOT
                    ))
                    .route(web::get().to(engine::api::query)),
                )
                .service(
                    web::resource(format!("{}/projects", API_ROOT))
                        .route(web::get().to(projects::api::read_projects))
                        .route(web::post().to(projects::api::create_project)),
                )
                // This is the admin UI and should be the last service
                .service(static_admin)
        })
        .bind(("localhost", 58236))?
        .workers(1)
        .run(),
    )?;
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
