use std::fs::File;
use std::fs::create_dir_all;
use std::fs::remove_dir;
use std::fs::remove_file;
use std::io;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use color_eyre::eyre::eyre;

use crate::ColEyreVal;

#[extend::ext]
pub impl Path {
    /// Create a directory if it doesn't already exist
    fn create_directory_if_not_exist(&self) -> Result<(), io::Error> {
        if !self.is_dir() {
            create_dir_all(self)?
        }

        Ok(())
    }

    fn create_file_if_not_exist(&self) -> Result<(), io::Error> {
        if !self.is_file() {
            match File::create_new(self) {
                Ok(_) => return Ok(()),
                Err(err) => match err.kind() {
                    io::ErrorKind::AlreadyExists => {
                        return Ok(());
                    }
                    _ => {
                        return Err(err);
                    }
                },
            };
        }

        Ok(())
    }

    fn delete_if_exists(&self) -> Result<(), io::Error> {
        if self.is_file() {
            remove_file(self)?;
        } else if self.is_dir() {
            remove_dir(self)?;
        }

        Ok(())
    }

    /// Normalize a path, including `..` without traversing the filesystem.
    ///
    /// Returns an error if normalization would leave leading `..` components.
    ///
    /// <div class="warning">
    ///
    /// This function always resolves `..` to the "lexical" parent.
    /// That is "a/b/../c" will always resolve to `a/c` which can change the meaning of the path.
    /// In particular, `a/c` and `a/b/../c` are distinct on many systems because `b` may be a symbolic link, so its parent isn’t `a`.
    ///
    /// </div>
    ///
    /// [`path::absolute`](absolute) is an alternative that preserves `..`.
    /// Or [`Path::canonicalize`] can be used to resolve any `..` by querying the filesystem.
    ///
    /// Extracted from https://github.com/rust-lang/rust/pull/134696/files
    fn normalize_lexically_stable(&self) -> ColEyreVal<PathBuf> {
        let mut lexical = PathBuf::new();
        let mut iter = self.components().peekable();

        // Find the root, if any, and add it to the lexical path.
        // Here we treat the Windows path "C:\" as a single "root" even though
        // `components` splits it into two: (Prefix, RootDir).
        let root = match iter.peek() {
            Some(Component::ParentDir) => return Err(eyre!("error")),
            Some(p @ Component::RootDir) | Some(p @ Component::CurDir) => {
                lexical.push(p);
                iter.next();
                lexical.as_os_str().len()
            }
            Some(Component::Prefix(prefix)) => {
                lexical.push(prefix.as_os_str());
                iter.next();
                if let Some(p @ Component::RootDir) = iter.peek() {
                    lexical.push(p);
                    iter.next();
                }
                lexical.as_os_str().len()
            }
            None => return Ok(PathBuf::new()),
            Some(Component::Normal(_)) => 0,
        };

        for component in iter {
            match component {
                Component::RootDir => unreachable!(),
                Component::Prefix(_) => return Err(eyre!("error")),
                Component::CurDir => continue,
                Component::ParentDir => {
                    // It's an error if ParentDir causes us to go above the "root".
                    if lexical.as_os_str().len() == root {
                        return Err(eyre!("error"));
                    } else {
                        lexical.pop();
                    }
                }
                Component::Normal(path) => lexical.push(path),
            }
        }
        Ok(lexical)
    }
}
