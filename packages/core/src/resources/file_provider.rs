#[cfg(test)]
use std::io::Cursor;
use std::{
    fs,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

pub trait ReadSeek: Read + Seek {}
impl<T> ReadSeek for T where T: Read + Seek {}

pub trait WriteSeek: Write + Seek {}
impl<T> WriteSeek for T where T: Write + Seek {}

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

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error>;
}

pub trait WritableFileProvider {
    fn create_file<'a>(&'a self, path: &str) -> Result<Box<dyn WriteSeek + 'a>, io::Error>;
    fn create_dir_all(&self, path: &str) -> Result<(), io::Error>;
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

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error> {
        let full_path = match path {
            Some(path) => self.join(path),
            None => self.to_path_buf(),
        };

        Ok(full_path
            .read_dir()?
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|entry| entry.file_name().into_string().ok())
            })
            .collect())
    }
}

impl WritableFileProvider for PathBuf {
    fn create_file<'a>(&'a self, path: &str) -> Result<Box<dyn WriteSeek + 'a>, io::Error> {
        let full_path = self.join(path);
        fs::File::create(full_path).map(|f| Box::new(f) as Box<dyn WriteSeek>)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), io::Error> {
        let full_path = self.join(path);
        fs::create_dir_all(full_path)
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

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error> {
        self.as_ref().list_files(path)
    }
}

impl WritableFileProvider for Box<dyn WritableFileProvider> {
    fn create_file<'a>(&'a self, path: &str) -> Result<Box<dyn WriteSeek + 'a>, io::Error> {
        self.as_ref().create_file(path)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), io::Error> {
        self.as_ref().create_dir_all(path)
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

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error> {
        self.as_ref().list_files(path)
    }
}

impl WritableFileProvider for Arc<dyn WritableFileProvider> {
    fn create_file<'a>(&'a self, path: &str) -> Result<Box<dyn WriteSeek + 'a>, io::Error> {
        self.as_ref().create_file(path)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), io::Error> {
        self.as_ref().create_dir_all(path)
    }
}

impl<FP: FileProvider> FileProvider for Arc<FP> {
    fn base_path(&self) -> String {
        self.as_ref().base_path()
    }

    fn exists(&self, path: &str) -> bool {
        self.as_ref().exists(path)
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, io::Error> {
        self.as_ref().open_file(path)
    }

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error> {
        self.as_ref().list_files(path)
    }
}

impl<FP: WritableFileProvider> WritableFileProvider for Arc<FP> {
    fn create_file<'a>(&'a self, path: &str) -> Result<Box<dyn WriteSeek + 'a>, io::Error> {
        self.as_ref().create_file(path)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), io::Error> {
        self.as_ref().create_dir_all(path)
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

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error> {
        let dir = match path {
            Some(path) => self.get_dir(path).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("{} not found", path))
            })?,
            None => self,
        };
        Ok(dir
            .entries()
            .iter()
            .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
            .collect())
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

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, io::Error> {
        (*self).list_files(path)
    }
}
