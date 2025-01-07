use crate::{config::get_static_admin_dir, error::PiResult, PiEvent};
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    http, rt, web, App, HttpServer, Responder,
};
use log::info;
use settings::{
    check_mqtt_broker, check_settings_status, read_settings, request_setup_gliner, update_settings,
};
use std::{path::PathBuf, sync::mpsc};

pub mod settings;

const API_ROOT: &str = "/api";

#[derive(Clone)]
pub struct ApiState {
    pub cli_tx: mpsc::Sender<PiEvent>,
}

async fn hello() -> impl Responder {
    format!("Hello, world! I am the API of Pixlie AI.")
}

pub fn api_manager(tx: mpsc::Sender<PiEvent>) -> PiResult<()> {
    info!("Starting Pixlie AI API");
    let api_state = web::Data::new(ApiState { cli_tx: tx });
    let static_admin_dir = get_static_admin_dir()?;
    rt::System::new().block_on(
        HttpServer::new(move || {
            // Allow for localhost, ports 5173 (development)
            let cors = Cors::default()
                .allowed_origin("http://localhost:5173")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![http::header::ACCEPT, http::header::CONTENT_TYPE]);

            let static_admin = Files::new("/", static_admin_dir.clone())
                .index_file("index.html")
                .default_handler(fn_service(|req: ServiceRequest| async {
                    let mut static_admin_default = PathBuf::from(get_static_admin_dir()?.clone());
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
                        .route(web::get().to(read_settings))
                        .route(web::put().to(update_settings)),
                )
                .service(
                    web::resource(format!("{}/settings/status", API_ROOT))
                        .route(web::get().to(check_settings_status)),
                )
                .service(
                    web::resource(format!("{}/settings/check_mqtt_broker", API_ROOT))
                        .route(web::get().to(check_mqtt_broker)),
                )
                .service(
                    web::resource(format!("{}/settings/setup_gliner", API_ROOT))
                        .route(web::post().to(request_setup_gliner)),
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
