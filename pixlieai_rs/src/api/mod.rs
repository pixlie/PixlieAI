use crate::{config::Settings, error::PiResult, PiCliEvent};
use actix_web::{error::ErrorInternalServerError, rt, web, App, HttpServer, Responder, Result};
use log::{error, info};
use std::sync::mpsc;

const API_ROOT: &str = "/api";

#[derive(Clone)]
struct ApiState {
    cli_tx: mpsc::Sender<PiCliEvent>,
}

async fn hello() -> impl Responder {
    format!("Hello, world!")
}

async fn read_settings() -> impl Responder {
    let settings = Settings::get_cli_settings().unwrap();
    web::Json(settings)
}

async fn update_settings(
    updates: web::Json<Settings>,
    api_state: web::Data<ApiState>,
) -> Result<web::Json<Settings>> {
    // updates contains partial settings, we merge it with the existing settings
    match Settings::get_cli_settings() {
        Ok(mut settings) => {
            settings.merge_updates(&updates);
            match settings.write_to_config_file() {
                Ok(_) => {
                    api_state.cli_tx.send(PiCliEvent::SettingsUpdated).unwrap();
                    Ok(web::Json(settings))
                }
                Err(err) => {
                    error!("Error writing settings: {}", err);
                    Err(ErrorInternalServerError::<_>(err))
                }
            }
        }
        Err(err) => {
            error!("Error reading settings: {}", err);
            Err(ErrorInternalServerError::<_>(err))
        }
    }
}

pub fn api_manager(tx: mpsc::Sender<PiCliEvent>) -> PiResult<()> {
    info!("Starting Pixlie AI API");
    let api_state = web::Data::new(ApiState { cli_tx: tx });
    rt::System::new().block_on(
        HttpServer::new(move || {
            App::new()
                .app_data(api_state.clone())
                .service(web::resource(API_ROOT).route(web::get().to(hello)))
                .service(
                    web::resource(format!("{}/settings", API_ROOT))
                        .route(web::get().to(read_settings))
                        .route(web::put().to(update_settings)),
                )
        })
        .bind(("localhost", 58236))?
        .workers(1)
        .run(),
    )?;
    Ok(())
}
