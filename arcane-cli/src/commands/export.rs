use arcane_core::db::schedule::export_manifest;
use sqlx::SqlitePool;

pub async fn export_cmd(pool: &SqlitePool) {
    match export_manifest(pool).await {
        Ok(manifest) => match toml::to_string(&manifest) {
            Ok(toml_str) => {
                println!("{}", toml_str);
            }
            Err(e) => {
                eprintln!("Error serializing manifest to TOML: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Error exporting manifest from database: {}", e);
        }
    }
}
