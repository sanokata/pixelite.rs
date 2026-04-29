use clap::Args;
use image::open;
use std::path::PathBuf;

use super::{DitherMethod, Palette};
use crate::Result;

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

    /// Dithering method to use
    #[arg(short, long, default_value = "none")]
    pub method: DitherMethod,
}

pub fn run(args: ApplyArgs) -> Result<()> {
    // 1. Load the palette
    let palette = Palette::load(&args.palette)?;
    println!("Loaded palette with {} colors", palette.0.len());

    // 2. Load the image
    let img = open(&args.input)?;

    // 3. Apply the palette
    let result = palette.apply(img, args.method)?;

    // 4. Save the result
    result.save(&args.output)?;
    println!(
        "Applied palette (method: {:?}) saved to {:?}",
        args.method, args.output
    );

    Ok(())
}
