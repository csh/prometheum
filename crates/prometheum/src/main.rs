use prometheum::import::pak_file::expand_pak_file;
use std::path::Path;

fn main() {
    tracing_subscriber::fmt::init();

    let dir =
        std::fs::read_dir("./data/sample-mods").expect("Failed to read sample mods directory");

    for file in dir {

        let Ok(file) = file else {
            continue;
        };

        let Ok(file_type) = file.file_type() else {
            tracing::warn!("skipping file: {:?}", file);
            continue;
        };

        if !file_type.is_file() {
            continue;
        }

        let path = file.path();
        tracing::info!("attempting to read {}", path.display());

        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .expect("Failed to get file extension");

        if ext != "pak" {
            continue;
        }

        expand_pak_file(path.as_path(), Path::new("./data/sample-mods/out"));
    }
}
