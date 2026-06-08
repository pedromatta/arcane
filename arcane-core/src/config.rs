use directories::ProjectDirs;

pub fn get_args_config_path(args: String) -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(args);
    if path.is_absolute() {
        Some(path)
    } else {
        std::env::current_dir().ok().map(|cwd| cwd.join(path))
    }
}

pub fn get_env_config_path() -> Option<std::path::PathBuf> {
    if let Ok(env_path) = std::env::var("ARCANE_CONFIG_FILE") {
        let path = std::path::PathBuf::from(env_path);
        if path.is_absolute() {
            Some(path)
        } else {
            std::env::current_dir().ok().map(|cwd| cwd.join(path))
        }
    } else {
        None
    }
}

pub fn get_default_config_path() -> Option<std::path::PathBuf> {
    let proj_dirs = ProjectDirs::from("", "Arcane", "arcane")?;
    let config_dir = proj_dirs.config_dir();

    generate_default_config(config_dir);

    Some(config_dir.join("config.toml"))
}

fn generate_default_config(config_dir: &std::path::Path) {
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir).expect("Failed to create config directory");
    }
    let config_path = config_dir.join("config.toml");
    if !config_path.exists() {
        let default_db_path = get_default_db_path()
            .expect("Failed to determine default database path")
            .to_str()
            .unwrap()
            .to_string();
        let default_config = format!(
            "[general]\ndatabase_path = \"{}\"\nnotifications_enabled = true\n",
            default_db_path
        );
        std::fs::write(config_path, default_config).expect("Failed to write default config file");
    }
}

fn get_default_db_path() -> Option<std::path::PathBuf> {
    let proj_dirs = ProjectDirs::from("", "Arcane", "arcane")?;
    let data_dir = proj_dirs.data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    }
    Some(data_dir.join("arcane.db"))
}
