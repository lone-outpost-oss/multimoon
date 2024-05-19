mod cmdline;
mod common;
mod core;
mod global;
mod installer;
mod prelude;
mod registry;
mod subcommand;

use crate::{global::GlobalInfo, prelude::*};

fn init(args: &cmdline::Args) {
    // home
    let home = dirs::home_dir().expect("unable to detect user's home directory");

    // moonhome
    let moonhome = args.moonhome.as_ref().map_or_else(|| {
        home.join(".moon")
    }, |value| {
        value.clone()
    });

    // multimoonhome
    let multimoonhome = args.multimoonhome.as_ref().map_or_else(|| {
        home.join(".multimoon")
    }, |value| {
        value.clone()
    });

    // registry
    let registry_str = args.registry.as_ref().map_or_else(|| {
        "https://multimoon.lopt.dev/".to_string()
    }, |value| {
        value.clone()
    });
    let registry = Url::parse(&registry_str).expect("invalid registry url");

    global::init(move || {
        GlobalInfo {
            home,
            multimoonhome,
            moonhome,
            registry,
            verbose: args.verbose,
        }
    }).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    use subcommand::{core, toolchain};
    let args = cmdline::Args::parse();

    init(&args);

    match &args.command {
        cmdline::Command::Show => toolchain::show().await,
        cmdline::Command::Update => toolchain::update_to_latest().await,
        cmdline::Command::Toolchain(args) => {
            match &args.command {
                cmdline::ToolchainCommand::Show => toolchain::show().await,
                cmdline::ToolchainCommand::List => toolchain::list().await,
                cmdline::ToolchainCommand::Update(a) => toolchain::update(a).await,
                cmdline::ToolchainCommand::Rollback(a) => toolchain::update(a).await,
            }
        },
        cmdline::Command::Core(args) => {
            match &args.command {
                cmdline::CoreCommand::List => core::list().await,
                cmdline::CoreCommand::Backup(a) => core::backup(a).await,
                cmdline::CoreCommand::Restore(a) => core::restore(a).await,
            }
        },
        cmdline::Command::UpdateSelf => update_self().await,
    }.map_err(|e| {
        // println!("Error backtrace: {:?}", e.backtrace());
        e
    })
}

async fn update_self() -> Result<()> {
    println!("The update-self command is not implemented yet, sorry for the inconvinence! ");
    println!("");
    println!("Latest version of MultiMoon can be downloaded at:");
    println!("");
    println!("  https://github.com/lone-outpost-oss/multimoon/releases");
    return Ok(());
}
