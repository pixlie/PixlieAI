use super::{Settings, SettingsStatus};
use crate::{api::ApiState, PiEvent};
use actix_web::{error::ErrorInternalServerError, get, put, web, Responder, Result};
use log::{debug, error};

/// Read Pixlie AI settings
#[utoipa::path(
    path="/settings",
    responses(
        (status = 200, description = "Settings retrieved successfully", body = Settings),
        (status = 500, description = "Internal server error"),
    ),
    tag = "settings",
)]
#[get("")]
pub async fn read_settings() -> Result<impl Responder> {
    let settings = Settings::get_cli_settings()?;
    Ok(web::Json(settings))
}

/// Check the status of Pixlie AI settings
#[utoipa::path(
    path="/settings/status",
    responses(
        (status = 200, description = "Settings status retrieved successfully", body = SettingsStatus),
        (status = 500, description = "Internal server error"),
    ),
    tag = "settings",
)]
#[get("/status")]
pub async fn check_settings_status() -> Result<impl Responder> {
    let settings = Settings::get_cli_settings()?;
    Ok(web::Json(settings.get_settings_status()?))
}

/// Update Pixlie AI settings
#[utoipa::path(
    path="/settings",
    request_body = Settings,
    responses(
        (status = 200, description = "Settings updated successfully", body = Settings),
        (status = 500, description = "Internal server error"),
    ),
    tag = "settings",
)]
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

pub fn configure_api_pixlie_settings(
    app_config: &mut utoipa_actix_web::service_config::ServiceConfig,
) {
    app_config.service(
        utoipa_actix_web::scope::scope("/settings")
            .service(read_settings)
            .service(update_settings)
            .service(check_settings_status),
    );
}

// pub async fn request_setup_gliner(api_state: web::Data<ApiState>) -> Result<impl Responder> {
//     api_state.main_tx.send(PiEvent::SetupGliner).unwrap();
//     Ok("OK")
// }
