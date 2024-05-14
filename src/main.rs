mod cmdline;
mod global;
mod installer;
mod prelude;
mod registry;
mod toolchain;

use crate::{global::GlobalInfo, prelude::*};

fn init(args: &cmdline::Args) {
    // moonhome
    let moonhome = args.moonhome.as_ref().map_or_else(|| {
        dirs::home_dir().expect("unable to detect user's home directory").join(".moon")
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
            moonhome,
            registry,
            verbose: args.verbose,
        }
    }).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    let args = cmdline::Args::parse();

    init(&args);

    match &args.command {
        cmdline::Command::Show => toolchain::show().await,
        cmdline::Command::Update => toolchain::update_to_latest().await,
        cmdline::Command::Toolchain(args) => {
            match &args.command {
                cmdline::ToolchainCommand::Show => toolchain::show().await,
                cmdline::ToolchainCommand::List => toolchain::list().await,
                cmdline::ToolchainCommand::Update(args) => toolchain::update(args).await,
                cmdline::ToolchainCommand::Rollback(args) => toolchain::update(args).await,
            }
        },
        cmdline::Command::Core(_) => todo!(),
        cmdline::Command::UpdateSelf => update_self().await,
    }
}

async fn update_self() -> Result<()> {
    println!("The update-self command is not implemented yet, sorry for the inconvinence! ");
    println!("");
    println!("Latest version of MultiMoon can be downloaded at:");
    println!("");
    println!("  https://github.com/lone-outpost-oss/multimoon/releases");
    return Ok(());
}
