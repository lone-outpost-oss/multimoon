//! Operations for installing MoonBit toolchain.

mod inst_initial;

use crate::prelude::*;
use crate::registry::Toolchain;

pub trait Installer
{
    async fn matches(&self, toolchain: &Toolchain) -> Result<bool>;
    async fn install(&self, registry: &crate::registry::Registry, toolchain: &crate::registry::Toolchain) -> Result<()>;
}

pub fn get_installer(name: &str) -> Result<impl Installer> {
    match name {
        "initial" | "2024-05-07" => Ok(inst_initial::InstInitial::new()),
        _ => Err(anyhow!("registry error: unknown installer {} (a new version of MultiMoon may be needed?)", name)),
    }
}
