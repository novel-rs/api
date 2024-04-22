use std::{fs, path::Path};

use image::DynamicImage;
use webp::Encoder;

use crate::Error;

pub fn save_as_webp<T>(image: &DynamicImage, quality: f32, path: T) -> Result<(), Error>
where
    T: AsRef<Path>,
{
    let encoder = Encoder::from_image(image).map_err(|err| Error::Webp(err.to_string()))?;
    let webp = encoder.encode(quality);
    fs::write(path, &*webp)?;
    Ok(())
}
