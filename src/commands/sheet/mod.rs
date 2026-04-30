use std::path::PathBuf;

use clap::Args;

use crate::Result;

/// The arguments of sheet sub-command
#[derive(Args, Debug)]
#[command(about = "Assembles multiple images into a spritesheet.")]
pub struct SheetArgs {
    /// Input image path(s) or glob pattern(s) (e.g. "sprites/*.png")
    #[arg(short, long, required = true)]
    pub input: Vec<String>,

    /// Output spritesheet image file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Number of columns in the spritesheet grid
    #[arg(short, long)]
    pub columns: Option<u32>,
}

pub fn run(_args: SheetArgs) -> Result<()> {
    todo!("spritesheet assembly not yet implemented")
}
