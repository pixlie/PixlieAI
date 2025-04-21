use super::Settings;
use crate::{api::ApiState, PiEvent};
use actix_web::{error::ErrorInternalServerError, get, put, web, Responder, Result, Scope};
use log::{debug, error};

#[get("")]
pub async fn read_settings() -> Result<impl Responder> {
    let settings = Settings::get_cli_settings()?;
    Ok(web::Json(settings))
}

#[get("/status")]
pub async fn check_settings_status() -> Result<impl Responder> {
    let settings = Settings::get_cli_settings()?;
    Ok(web::Json(settings.get_settings_status()?))
}

#[put("")]
pub async fn update_settings(
    updates: web::Json<Settings>,
    api_state: web::Data<ApiState>,
) -> Result<impl Responder> {
    // updates contains partial settings, we merge it with the existing settings
    match Settings::get_cli_settings() {
        Ok(mut settings) => {
            settings.merge_updates(&updates);
            debug!("Settings updated: {:?}", settings);
            match settings.write_to_config_file() {
                Ok(_) => {
                    api_state.main_tx.send(PiEvent::SettingsUpdated).unwrap();
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

pub fn api_settings_scope() -> Scope {
    web::scope("/settings")
        .service(read_settings)
        .service(update_settings)
        .service(check_settings_status)
}

// pub async fn request_setup_gliner(api_state: web::Data<ApiState>) -> Result<impl Responder> {
//     api_state.main_tx.send(PiEvent::SetupGliner).unwrap();
//     Ok("OK")
// }
