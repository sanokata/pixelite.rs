use image::{DynamicImage};

use super::MosaicMode;
use crate::Result;

/// Applies mode-appropriate preprocessing to the image before resizing and quantization.
pub fn preprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Character => preprocess_character(img),
        MosaicMode::Background => preprocess_background(img),
    }
}

fn preprocess_character(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn preprocess_background(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}
