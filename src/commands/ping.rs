use clap::Args;
use crate::Result;

#[derive(Args, Debug)]
pub struct PingArgs {
    /// The name to ping
    pub name: String,
}

pub fn run(args: PingArgs) -> Result<()> {
    println!("Pong! Hello, {}!", args.name);
    Ok(())
}
