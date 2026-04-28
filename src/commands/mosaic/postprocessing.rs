use image::DynamicImage;

use crate::Result;

use super::MosaicMode;

/// Post-processes the image after quantization.
pub fn postprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Character => {
            let img = orphan_pixel_removal(img)?;
            let img = pixel_perfection(img)?;
            let img = outlining(img)?;
            let img = de_anti_aliasing(img)?;
            Ok(img)
        }
        MosaicMode::Background => {
            let img = de_anti_aliasing(img)?;
            Ok(img)
        }
    }
}

fn orphan_pixel_removal(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn pixel_perfection(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn outlining(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn de_anti_aliasing(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}
