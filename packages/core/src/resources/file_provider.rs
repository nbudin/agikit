#[cfg(test)]
use std::io::Cursor;
use std::{
    fs,
    io::{self, Read, Seek},
    path::PathBuf,
};

pub trait FileProvider {
    fn open_file(&self, path: &str) -> Result<impl Read + Seek, io::Error>
    where
        Self: Sized;
}

impl FileProvider for PathBuf {
    fn open_file(&self, path: &str) -> Result<impl Read + Seek, io::Error> {
        let full_path = self.join(path);
        fs::File::open(full_path)
    }
}

#[cfg(test)]
impl FileProvider for &include_dir::Dir<'_> {
    fn open_file(&self, path: &str) -> Result<impl Read + Seek, io::Error> {
        let file = self.get_file(self.path().join(path)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("File not found: {}", path))
        })?;
        let cursor = Cursor::new(file.contents().to_vec());
        Ok(cursor)
    }
}

#[cfg(test)]
impl FileProvider for include_dir::Dir<'_> {
    fn open_file(&self, path: &str) -> Result<impl Read + Seek, io::Error> {
        let file = self.get_file(self.path().join(path)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("File not found: {}", path))
        })?;
        let cursor = Cursor::new(file.contents().to_vec());
        Ok(cursor)
    }
}
