use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use log::debug;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApplicationConfig {
    pub project_dirs: ProjectDirs,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectDirs {
    pub project_path: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_local_dir: PathBuf,
    pub data_dir: PathBuf,
    pub data_local_dir: PathBuf,
    pub preference_dir: PathBuf,
    pub database_path: PathBuf
}

impl From<directories::ProjectDirs> for ProjectDirs {
    fn from(value: directories::ProjectDirs) -> Self {
        ProjectDirs {
            project_path: value.project_path().to_path_buf(),
            cache_dir: value.cache_dir().to_path_buf(),
            config_dir: value.config_dir().to_path_buf(),
            config_local_dir: value.config_local_dir().to_path_buf(),
            data_dir: value.data_dir().to_path_buf(),
            data_local_dir: value.data_local_dir().to_path_buf(),
            preference_dir: value.preference_dir().to_path_buf(),
            database_path: value.data_dir().join("sks-trainer.db")
        }
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            project_dirs: project_dirs().unwrap().into()
        }
    }
}


/// In debug mode create project data in local path
#[cfg(debug_assertions)]
fn project_dirs() -> Option<directories::ProjectDirs> {
    let exe_path = std::env::current_exe().expect("Could not get the path of the current executable!");
    let mut data_path = PathBuf::from(exe_path.parent()?);
    data_path.push("data");

    debug!("Create path: {}", data_path.display());
    fs::create_dir_all(&data_path).expect("Data Path can not be created!");

    let data_path = fs::canonicalize(&data_path)
        .unwrap_or_else(|_| panic!("Data Path '{}' does not exist!", data_path.display()));


    let project_dirs = directories::ProjectDirs::from_path(data_path)?;

    debug!("{project_dirs:?}");
    Some(project_dirs)
}

/// In production mode create project data in the correct paths
#[cfg(not(debug_assertions))]
fn project_dirs() -> Option<ProjectDirs> {
    let project_dirs = ProjectDirs::from(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_HOMEPAGE"),
        env!("CARGO_PKG_NAME"))?;
    debug!("{project_dirs:?}");
    Some(project_dirs)
}