use std::fs::File;
use std::fs::create_dir_all;
use std::fs::remove_dir;
use std::fs::remove_file;
use std::io;
use std::path::Path;

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
            match File::create_new(&self) {
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
}
