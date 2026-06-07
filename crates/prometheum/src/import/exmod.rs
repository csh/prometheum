use std::{collections::HashMap, fs::File, path::Path};

use crate::import::{ImportError, ImportedMod, ModImporter};
use crate::GameIndex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct ExmodManifest {
    pub name: String,
    pub author: String,
    pub version: String,

    #[serde(default)]
    pub description: String,

    #[serde(rename = "fileName")]
    pub file_name: String,

    #[serde(rename = "imageURL")]
    pub image_url: String,

    #[serde(rename = "readmeURL")]
    pub readme_url: String,

    #[serde(rename = "Level2")]
    pub level2: String,

    #[serde(default)]
    #[serde(rename = "Rows")]
    pub rows: Vec<ExModFilePatch>,
}

impl From<ExmodManifest> for ImportedMod {
    fn from(manifest: ExmodManifest) -> ImportedMod {
        ImportedMod {
            name: manifest.name,
            author: manifest.author,
            version: manifest.version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExModFilePatch {
    #[serde(rename = "CurrentFile")]
    pub current_file: String,

    #[serde(rename = "File_Items", default)]
    pub file_items: Vec<ExModRowPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExModRowPatch {
    #[serde(rename = "Name")]
    pub name: String,

    /// Everything except for the Name field
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

pub struct ExmodImporter;

impl ExmodImporter {
    fn load_exmod_manifest(mod_path: &Path) -> Result<ExmodManifest, ImportError> {
        let is_compressed = match mod_path.extension().and_then(|e| e.to_str()) {
            Some("EXMOD") => false,
            Some("EXMODZ") => true,
            _ => return Err(ImportError::UnsupportedFormat),
        };

        let data: ExmodManifest = if is_compressed {
            tracing::info!("loading compressed mod data");

            let mut zip = zip::ZipArchive::new(File::open(mod_path)?).map_err(|e| {
                tracing::error!(error = %e, file = %mod_path.display(), "failed to open EXZMOD");
                ImportError::UnsupportedFormat
            })?;

            let Some(mod_file) = zip.file_names().find_map(|f| {
                if f.ends_with("EXMOD") {
                    Some(f.to_owned())
                } else {
                    None
                }
            }) else {
                tracing::error!("failed to find an EXMOD file?");
                return Err(ImportError::UnsupportedFormat);
            };

            let file = zip.by_name(&mod_file).map_err(|e| {
                tracing::error!(error = %e, "found an exmod file but failed to read it?");
                ImportError::UnsupportedFormat
            })?;

            serde_json::from_reader(file)?
        } else {
            tracing::info!("loading exmod data");
            serde_json::from_reader(File::open(mod_path)?)?
        };

        dbg!(&data);

        Ok(data)
    }
}

impl ModImporter for ExmodImporter {
    fn import(
        index: &GameIndex,
        mod_path: &Path,
        data_dir: &Path,
    ) -> Result<ImportedMod, ImportError> {
        let manifest = Self::load_exmod_manifest(mod_path)?;

        todo!()
    }
}
