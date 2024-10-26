use crate::{
    error::{PiError, PiResult},
    services::EntityExtractionProvider,
};
use config::Config;
use log::error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub anthropic_api_key: Option<String>,
    pub path_to_gliner_home: Option<String>,
    pub path_to_storage_root: Option<String>,
}

pub fn get_test_settings() -> PiResult<Settings> {
    let settings = Config::builder()
        .add_source(config::File::with_name("src/config/test.toml"))
        .build()?;
    Ok(settings.try_deserialize::<Settings>()?)
}

pub fn get_cli_settings() -> PiResult<Settings> {
    let settings = Config::builder()
        .add_source(config::File::with_name("src/config/cli.toml"))
        .build()?;
    Ok(settings.try_deserialize::<Settings>()?)
}

impl Settings {
    pub fn get_entity_extraction_provider(&self) -> PiResult<EntityExtractionProvider> {
        if let Some(_) = self.path_to_gliner_home {
            return Ok(EntityExtractionProvider::Gliner);
        } else if let Some(_) = self.anthropic_api_key {
            return Ok(EntityExtractionProvider::Anthropic);
        }
        Err(PiError::NotConfiguredProperly)
    }
}
