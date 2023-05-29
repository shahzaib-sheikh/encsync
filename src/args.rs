use clap::{Arg, Parser, Subcommand};

/// Encsync is a simple CLI tool to encrypt and sync files
#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub struct Arguments {
    #[clap(subcommand)]
    pub subcommand: Subcommands,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Parser, Debug)]
pub enum Subcommands {
    /// Initialize encsync
    Init(InitSubcommand),
    /// Sync files
    Sync(SyncSubCommand),
    /// Restore files
    Restore(RestoreSubcommand),
}

#[derive(Parser, Debug)]
pub struct SyncSubCommand {
    /// The pattern to look for
    pub pattern: String,
    /// The path to the file to read
    pub path: std::path::PathBuf,
}

#[derive(Parser, Debug)]
pub struct RestoreSubcommand {
    /// The pattern to look for
    pub pattern: String,
    /// The path to the file to read
    pub path: std::path::PathBuf,
}

#[derive(Parser, Debug)]
pub struct InitSubcommand {
    /// Directory to initialize encsync in
    #[clap(long, short, default_value = "./")]
    pub path: std::path::PathBuf,
}


pub fn get_args() -> Arguments {
    Arguments::parse()
}