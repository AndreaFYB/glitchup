use std::fs::{OpenOptions, copy, rename, remove_file};
use memmap2::MmapMut;
use std::path::PathBuf;
use std::io::{Error, ErrorKind};

/// A Loader struct to facilitate file manipulation (loading, memorymapping...)
pub struct Loader;

impl Loader {
    /// Copies a file from `from` to `to`. 
    pub fn copy_file(from: &str, to: &str) -> std::io::Result<()>{
        copy(from, to)?;
        Ok(())
    }

    /// Constructs a mutable memory map of file at `name`.
    pub fn map_file_mut(name: &str) -> std::io::Result<memmap2::MmapMut> {
        let path : PathBuf = PathBuf::from(name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(mmap)
    }

    /// A combination of `copy_file` and `map_file_mut`.
    pub fn init_file_mut(from: &str, to:&str) -> std::io::Result<memmap2::MmapMut> {
        Loader::copy_file(from, to.clone())?;
        Loader::map_file_mut(to)
    }

    pub fn rename_file(from: &str, to: &str) -> std::io::Result<()> {
        if !Loader::file_exists(from) {
            Err(Error::new(ErrorKind::NotFound, format!("File '{}' does not exist.", from)))
        }
        else {
            rename(from, to)?;
            Ok(())
        }
    }

    pub fn remove_file(file: &str) -> std::io::Result<()> {
        if !Loader::file_exists(file) {
            Err(Error::new(ErrorKind::NotFound, format!("File '{}' does not exist.", file)))
        }
        else {
            remove_file(file)?;
            Ok(())
        }
    }

    #[inline]
    pub fn file_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}