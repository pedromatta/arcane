use crate::CategoryCommands;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the database and run embedded migrations
    Init,
    Log,
    Categories {
        #[command(subcommand)]
        subcommand: CategoryCommands,
    },
}
