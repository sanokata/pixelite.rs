use std::path::PathBuf;
use clap::Args;
use image::open;

use crate::Result;
use super::Palette;

#[derive(Args, Debug)]
pub struct ExtractArgs {
    /// Input image file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output palette file path (currently only .png and .gpl are supported)
    #[arg(short, long)]
    pub output: PathBuf,

    /// The number of colors to extract
    #[arg(short, long, default_value_t = 16)]
    pub colors: u8,
}

pub fn run(args: ExtractArgs) -> Result<()> {
    let img = open(&args.input)?;
    let palette = Palette::extract(&img, args.colors, true)?;

    palette.save(&args.output)?;

    println!("Palette extracted to {:?}", args.output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba, DynamicImage};
    use std::collections::HashSet;

    #[test]
    fn test_extract_palette() {
        let mut img_buf = ImageBuffer::new(4, 4);
        for (x, y, pixel) in img_buf.enumerate_pixels_mut() {
            let r = if x < 2 { 255 } else { 0 };
            let g = if y < 2 { 255 } else { 0 };
            *pixel = Rgba([r, g, 0, 255]);
        }
        let img = DynamicImage::ImageRgba8(img_buf);

        let palette = Palette::extract(&img, 4, false).unwrap();
        assert_eq!(palette.0.len(), 4);

        let unique_colors: HashSet<_> = palette.0.iter().map(|c| c.0).collect();
        assert_eq!(unique_colors.len(), 4);
    }
}
