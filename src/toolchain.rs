//! Subcommands under toolchain.

use crate::{installer, registry, prelude::*};

pub async fn show() -> Result<()> {
    use installer::Installer;
    println!("MoonBit homedir: {}", global().moonhome.display());

    // download registry index
    let registry = registry::get().await?;
    if !(registry.toolchains.len() > 0) {
        return Err(anyhow!("registry error: no toolchains found"));
    }

    // iterate all toolchains, check if any
    let mut toolchains = registry.toolchains.clone();
    toolchains.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    for toolchain in &toolchains {
        let installer = installer::get_installer(&toolchain.installer)?;
        if installer.matches(toolchain).await? {
            println!("using {} toolchain.", &toolchain.name);
            return Ok(())
        }
    }

    println!("using a toolchain not listed in the registry. (run `moon version` to see version)");
    return Ok(());
}

pub async fn list() -> Result<()> {
    use installer::Installer;
    println!("MoonBit homedir: {}", global().moonhome.display());

    // download registry index
    let registry = registry::get().await?;
    if !(registry.toolchains.len() > 0) {
        return Err(anyhow!("registry error: no toolchains found"));
    }

    // print all toolchains
    let mut toolchains = registry.toolchains.clone();
    toolchains.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));

    let mut print = vec![];
    for toolchain in &toolchains {
        let installer = installer::get_installer(&toolchain.installer)?;
        let matches = installer.matches(toolchain).await?;
        
        print.push(format!("{} toolchain{}", &toolchain.name, if matches { " (current)" } else { "" }));
    }

    for print_line in print {
        println!("{}", print_line);
    }
    Ok(())
}

pub async fn update_to_latest() -> Result<()> {
    use installer::Installer;
    println!("MoonBit homedir: {}", global().moonhome.display());

    // download registry index
    let registry = registry::get().await?;
    if !(registry.toolchains.len() > 0) {
        return Err(anyhow!("registry error: no toolchains found"));
    }

    // find latest toolchain
    let mut toolchains = registry.toolchains.clone();
    toolchains.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    let latest_toolchain = &toolchains[0];

    // check if latest, install if not
    let latest_installer = installer::get_installer(&latest_toolchain.installer)?;
    let matches = latest_installer.matches(latest_toolchain).await?;
    if matches {
        println!("current installed toolchain is already latest version ({})", latest_toolchain.name);
        return Ok(());
    } else {
        latest_installer.install(&registry, latest_toolchain).await?;
        return Ok(());
    }
}

pub async fn update(args: &crate::cmdline::ToolchainUpdateArgs) -> Result<()> {
    use installer::Installer;
    println!("MoonBit homedir: {}", global().moonhome.display());

    // download registry index
    let registry = registry::get().await?;
    if !(registry.toolchains.len() > 0) {
        return Err(anyhow!("registry error: no toolchains found"));
    }

    // find latest toolchain
    let toolchain = match registry.toolchains.iter().filter(|&t| &t.name == &args.toolchain).next() {
        Some(t) => t,
        None => return Err(anyhow!("error: toolchain {} not found in registry", &args.toolchain)),
    };

    // check if latest, install if not
    let installer = installer::get_installer(&toolchain.installer)?;
    let matches = installer.matches(toolchain).await?;
    if matches && (!args.force) {
        println!("current installed toolchain is already {}. (add --force to reinstall)", toolchain.name);
        return Ok(());
    } else {
        installer.install(&registry, toolchain).await?;
        return Ok(());
    }
}

