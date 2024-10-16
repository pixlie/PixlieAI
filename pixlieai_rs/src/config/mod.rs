use crate::error::{PiError, PiResult};
use config::Config;
use log::error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub anthropic_api_key: String,
    pub path_to_gliner_home: String,
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
