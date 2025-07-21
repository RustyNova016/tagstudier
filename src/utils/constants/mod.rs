use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use directories::BaseDirs;

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BaseDirs::new()
        .expect(
            "Couldn't find the standard directory configuration. Is your system an oddball one?",
        )
        .config_dir()
        .to_path_buf();

    if !fs::exists(&path).unwrap() {
        fs::create_dir_all(&path).expect("Couldn't create config directory");
    }

    path
});

pub static LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CONFIG_DIR.to_path_buf();
    path.push("logs");

    if !fs::exists(&path).unwrap() {
        fs::create_dir_all(&path).expect("Couldn't create log directory");
    }

    path
});
