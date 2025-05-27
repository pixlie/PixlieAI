use super::{Settings, SettingsStatus};
use crate::{api::ApiState, PiEvent};
use actix_web::{error::ErrorInternalServerError, get, post, put, web, Responder, Result};
use log::{debug, error};
use reqwest::Client;
use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};
use url::Url;

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

#[utoipa::path(
    path = "/settings/gliner",
    responses(
        (status = 200, description = "Download started"),
        (status = 500, description = "Download failed"),
    ),
    tag = "settings",
)]
#[post("/gliner")]
pub async fn gliner_settings() -> Result<impl Responder> {
    let settings: Settings = Settings::get_cli_settings()?;
    let storage_dir = match settings.path_to_storage_dir {
        Some(path) => PathBuf::from(path),
        None => {
            return Err(ErrorInternalServerError(
                "Cannot find path to storage directory",
            ));
        }
    };
    let gliner_dir = storage_dir.join("gliner_onnx_models/multitask_large_v0_5");
    if !gliner_dir.exists() {
        create_dir_all(&gliner_dir).map_err(|e| {
            error!("Failed to create Gliner directory: {:?}", e);
            ErrorInternalServerError("Could not create directory")
        })?;
    }
    let client = Client::new();
    let download_links = vec![
        "https://huggingface.co/onnx-community/gliner-multitask-large-v0.5/resolve/main/onnx/model.onnx",
        "https://huggingface.co/onnx-community/gliner-multitask-large-v0.5/resolve/main/tokenizer.json",
    ];
    for download_link in download_links {
        let url = Url::parse(download_link).map_err(|e| {
            error!("Failed to parse URL {}: {:?}", download_link, e);
            ErrorInternalServerError("Invalid URL")
        })?;
        let filename = url
            .path_segments()
            .and_then(|segments| segments.last())
            .ok_or_else(|| {
                error!("Failed to extract filename from URL {}", download_link);
                ErrorInternalServerError("Failed to extract filename from URL")
            })?;
        let path = gliner_dir.join(filename);
        let res = client.get(download_link).send().await.map_err(|e| {
            error!("Failed to send request for {}: {:?}", filename, e);
            ErrorInternalServerError("Failed to send request")
        })?;
        if !res.status().is_success() {
            error!("Failed to download {}: HTTP {}", filename, res.status());
            return Err(ErrorInternalServerError("Failed to download file"));
        }
        let bytes = res.bytes().await.map_err(|e| {
            error!("Failed to read response for {}: {:?}", filename, e);
            ErrorInternalServerError("Failed to read response")
        })?;
        write(&path, &bytes).map_err(|e| {
            error!("Failed to write file {}: {:?}", filename, e);
            ErrorInternalServerError("Failed to save file")
        })?;
    }
    Ok(web::Json(gliner_dir))
}

pub fn configure_api_pixlie_settings(
    app_config: &mut utoipa_actix_web::service_config::ServiceConfig,
) {
    app_config.service(
        utoipa_actix_web::scope::scope("/settings")
            .service(read_settings)
            .service(update_settings)
            .service(check_settings_status)
            .service(gliner_settings),
    );
}

// pub async fn request_setup_gliner(api_state: web::Data<ApiState>) -> Result<impl Responder> {
//     api_state.main_tx.send(PiEvent::SetupGliner).unwrap();
//     Ok("OK")
// }
