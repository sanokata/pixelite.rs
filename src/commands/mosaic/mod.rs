pub mod postprocessing;
pub mod preprocessing;
pub mod processing;

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use image::{ImageFormat, open};

use crate::Result;

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum MosaicMode {
    #[default]
    Character,
    Background,
}

/// The arguments of mosaic sub-command
#[derive(Args, Debug)]
#[command(about = "Creates a mosaic-like pixel art from an image.")]
pub struct MosaicArgs {
    /// Input image file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output image file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// The size of the longest edge of the resulting pixel art (e.g., 64). Aspect ratio will be maintained.
    #[arg(short, long, default_value_t = 64)]
    pub size: u32,

    /// The maximum number of colors to use in the output image.
    #[arg(short, long, default_value_t = 16)]
    pub colors: u8,

    /// The mode of pixel art generation.
    #[arg(short = 'm', long, value_enum, default_value_t = MosaicMode::Character)]
    pub mode: MosaicMode,

    /// Whether to crop the input image to a square centered on the middle before processing.
    #[arg(long, default_value_t = false)]
    pub crop: bool,
}

pub fn run(args: MosaicArgs) -> Result<()> {
    let img = open(&args.input)?;
    // 0. preprocess: noise reduction / alpha normalization per mode
    let preprocessed = preprocessing::preprocess(img, &args.mode)?;
    // 0.5 crop if requested
    let prepared = if args.crop {
        processing::crop_to_square(preprocessed)
    } else {
        preprocessed
    };
    // 1. resize the original image to the desired size
    let resized = processing::resize_to_max_long_side(prepared, args.size, &args.mode)?;
    // 2. quantize the colors of the resized image to the desired number of colors
    let quantized = processing::quantize_colors(resized, args.colors, args.mode.clone())?;
    // 3. postprocess the quantized image
    let postprocessed = postprocessing::postprocess(quantized, &args.mode)?;
    // 4. save the postprocessed image to the output file
    let format = ImageFormat::from_path(&args.output).unwrap_or(ImageFormat::Png);
    postprocessed.save_with_format(&args.output, format)?;
    Ok(())
}
