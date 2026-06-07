pub mod pak_file;

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::GameIndex;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("not implemented")]
    NotImplemented,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Repak(#[from] repak::Error)

}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportedMod {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_author")]
    pub author: String,

    #[serde(default = "default_version")]
    pub version: String,

    pub source: PathBuf
}

fn default_name() -> String {
    String::from("Unknown Mod")
}

fn default_author() -> String {
    String::from("Unknown")
}

fn default_version() -> String {
    String::from("0.1.0")
}

pub trait ModImporter {
    fn import(index: &GameIndex, mod_path: &Path, data_dir: &Path) -> Result<ImportedMod, ImportError>;
}