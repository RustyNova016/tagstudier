use std::path::PathBuf;

use extend::ext;
use tagstudio_db::models::library::Library;

use crate::exts::path::PathExt;

#[ext]
pub impl Library {
    fn get_download_folder(&self) -> PathBuf {
        let path = self.path.join("Tagerine Downloads");
        path.create_directory_if_not_exist().unwrap();
        path
    }
}
