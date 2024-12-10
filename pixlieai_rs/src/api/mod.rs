use crate::{error::PiResult, PiCliEvent};
use actix_cors::Cors;
use actix_web::{http, rt, web, App, HttpServer, Responder};
use log::info;
use settings::{check_settings_status, read_settings, update_settings};
use std::sync::mpsc;

pub mod settings;

const API_ROOT: &str = "/api";

#[derive(Clone)]
pub struct ApiState {
    pub cli_tx: mpsc::Sender<PiCliEvent>,
}

async fn hello() -> impl Responder {
    format!("Hello, world!")
}

pub fn api_manager(tx: mpsc::Sender<PiCliEvent>) -> PiResult<()> {
    info!("Starting Pixlie AI API");
    let api_state = web::Data::new(ApiState { cli_tx: tx });
    rt::System::new().block_on(
        HttpServer::new(move || {
            // Allow for localhost, ports 5173 (development) and 58235
            let cors = Cors::default()
                .allowed_origin("http://localhost:5173")
                .allowed_origin("http://localhost:58235")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![http::header::ACCEPT]);

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
        })
        .bind(("localhost", 58236))?
        .workers(1)
        .run(),
    )?;
    Ok(())
}
