//! The prelude of this project.

pub use crate::global::{arch, global};
pub use std::path::PathBuf;
pub use std::sync::Arc;
pub use std::sync::atomic::{AtomicI32, Ordering::SeqCst};
pub use anyhow::{anyhow, Result};
pub use url::Url;
