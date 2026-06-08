use clap::Subcommand;

#[derive(Subcommand)]
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
        #[arg(short, long, default_value = "#FFFFFF")]
        color: String,
    },
    /// List all categories
    List,
}
