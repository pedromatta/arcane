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
        help = "Define a custom path to the config file",
        env = "ARCANE_CONFIG_FILE"
    )]
    pub config: Option<std::path::PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Log,
    Categories {
        #[command(subcommand)]
        subcommand: CategoryCommands,
    },
    Schedule {
        #[command(subcommand)]
        subcommand: ScheduleCommands,
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
    /// Remove a category
    Remove {
        /// Name of the category to remove
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ScheduleCommands {
    /// List all weekly scheduled slots
    List,
    /// Add a weekly scheduled slot
    Add {
        /// Category ID to schedule
        #[arg(long)]
        category_id: u32,

        /// Start time for the slot (HH:MM)
        #[arg(short, long)]
        time: String,

        /// Active weekdays bitmask (0-127)
        #[arg(short, long)]
        days: u8,
    },
    /// Remove a weekly scheduled slot
    Remove {
        /// ID of the schedule slot to remove
        #[arg(short, long)]
        id: u32,
    },
}
