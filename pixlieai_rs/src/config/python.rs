use log::error;
use std::process::Command;

pub struct Python {
    pub version: f32,
    pub venv: bool,
    pub pip: bool,
}

pub fn check_venv() -> bool {
    let output_result = Command::new("python3")
        .arg("-m")
        .arg("venv")
        .arg("--help")
        .output();

    match output_result {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                false
            }
        }
        Err(err) => {
            error!("Error: {}", err);
            false
        }
    }
}

pub fn check_pip() -> bool {
    let output_result = Command::new("python3")
        .arg("-m")
        .arg("pip")
        .arg("--help")
        .output();

    match output_result {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                false
            }
        }
        Err(err) => {
            error!("Error: {}", err);
            false
        }
    }
}

pub fn check_system_python() -> Option<Python> {
    // We run the std::process::Command to check if python3 is installed
    let output_result = Command::new("python3").arg("--version").output();

    match output_result {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).to_string();
                // We need Python version >= 3.9, so we check this here
                // Parse the version string and check if it's >= 3.9
                let parsed = version.split(' ').collect::<Vec<&str>>();
                // Parse the second part as float
                let version = parsed[1].parse::<f32>().unwrap();
                if version >= 3.9 {
                    Some(Python {
                        version,
                        venv: check_venv(),
                        pip: check_pip(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(err) => {
            error!("Error: {}", err);
            None
        }
    }
}
