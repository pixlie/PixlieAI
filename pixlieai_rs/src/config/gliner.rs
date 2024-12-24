use super::Settings;
use crate::{error::PiResult, PiEvent};
use log::error;
use std::{path::PathBuf, process::Command, sync::mpsc};

fn get_path_to_gliner() -> PiResult<PathBuf> {
    let settings = Settings::get_cli_settings()?;
    let mut path_to_venv = PathBuf::from(settings.path_to_storage_dir.unwrap());
    path_to_venv.push("gliner");
    Ok(path_to_venv)
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

fn test_long_running_job() -> PiResult<bool> {
    let mut path_to_gliner = get_path_to_gliner()?;
    path_to_gliner.push("arch_test.iso");
    match Command::new("curl")
        .arg("-o")
        .arg(path_to_gliner)
        .arg("https://mirrors.abhy.me/archlinux/iso/2024.12.01/archlinux-2024.12.01-x86_64.iso")
        .output()
    {
        Ok(_) => Ok(true),
        Err(err) => {
            error!("Error opening command: {}", err);
            Err(err.into())
        }
    }
}

pub fn setup_gliner(tx: mpsc::Sender<PiEvent>) -> PiResult<()> {
    create_venv_for_gliner()?;
    test_long_running_job()?;
    tx.send(PiEvent::FinishedSetupGliner).unwrap();
    Ok(())
}
