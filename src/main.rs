use clap::Parser;
use pixelite::cli::{Cli, Commands};
use pixelite::commands;

fn main() -> pixelite::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
    }

    match cli.command {
        Commands::Ping(args) => commands::ping::run(args)?,
    }

    Ok(())
}
