use std::{fs, path::Path};

use crate::import::{ImportError, ImportedMod, ModImporter};
use crate::GameIndex;

pub struct PakImporter;

impl ModImporter for PakImporter {
    fn import(
        index: &GameIndex,
        mod_path: &Path,
        data_dir: &Path,
    ) -> Result<ImportedMod, ImportError> {
        let mut name = mod_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Mod");

        let mut pak_file = fs::File::open(mod_path)?;
        let reader = repak::PakBuilder::new().reader(&mut pak_file)?;

        Err(ImportError::NotImplemented)
    }
}

pub fn expand_pak_file(pak_path: &Path, out_dir: &Path) {
    let name = pak_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("Failed to get file name");
    let out_dir = out_dir.join(name);

    if let Some(parent) = out_dir.parent() {
        fs::create_dir_all(parent).expect("Failed to create directory");
    }

    let mut pak_file = fs::File::open(pak_path).expect("Failed to open pak file");
    let reader = repak::PakBuilder::new()
        .reader(&mut pak_file)
        .expect("Failed to read pak file");

    for file in reader.files() {
        let out_path = out_dir.join(&file);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }

        let mut out_file = fs::File::create(out_path).expect("Failed to create pak file");

        reader
            .read_file(&file, &mut pak_file, &mut out_file)
            .expect("Failed to read file");
    }
}
