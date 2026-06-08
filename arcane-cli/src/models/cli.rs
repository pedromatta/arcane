use crate::Commands;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "Arcane", version = "0.1", about = "A Rust routine planner")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        global = true,
        short,
        long,
        help = "Define a custom path to the config file"
    )]
    pub config: Option<String>,
}
