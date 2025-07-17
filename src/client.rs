use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

use tagstudio_db::client::TagStudioClient;

// static CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

// pub struct Client {
//     db: TagStudioClient
// }

// impl Client {
//     pub fn new(mut library_path: PathBuf) -> Self {
        
//     }
// }