use std::{fs, path::PathBuf};

use serde::{de::DeserializeOwned, Serialize};
use tracing::info;

use crate::Error;

const CONFIG_FILE_NAME: &str = "config.toml";

const CONFIG_FILE_PASSWORD: &str = "nupwuz-toxvif-0timNo";
const CONFIG_FILE_AAD: &str = "novel-rs-config";

pub(crate) fn load_config_file<T, R>(app_name: T) -> Result<Option<R>, Error>
where
    T: AsRef<str>,
    R: DeserializeOwned,
{
    let config_file_path = config_file_path(app_name)?;

    if config_file_path.try_exists()? {
        info!(
            "The config file is located at: `{}`",
            config_file_path.display()
        );

        let config = crate::aes_256_gcm_base64_decrypt(
            config_file_path,
            CONFIG_FILE_PASSWORD,
            CONFIG_FILE_AAD,
        )?;
        let config: R = toml::from_str(&config)?;

        Ok(Some(config))
    } else {
        fs::create_dir_all(config_file_path.parent().unwrap())?;

        info!(
            "The config file will be created at: `{}`",
            config_file_path.display()
        );

        Ok(None)
    }
}

pub(crate) fn save_config_file<T, E>(app_name: T, config: E) -> Result<(), Error>
where
    T: AsRef<str>,
    E: Serialize,
{
    let config_file_path = config_file_path(app_name)?;

    info!("Save the config file at: `{}`", config_file_path.display());

    crate::aes_256_gcm_base64_encrypt(
        toml::to_string(&config)?,
        config_file_path,
        CONFIG_FILE_PASSWORD,
        CONFIG_FILE_AAD,
    )?;

    Ok(())
}

fn config_file_path<T>(app_name: T) -> Result<PathBuf, Error>
where
    T: AsRef<str>,
{
    let mut config_file_path = crate::config_dir_path(app_name.as_ref())?;
    config_file_path.push(CONFIG_FILE_NAME);

    Ok(config_file_path)
}
