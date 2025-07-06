#[cfg(test)]
use std::io::Cursor;
use std::{
    fs,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
    sync::Arc,
};

pub trait ReadSeek: Read + Seek {}
impl<T> ReadSeek for T where T: Read + Seek {}

pub trait FileProvider {
    fn base_path(&self) -> String;

    fn exists(&self, path: &str) -> bool;

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error>;

    fn read_file_bytes(&self, path: &str) -> Result<Vec<u8>, io::Error> {
        let mut file = self.open_file(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn read_file_utf8(&self, path: &str) -> Result<String, io::Error> {
        let bytes = self.read_file_bytes(path)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl FileProvider for PathBuf {
    fn base_path(&self) -> String {
        self.to_string_lossy().into_owned()
    }

    fn exists(&self, path: &str) -> bool {
        Path::exists(&self.join(path))
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error> {
        let full_path = self.join(path);
        fs::File::open(full_path).map(|f| Box::new(f) as Box<dyn ReadSeek>)
    }
}

impl FileProvider for Box<dyn FileProvider> {
    fn base_path(&self) -> String {
        self.as_ref().base_path()
    }

    fn exists(&self, path: &str) -> bool {
        self.as_ref().exists(path)
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error> {
        self.as_ref().open_file(path)
    }
}

impl FileProvider for Arc<dyn FileProvider> {
    fn base_path(&self) -> String {
        self.as_ref().base_path()
    }

    fn exists(&self, path: &str) -> bool {
        self.as_ref().exists(path)
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error> {
        self.as_ref().open_file(path)
    }
}

#[cfg(test)]
impl FileProvider for include_dir::Dir<'_> {
    fn base_path(&self) -> String {
        "[include_dir!]".to_string()
    }

    fn exists(&self, path: &str) -> bool {
        self.get_entry(self.path().join(path)).is_some()
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error> {
        let file = self.get_file(self.path().join(path)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("File not found: {}", path))
        })?;
        let cursor = Cursor::new(file.contents().to_vec());
        Ok(Box::new(cursor) as Box<dyn ReadSeek>)
    }
}

#[cfg(test)]
impl FileProvider for &include_dir::Dir<'_> {
    fn base_path(&self) -> String {
        (*self).base_path()
    }

    fn exists(&self, path: &str) -> bool {
        (*self).exists(path)
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error> {
        (*self).open_file(path)
    }
}
