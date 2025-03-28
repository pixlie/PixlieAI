use log::error;
use std::{fs, path::PathBuf};

use crate::error::{PiError, PiResult};

pub fn check_version() -> PiResult<()> {
    let file_version = get_version_from_file()?;
    let cargo_version = get_cargo_version()?;
    if file_version != cargo_version {
        return Err(PiError::VersionMismatch(cargo_version, file_version));
    }
    Ok(())
}

pub fn get_cargo_version() -> PiResult<String> {
    let mut version_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    version_file.push("Cargo.toml");
    if !version_file.exists() {
        return Err(PiError::VersionCheckError(
            "Cargo.toml does not exist".to_string(),
        ));
    }
    
    let contents = fs::read_to_string(version_file)?;
    let version = match toml::de::from_str::<toml::Value>(&contents) {
        Ok(toml_value) => match toml_value.get("package") {
            Some(package) => match package.get("version") {
                Some(version) => version.to_string(),
                None => return Err(PiError::VersionCheckError("`package.version` not found in Cargo.toml".to_string())),
            },
            None => return Err(PiError::VersionCheckError("`package` not found in Cargo.toml".to_string())),
        },
        Err(err) => {
            return Err(PiError::VersionCheckError("Error reading Cargo.toml".to_string()));
        }
    };
    let version = version.replace("\"", "");
    return Ok(version);
}

pub fn get_version_from_file() -> PiResult<String> {
    let cwd = std::env::current_dir()?;
    let parent = cwd.parent().unwrap();
    let mut version_file = PathBuf::from(parent);
    version_file.push("VERSION");
    if !version_file.exists() {
        return Err(PiError::VersionFileMissing);
    }
    let content = fs::read_to_string(version_file)?;
    Ok((*(content.trim())).to_string())
}