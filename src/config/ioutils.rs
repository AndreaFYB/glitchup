use std::{error::Error, fs::{self, OpenOptions}, io::Write, path::PathBuf};

use image::DynamicImage;
use memmap2::MmapMut;

type LoaderResult<T> = Result<T,Box<dyn Error>>;

pub fn load_yaml_file(path: String) -> LoaderResult<serde_yaml::Value> {
    let config = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&config)?)
}

pub fn load_as_bytes_from_url(url: &str) -> LoaderResult<Vec<u8>> {
    Ok(reqwest::blocking::get(url)?.bytes()?.into())
}

pub fn load_as_bytes_from_path(path: &str) -> LoaderResult<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

pub fn load_as_mmap_from_path(path: &str) -> LoaderResult<MmapMut> {
    Ok(unsafe {
        MmapMut::map_mut(&OpenOptions::new()
            .read(true)
            .write(true)
            .open(&PathBuf::from(path))?)?
    })
}

pub fn load_as_image_from_bytes(bytes: &[u8]) -> LoaderResult<DynamicImage> {
    Ok(image::load_from_memory(bytes)?)
}

pub fn save_bytes_to_file(path: &str, bytes: &[u8]) -> LoaderResult<()> {
    let mut file = fs::OpenOptions::new()
        .create(true) 
        .write(true)
        .open(path)?;

    file.write_all(&bytes)?;

    Ok(())
}

pub fn setup_mmap_from_bytes(path: String, bytes: &[u8]) -> LoaderResult<MmapMut> {
    save_bytes_to_file(&path, bytes)?;

    load_as_mmap_from_path(&path)
}