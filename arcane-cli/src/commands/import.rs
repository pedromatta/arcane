use arcane_core::{config::ImportManifest, db::schedule::import_manifest};
use sqlx::SqlitePool;

pub async fn import_cmd(pool: &SqlitePool, path: &str) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            match toml::from_str::<ImportManifest>(&content) {
                Ok(manifest) => {
                    match import_manifest(&manifest, pool).await {
                        Ok(_) => {
                            println!("Successfully imported categories and schedule from '{}'.", path);
                        }
                        Err(e) => {
                            eprintln!("Error importing manifest: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing manifest TOML: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading manifest file '{}': {}", path, e);
        }
    }
}
