use super::Settings;
use crate::error::PiResult;
use log::error;
use std::{path::PathBuf, process::Command};

pub fn get_path_to_gliver() -> PiResult<PathBuf> {
    let settings = Settings::get_cli_settings()?;
    let mut path_to_venv = PathBuf::from(settings.path_to_storage_dir.unwrap());
    path_to_venv.push("gliner");
    Ok(path_to_venv)
}

pub fn create_venv_for_gliner() -> PiResult<bool> {
    let mut path_to_venv = get_path_to_gliver()?;
    path_to_venv.push(".venv");
    match Command::new("python3")
        .arg("-m")
        .arg("venv")
        .arg(path_to_venv.to_str().unwrap())
        .status()
    {
        Ok(status) => Ok(status.success()),
        Err(err) => {
            error!("Error: {}", err);
            Err(err.into())
        }
    }
}

pub fn setup_gliner() -> PiResult<bool> {
    let path_to_gliner = get_path_to_gliver()?;
    let mut path_to_python = path_to_gliner.clone();
    path_to_python.push(".venv");
    path_to_python.push("bin");
    path_to_python.push("python");
    match Command::new(path_to_python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("-r")
        .arg("requirements.txt")
        .current_dir(path_to_gliner.to_str().unwrap())
        .status()
    {
        Ok(status) => Ok(status.success()),
        Err(err) => {
            error!("Error: {}", err);
            Err(err.into())
        }
    }
}
