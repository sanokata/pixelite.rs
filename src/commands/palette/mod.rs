use crate::Result;
use clap::Args;
use std::path::PathBuf;

/// The arguments of palette sub-command
#[derive(Args, Debug)]
#[command(about = "Manage color palettes: extract, apply, or preview.")]
pub struct PaletteArgs {
    /// Input image file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output palette file path (e.g., .ase, .gpl, or .png)
    #[arg(short, long)]
    pub output: PathBuf,

    /// The number of colors to extract
    #[arg(short, long, default_value_t = 16)]
    pub colors: u8,
}

pub fn run(args: PaletteArgs) -> Result<()> {
    Ok(())
}
