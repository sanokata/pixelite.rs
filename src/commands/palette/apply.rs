use std::path::PathBuf;
use clap::Args;
use image::open;

use crate::Result;
use super::Palette;

#[derive(Args, Debug)]
pub struct ApplyArgs {
    /// Input image file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Palette file path (.gpl or .png)
    #[arg(short, long)]
    pub palette: PathBuf,

    /// Output image file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Use dithering to represent gradients (not yet implemented)
    #[arg(short, long)]
    pub dither: bool,
}

pub fn run(args: ApplyArgs) -> Result<()> {
    // 1. Load the palette
    let palette = Palette::load(&args.palette)?;
    println!("Loaded palette with {} colors", palette.0.len());

    // 2. Load the image
    let img = open(&args.input)?;

    // 3. Apply the palette
    let result = palette.apply(img, args.dither);

    // 4. Save the result
    result.save(&args.output)?;
    println!("Applied palette saved to {:?}", args.output);

    Ok(())
}
