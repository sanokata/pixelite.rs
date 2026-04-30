use clap::Parser;
use pixelite::cli::{Cli, Commands};
use pixelite::commands;

fn main() -> pixelite::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
    }

    match cli.command {
        // convert image file to pixel art
        Commands::Mosaic(args) => commands::mosaic::run(args)?,
        // extract color palette from image
        Commands::Palette(args) => commands::palette::run(args)?,
        // assemble images into a spritesheet
        Commands::Sheet(args) => commands::sheet::run(args)?,
    }

    Ok(())
}
