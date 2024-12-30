use super::Settings;
use crate::{
    error::{PiError, PiResult},
    PiEvent,
};
use log::error;
use std::{fs::create_dir, path::PathBuf, process::Command, sync::mpsc};

pub fn get_path_to_gliner() -> PiResult<PathBuf> {
    let settings: Settings = Settings::get_cli_settings()?;
    let mut path_to_gliner = PathBuf::from(&settings.path_to_storage_dir.unwrap());
    path_to_gliner.push("gliner");
    if !path_to_gliner.exists() {
        // Create the `gliner` directory since it does not exist
        match create_dir(path_to_gliner.clone()) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Could not create gliner directory at {}\nError: {}",
                    path_to_gliner.display(),
                    err
                );
                return Err(PiError::CannotReadOrWriteToStorageDirectory);
            }
        }
    }
    Ok(path_to_gliner)
}

fn create_venv_for_gliner() -> PiResult<bool> {
    let mut path_to_venv = get_path_to_gliner()?;
    path_to_venv.push(".venv");
    if path_to_venv.exists() {
        return Ok(true);
    }
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

fn install_gliner_dependencies() -> PiResult<bool> {
    let path_to_gliner = get_path_to_gliner()?;
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

pub fn setup_gliner(tx: mpsc::Sender<PiEvent>) -> PiResult<()> {
    create_venv_for_gliner()?;
    install_gliner_dependencies()?;
    tx.send(PiEvent::FinishedSetupGliner).unwrap();
    Ok(())
}
