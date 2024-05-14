//! The global data.

use crate::prelude::*;
use std::sync::OnceLock;

static GLOBAL: OnceLock<GlobalInfo> = OnceLock::new();

pub struct GlobalInfo {
    pub moonhome: PathBuf,
    pub registry: Url,
    pub verbose: bool,
}

pub fn global() -> &'static GlobalInfo {
    return GLOBAL.get().unwrap()
}

pub fn init<F: FnOnce() -> GlobalInfo>(f: F) -> Result<()> {
    if GLOBAL.get().is_none() {
        GLOBAL.get_or_init(f);
        Ok(())
    } else {
        Err(anyhow!("duplicate init"))
    }
}

pub const fn arch() -> &'static str {
    #![allow(dead_code)]

    #[cfg(all(target_os = "macos", target_arch = "aarch64", target_pointer_width = "64"))]
    return "macos_aarch64";

    #[cfg(all(target_os = "macos", target_arch = "x86_64", target_pointer_width = "64"))]
    return "macos_amd64";

    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_pointer_width = "64"))]
    return "ubuntu_amd64";

    #[cfg(all(target_os = "windows", target_arch = "x86_64", target_pointer_width = "64"))]
    return "windows_x64";

    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64", target_pointer_width = "64"),
        all(target_os = "macos", target_arch = "x86_64", target_pointer_width = "64"),
        all(target_os = "linux", target_arch = "x86_64", target_pointer_width = "64"),
        all(target_os = "windows", target_arch = "x86_64", target_pointer_width = "64"),
    )))]
    compile_error!("unsupported platform")
}

pub const fn moon_executable_name() -> &'static str {
    #![allow(dead_code)]

    #[cfg(all(target_os = "macos", target_arch = "aarch64", target_pointer_width = "64"))]
    return "moon";

    #[cfg(all(target_os = "macos", target_arch = "x86_64", target_pointer_width = "64"))]
    return "moon";

    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_pointer_width = "64"))]
    return "moon";

    #[cfg(all(target_os = "windows", target_arch = "x86_64", target_pointer_width = "64"))]
    return "moon.exe";

    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64", target_pointer_width = "64"),
        all(target_os = "macos", target_arch = "x86_64", target_pointer_width = "64"),
        all(target_os = "linux", target_arch = "x86_64", target_pointer_width = "64"),
        all(target_os = "windows", target_arch = "x86_64", target_pointer_width = "64"),
    )))]
    compile_error!("unsupported platform")
}
