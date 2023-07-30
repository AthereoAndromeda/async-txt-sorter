use clap::Parser;

/// Sort large text files quickly
///
/// Allows you to sort files quickly. With a choice of either using
/// a low-memory mode or standard mode
#[derive(Parser, Debug, Clone)]
#[command(author, version)]
pub struct Args {
    /// Path to file
    pub path: String,

    /// Output path. Defaults to res.txt
    #[arg(short, long, value_name = "OUTPUT_PATH")]
    pub output: Option<String>,

    /// Determines on which character to split the file to. Defaults to newline
    #[arg(short, long, default_value_t = String::from("\n"))]
    pub delimiter: String,

    /// Determines how the output should be joined together. Defaults to newline
    #[arg(short = 'D', long, default_value_t = String::from("\n"))]
    pub output_delimiter: String,

    /// Lowers memory usage, but takes a lot longer. Disabled by default, but enables if the file is
    /// larger than 500MB
    #[arg(short = 'L', long)]
    pub low_memory_mode: bool,

    /// Disables low memory usage even for files larger than 500MB. Has no effect for files
    /// under 500MB
    #[arg(short = 'l', long)]
    pub disable_low_memory_mode: bool,
}
