use crate::loc;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::Path;

pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("dev", "fredol", "telegram-eraser").context(loc::ERR_HOME_DIR)
}

pub fn create_private_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
        .with_context(|| loc::err_create_dir(&path.display().to_string()))?;
    restrict_to_owner(path)
}

#[cfg(unix)]
fn restrict_to_owner(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .with_context(|| loc::err_permissions(&path.display().to_string()))
}

#[cfg(not(unix))]
fn restrict_to_owner(_: &Path) -> Result<()> {
    Ok(())
}
