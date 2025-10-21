use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;

use crate::ColEyre;

pub static FILE_CACHE: LazyLock<FileCache> = LazyLock::new(|| FileCache::default());

/// Cache the existing files in a filesystem
#[derive(Debug, Default)]
pub struct FileCache {
    cache: DashMap<PathBuf, Vec<PathBuf>>,
}

impl FileCache {
    pub fn read_dir(&self, dir: PathBuf) -> ColEyre {
        let mut dir_entry = self.cache.entry(dir.clone()).insert(Vec::new());
        for elem in read_dir(dir)? {
            let entry = elem?;
            dir_entry.push(entry.path());
        }

        Ok(())
    }

    pub fn get_or_read_dir(&self, path: &Path) -> ColEyre<Ref<'_, PathBuf, Vec<PathBuf>>> {
        match self.cache.get(path) {
            Some(val) => Ok(val),
            None => {
                self.read_dir(path.to_path_buf())?;
                Ok(self.cache.get(path).unwrap())
            }
        }
    }

    pub fn path_exist(&self, path: &Path) -> ColEyre<bool> {
        Ok(match path.parent() {
            Some(parent) => self.get_or_read_dir(parent)?.iter().any(|p| p == path),
            None => path.exists(),
        })
    }
}
