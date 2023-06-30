use std::{env, path::PathBuf};

use directories::{ProjectDirs, UserDirs};
use tracing::error;

use crate::Error;

/// Return the path to the user's home directory
pub fn home_dir_path() -> Result<PathBuf, Error> {
    if let Some(user_dirs) = UserDirs::new() {
        Ok(user_dirs.home_dir().to_path_buf())
    } else {
        Err(Error::NovelApi(
            "No valid home directory path could be retrieved from the operating system".to_string(),
        ))
    }
}

/// Return the path to the project's config directory or the current directory on failure
pub fn config_dir_path(app_name: &str) -> Result<PathBuf, Error> {
    match ProjectDirs::from("", "novel-rs", app_name) {
        Some(dir) => Ok(dir.config_dir().to_path_buf()),
        None => {
            error!("Failed to get the path to the project's config directory, using the current working directory");
            Ok(env::current_dir()?)
        }
    }
}

/// Return the path to the project's local data directory or the current directory on failure
pub fn data_dir_path(app_name: &str) -> Result<PathBuf, Error> {
    match ProjectDirs::from("", "novel-rs", app_name) {
        Some(dir) => Ok(dir.data_local_dir().to_path_buf()),
        None => {
            error!("Failed to get the path to the project's local data directory, using the current working directory");
            Ok(env::current_dir()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_path() -> Result<(), Error> {
        let _ = super::config_dir_path("test-app")?;
        Ok(())
    }

    #[test]
    fn data_dir_path() -> Result<(), Error> {
        let _ = super::data_dir_path("test-app")?;
        Ok(())
    }
}
