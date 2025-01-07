use super::Settings;
use crate::{
    error::{PiError, PiResult},
    PiEvent,
};
use bytes::Buf;
use flate2::read::GzDecoder;
use log::{debug, error, info};
use std::{
    fs::{create_dir, exists, remove_dir_all, remove_file, File},
    path::PathBuf,
    process::Command,
    sync::mpsc,
};
use tar::Archive;

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

fn download_gliner_code() -> PiResult<()> {
    // We download gliner.tar.gz from our GitHub release
    let gliner_tar_gz_url =
        "https://github.com/pixlie/PixlieAI/releases/download/v0.1.0/gliner.tar.gz";
    let gliner_path = get_path_to_gliner()?;
    let gliner_tar_gz_response = reqwest::blocking::get(gliner_tar_gz_url)?;
    let gliner_tar_gz_bytes = gliner_tar_gz_response.bytes()?;
    // Use flate2 to decompress the tar.gz file
    let gliner_tar_gz = GzDecoder::new(gliner_tar_gz_bytes.reader());
    // Use tar to extract the files from the tar.gz file
    Archive::new(gliner_tar_gz).unpack(&gliner_path)?;
    Ok(())
}

fn install_gliner_dependencies() -> PiResult<bool> {
    let path_to_gliner = get_path_to_gliner()?;
    let mut path_to_python = path_to_gliner.clone();
    path_to_python.push(".venv");
    path_to_python.push("bin");
    path_to_python.push("python");
    let mut path_to_requirements = path_to_gliner.clone();
    path_to_requirements.push("requirements.txt");
    match Command::new(path_to_python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("-r")
        .arg(path_to_requirements.to_str().unwrap())
        .current_dir(path_to_gliner.to_str().unwrap())
        .status()
    {
        Ok(status) => Ok(status.success()),
        Err(err) => {
            error!(
                "Could not install Gliner dependencies at {}\nError: {}",
                path_to_gliner.display(),
                err
            );
            Err(err.into())
        }
    }
}

pub fn get_is_gliner_setup() -> PiResult<bool> {
    // GLiNER is supported locally only
    // We check if the virtual environment for GLiNER has been created
    // The virtual environment is created in a gliner/.venv directory in the cli settings directory
    let path_to_gliner = get_path_to_gliner()?;
    let mut path_to_gliner_venv = path_to_gliner.clone();
    path_to_gliner_venv.push(".venv");
    let mut path_to_gliner_python = path_to_gliner_venv.clone();
    path_to_gliner_python.push("bin");
    path_to_gliner_python.push("python");
    // Check if the virtual environment has been created
    match exists(&path_to_gliner_venv) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(false);
        }
        Err(err) => {
            error!(
                "Error checking if Gliner virtual environment exists: {}",
                err
            );
            return Err(err.into());
        }
    }
    // Check if Gliner has been installed
    match Command::new(path_to_gliner_python)
        .arg("-m")
        .arg("pip")
        .arg("freeze")
        .current_dir(path_to_gliner.to_str().unwrap())
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                debug!(
                    "Gliner pip freeze output: {}",
                    String::from_utf8_lossy(&output.stdout)
                );
                // Check if gliner is installed
                let output = String::from_utf8_lossy(&output.stdout).to_string();
                if output.contains("gliner") {
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        Err(err) => {
            error!("Error: {}", err);
            Err(err.into())
        }
    }
}

pub fn setup_gliner(tx: mpsc::Sender<PiEvent>) -> PiResult<()> {
    // Touch a lock file when setting up Gliner
    let path_to_gliner = get_path_to_gliner()?;
    let mut path_to_gliner_venv = path_to_gliner.clone();
    path_to_gliner_venv.push(".venv");
    let mut path_to_gliner_setup_lock = path_to_gliner.clone();
    path_to_gliner_setup_lock.push(".gliner_setup");
    match File::create_new(&path_to_gliner_setup_lock) {
        Ok(_) => match exists(&path_to_gliner_venv) {
            Ok(true) => match remove_dir_all(&path_to_gliner_venv) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "Could not delete existing Gliner's virtual environment at {}\nError: {}",
                        path_to_gliner_venv.display(),
                        err
                    );
                    return Err(err.into());
                }
            },
            _ => {}
        },
        Err(err) => {
            error!(
                "Could not create Gliner setup lock file at {}, perhaps Gliner is being setup\nError: {}",
                path_to_gliner_setup_lock.display(),
                err
            );
            return Err(err.into());
        }
    }

    create_venv_for_gliner()?;
    download_gliner_code()?;
    install_gliner_dependencies()?;

    // Remove the lock file
    match remove_file(&path_to_gliner_setup_lock) {
        Ok(_) => {}
        Err(err) => {
            error!(
                "Could not remove Gliner setup lock file at {}\nError: {}",
                path_to_gliner_setup_lock.display(),
                err
            );
            return Err(err.into());
        }
    }
    info!("Gliner setup complete");
    tx.send(PiEvent::FinishedSetupGliner).unwrap();
    Ok(())
}
