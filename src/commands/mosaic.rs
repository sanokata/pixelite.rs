use std::path::PathBuf;

use clap::Args;

use crate::Result;

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
}

pub fn run(args: MosaicArgs) -> Result<()> {
    Ok(())
}
