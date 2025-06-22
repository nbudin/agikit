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

    fn read_file_bytes(&self, path: &str) -> Result<Vec<u8>, io::Error>
    where
        Self: Sized,
    {
        let mut file = self.open_file(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn read_file_utf8(&self, path: &str) -> Result<String, io::Error>
    where
        Self: Sized,
    {
        let bytes = self.read_file_bytes(path)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
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
