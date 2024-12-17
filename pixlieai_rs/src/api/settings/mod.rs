use super::ApiState;
use crate::{config::Settings, PiCliEvent};
use actix_web::{error::ErrorInternalServerError, web, Responder, Result};
use log::error;

pub async fn read_settings() -> Result<impl Responder> {
    let settings = Settings::get_cli_settings()?;
    Ok(web::Json(settings))
}

pub async fn check_settings_status() -> Result<impl Responder> {
    let settings = Settings::get_cli_settings().unwrap();
    Ok(web::Json(settings.get_settings_status()))
}

pub async fn update_settings(
    updates: web::Json<Settings>,
    api_state: web::Data<ApiState>,
) -> Result<impl Responder> {
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