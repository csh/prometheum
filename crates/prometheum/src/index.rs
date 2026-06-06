use std::{
    collections::HashMap,
    fs,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("error reading pak file: {0}")]
    RepakError(#[from] repak::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameIndex {
    pak_hash: String,
    file_mapping: HashMap<String, String>,
}

impl GameIndex {
    /// Create a local copy of the contents of the data.pak or if we've already processed the
    /// data.pak for this version of the game, load the data we're aware of from disk.
    ///
    /// Creates a mapping of file names to support mods that target top level file names
    /// (for example, a pak file may contain `D_Talents.json` instead of `Talents/D_Talents.json`)
    /// so we know where to route merged JSON documents during the packing process.
    pub fn init(data_pak_path: &Path, index_data_dir: &Path) -> Result<Self, IndexError> {
        let index_file = index_data_dir.join("index.json");
        let extracted_dir = index_data_dir.join("base");
        let hash = hash_file(data_pak_path)?;

        if index_file.exists() {
            let json_str = fs::read_to_string(&index_file)?;
            let cached = serde_json::from_str::<Self>(&json_str)?;
            if cached.pak_hash == hash {
                tracing::info!("Game index up to date, using cached data");
                return Ok(cached);
            }
            tracing::warn!("data.pak changed, rebuilding index");

            // ensure no old data survives
            fs::remove_dir(extracted_dir)?;
        }

        tracing::info!("building index");

        let index = Self::build(data_pak_path, index_data_dir, hash)?;
        tracing::debug!("writing index.json");
        fs::write(index_file, serde_json::to_string_pretty(&index)?)?;
        tracing::info!("indexed {} files", index.file_mapping.len());

        Ok(index)
    }

    /// Expand the game data.pak to a local directory for use in JSON diffing
    fn build(
        data_pak_path: &Path,
        index_data_dir: &Path,
        hash: String,
    ) -> Result<Self, IndexError> {
        let mut pak_file = File::open(data_pak_path)?;
        let reader = repak::PakBuilder::new().reader(&mut pak_file)?;
        let extract_dir = index_data_dir.join("base");
        let mut file_mapping = HashMap::new();

        for archive_file in reader.files() {
            tracing::debug!("processing {archive_file}");

            let Some(filename) = Path::new(&archive_file)
                .file_name()
                .and_then(|file| file.to_str())
            else {
                tracing::warn!(file = %archive_file, "error processing file");
                continue;
            };

            file_mapping.insert(filename.to_string(), archive_file.clone());

            let out_file = extract_dir.join(filename);
            if let Some(parent) = out_file.parent() {
                fs::create_dir_all(parent)?;
            }

            let data = reader.get(&archive_file, &mut pak_file)?;
            fs::write(out_file, data)?;

            tracing::debug!("extracted '{archive_file}'")
        }

        Ok(Self {
            pak_hash: hash,
            file_mapping,
        })
    }

    /// Resolve the full path of a file whether nested or flat.
    pub fn resolve_file(&self, file_path: &str) -> Option<&str> {
        self.file_mapping
            .get(file_path)
            .map(String::as_str)
            .or_else(|| {
                self.file_mapping
                    .values()
                    .find(|value| value.as_str() == file_path)
                    .map(String::as_str)
            })
    }
}

fn hash_file(path: &Path) -> Result<String, IndexError> {
    tracing::debug!("attempting to hash data.pak at {}", path.display());

    let mut file = {
        let file = File::open(path)?;
        BufReader::new(file)
    };

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let hash = hex::encode(hasher.finalize());
    tracing::debug!("data.pak hash is {}", hash);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_INSTALL_PATH: &str =
        r#"C:\Program Files (x86)\Steam\steamapps\common\Icarus\Icarus\Content\Data\data.pak"#;

    #[test]
    fn test_build_index_succeeds() {
        let data_pak_path = Path::new(DEFAULT_INSTALL_PATH);

        if !data_pak_path.exists() {
            println!("skipping test, game is not installed");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("create temp dir for test");
        let index_file = temp_dir.path().join("index.json");

        let index =
            GameIndex::init(data_pak_path, &index_file).expect("index creation should succeed");

        assert!(index_file.exists(), "index file should have been created");
        assert!(
            index.file_mapping.len() > 0,
            "index should contain more than 0 file mappings"
        );
    }

    #[test]
    fn test_path_lookup() {
        let mut file_mapping = HashMap::new();
        file_mapping.insert(
            "D_Talents.json".to_string(),
            "Talents/D_Talents.json".to_string(),
        );

        let index = GameIndex {
            pak_hash: "".into(),
            file_mapping,
        };

        assert!(index.resolve_file("D_Talents.json").is_some());
        assert!(index.resolve_file("Talents/D_Talents.json").is_some());
    }
}
