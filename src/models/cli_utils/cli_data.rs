use core::str::FromStr as _;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::RwLock;

use color_eyre::eyre::Context as _;
use tagstudio_db::Library;

use crate::ColEyreVal;

pub static CLI_DATA: LazyLock<RwLock<CliData>> = LazyLock::new(|| RwLock::new(CliData::default()));

#[derive(Debug, Default)]
pub struct CliData {
    lib_path: Option<String>,
}

impl CliData {
    pub fn set_lib_path(&mut self, lib: String) {
        self.lib_path.replace(lib);
    }

    pub async fn get_library(&self) -> ColEyreVal<Library> {
        let lib_path = match &self.lib_path {
            Some(path) => {
                let path = PathBuf::from_str(&path).unwrap();
                if path.is_absolute() {
                    path
                } else {
                    path.canonicalize().context("Couldn't find the library")?
                }
            }
            None => current_dir().context("Couldn't get current working directory")?,
        };

        Ok(Library::try_new(lib_path).context("Couldn't get the root library")?)
    }
}
