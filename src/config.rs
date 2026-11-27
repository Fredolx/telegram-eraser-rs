use crate::loc;
use crate::types::Config;
use crate::utils::{create_private_dir, get_project_dirs};
use anyhow::{Context, Result, bail};
use std::path::PathBuf;

const CONFIG_NAME: &str = "config.json";
const API_HASH_LEN: usize = 32;

pub fn load() -> Result<Option<Config>> {
    let path = get_and_create_config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path).context(loc::ERR_READ_CONFIG)?;
    let config = serde_json::from_str(&content).context(loc::ERR_INVALID_CONFIG)?;
    Ok(Some(config))
}

pub fn save(config: &Config) -> Result<()> {
    let path = get_and_create_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content).context(loc::ERR_WRITE_CONFIG)?;
    Ok(())
}

pub fn parse(api_id: &str, api_hash: &str) -> Result<Config> {
    let api_id = api_id.trim().parse().context(loc::ERR_API_ID)?;
    let api_hash = api_hash.trim().to_lowercase();
    if api_hash.len() != API_HASH_LEN || !api_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!(loc::err_api_hash(API_HASH_LEN));
    }
    Ok(Config { api_id, api_hash })
}

fn get_and_create_config_path() -> Result<PathBuf> {
    let mut path = get_project_dirs()?.config_dir().to_owned();
    create_private_dir(&path)?;
    path.push(CONFIG_NAME);
    Ok(path)
}
