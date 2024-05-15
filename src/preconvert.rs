use std::{error::Error, io::Cursor};

use image::ImageFormat;

use crate::config::ioutils::load_as_image_from_bytes;



pub fn generic_preconvert(bytes: &[u8], format: ImageFormat) -> Result<Vec<u8>, Box<dyn Error>> {
    let image = load_as_image_from_bytes(bytes)?;
    let mut file = Vec::new();
    image.write_to(&mut Cursor::new(&mut file), format)?;
    Ok(file)
}