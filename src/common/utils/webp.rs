use std::{fs, path::Path};

use image::DynamicImage;
use webp::Encoder;

use crate::Error;

pub fn save_as_webp<T>(image: &DynamicImage, quality: f32, path: T) -> Result<(), Error>
where
    T: AsRef<Path>,
{
    let encoder = match image {
        DynamicImage::ImageLuma8(_) => Err(String::from("Unimplemented")),
        DynamicImage::ImageLumaA8(_) => Err(String::from("Unimplemented")),
        DynamicImage::ImageRgb8(image) => {
            Ok(Encoder::from_rgb(image, image.width(), image.height()))
        }
        DynamicImage::ImageRgba8(image) => {
            Ok(Encoder::from_rgba(image, image.width(), image.height()))
        }
        _ => Err(String::from("Unimplemented")),
    }
    .map_err(Error::Webp)?;

    let webp = encoder.encode(quality);
    fs::write(path, &*webp)?;
    Ok(())
}
