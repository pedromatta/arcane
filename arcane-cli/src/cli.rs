use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
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

#[derive(Subcommand, Debug)]
pub enum Commands {
    Log,
    Categories {
        #[command(subcommand)]
        subcommand: CategoryCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CategoryCommands {
    /// Add a new category
    Add {
        /// Name of the category
        #[arg(short, long)]
        name: String,

        /// Default duration for tasks in this category (in minutes)
        #[arg(short, long, default_value_t = 25)]
        default_minutes: u32,

        /// Color of the category (hex code, eg. #FF5733)
        #[arg(long, default_value = "#FFFFFF")]
        color: String,
    },
    /// List all categories
    List,
}
