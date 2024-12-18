use super::Settings;
use crate::error::PiResult;
use log::error;
use std::{path::PathBuf, process::Command};

pub struct Python {
    pub version: (u8, u8, u8),
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

fn parse_version(version: &str) -> (u8, u8, u8) {
    // Version is in the format "Python 3.9.0"
    let version = version.split(' ').collect::<Vec<&str>>();
    // Split the second part by '.', extract the parts as integers
    let version = version[1].split('.').collect::<Vec<&str>>();
    let major = version[0].parse::<u8>().unwrap();
    let minor = version[1].parse::<u8>().unwrap();
    let patch = if version.len() == 3 {
        version[2].parse::<u8>().unwrap()
    } else {
        0
    };
    (major, minor, patch)
}

pub fn check_system_python() -> Option<Python> {
    // We run the std::process::Command to check if python3 is installed
    let output_result = Command::new("python3").arg("--version").output();

    match output_result {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .to_string()
                    .trim()
                    .to_string();
                // We need Python version >= 3.9, so we check this here
                // Parse the version string and check if it's >= 3.9
                let version = parse_version(&version);
                if version.0 >= 3 && version.1 >= 9 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let version = parse_version("Python 3.9.0");
        assert_eq!(version, (3, 9, 0));

        let version = parse_version("Python 3.12.1");
        assert_eq!(version, (3, 12, 1));

        let version = parse_version("Python 3.10.0");
        assert_eq!(version, (3, 10, 0));
    }
}
