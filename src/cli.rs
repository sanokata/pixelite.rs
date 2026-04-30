use crate::commands;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "pixelite",
    about = "A de facto standard Rust CLI tool with subcommands",
    version
)]
pub struct Cli {
    /// Optional name to operate on
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a mosaic from an image
    Mosaic(commands::mosaic::MosaicArgs),

    /// Extract or manage color palettes from images
    Palette(commands::palette::PaletteArgs),

    /// Assemble multiple images into a spritesheet
    Sheet(commands::sheet::SheetArgs),
}
