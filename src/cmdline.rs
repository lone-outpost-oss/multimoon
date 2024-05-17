//! Command line tools.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Command line arguments.
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    author,
    help_template = CLAP_HELP_TEMPLATE,
)]
pub struct Args {
    /// Subcommands.
    #[command(subcommand)]
    pub command: Command,

    /// Specify MultiMoon registry URL. (default to MultiMoon official registry by Lone Outpost Tech)
    #[arg(long)]
    pub registry: Option<String>,

    /// Specify installation path of MoonBit. (default to `.moon` in the user home directory)
    #[arg(long)]
    pub moonhome: Option<PathBuf>,

    /// Specify data storage location of MultiMoon. (default to `.multimoon` in the user home directory)
    #[arg(long)]
    pub multimoonhome: Option<PathBuf>,

    /// Verbose output.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Top level subcommand.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Show current installed toolchains.
    Show,

    /// Update MoonBit toolchains to latest version.
    Update,

    /// Manipulate MoonBit toolchains. (list, update or revert)
    Toolchain(ToolchainArgs),

    /// Manipulate the `core` library. (list, update, revert, backup or restore)
    Core(CoreArgs),

    /// Show a help for how to update MultiMoon itself. (Actual self update is not yet implemented)
    UpdateSelf,
}

/// Argument for `toolchain`.
#[derive(Parser, Debug)]
#[command()]
pub struct ToolchainArgs {
    /// Subcommands.
    #[command(subcommand)]
    pub command: ToolchainCommand,
    
}

/// Second level subcommand for `toolchain`.
#[derive(Subcommand, Debug)]
pub enum ToolchainCommand {
    /// Show current installed toolchains.
    Show,

    /// List all toolchains.
    List,

    /// Update MoonBit toolchains to latest or any specified version.
    Update(ToolchainUpdateArgs),

    /// Rollback MoonBit toolchains to any specified version. (same as update)
    Rollback(ToolchainUpdateArgs),
}

/// Argument for `toolchain update`.
#[derive(Parser, Debug)]
#[command()]
pub struct ToolchainUpdateArgs {
    /// Specified toolchain version.
    #[arg()]
    pub toolchain: String,

    /// Force reinstall even if specified toolchain is currently installed.
    #[arg(long)]
    pub force: bool,
}

/// Argument for `core`.
#[derive(Parser, Debug)]
#[command()]
pub struct CoreArgs {
    /// Subcommands.
    #[command(subcommand)]
    pub command: CoreCommand,
    
}

/// Second level subcommand for `core`.
#[derive(Subcommand, Debug)]
pub enum CoreCommand {
    /// Use MoonBit core library from a local path.
    Use,

    /// Update MoonBit core library to latest or any specified version.
    Update,

    /// Rollback MoonBit core library to any specified version. (same as update)
    Rollback,

    /// Backup current MoonBit core library.
    Backup(CoreBackupArgs),

    /// Restore a backup of MoonBit core library.
    Restore(CoreRestoreArgs),
}

/// Argument for `core backup`.
#[derive(Parser, Debug)]
#[command()]
pub struct CoreBackupArgs {
    /// Backup name. (defaults to current date and time)
    pub name: Option<String>,
}

/// Argument for `core restore`.
#[derive(Parser, Debug)]
#[command()]
pub struct CoreRestoreArgs {
    /// Restore name. (defaults to current date and time)
    pub name: Option<String>,
}


const CLAP_HELP_TEMPLATE: &'static str = "{before-help}{about-with-newline}
Presented by {author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";
