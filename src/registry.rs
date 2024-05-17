//! Interacting with MultiMoon registries.

use crate::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Registry
{
    pub toolchains: Vec<Toolchain>,
    pub last_modified: i64,
    pub downloadfrom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Toolchain
{
    pub name: String,
    pub moonver: String,
    pub last_modified: i64,
    pub bin: Vec<File>,
    pub core: Vec<File>,
    pub installer: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File
{
    pub filename: String,
    pub downloadfrom: String,
    pub checksum: String,
}

pub async fn get() -> Result<Registry> {
    let url = global().registry.join(&format!("{}/", arch()))?;

    println!("downloading registry index from {}", &url);
    let response = reqwest::get(url).await?.error_for_status()?;
    let bytes = response.bytes().await?;
    let cursor = std::io::Cursor::new(bytes);

    let registry = serde_json::from_reader::<_, Registry>(cursor)?;

    Ok(registry)
}
