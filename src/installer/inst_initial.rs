//! Initial installer since 2024-05-07 toolchain.

use std::io::Read;

use anyhow::Context;

use crate::prelude::*;
use crate::installer::Installer;

pub struct InstInitial();

impl InstInitial {
    pub fn new() -> Self {
        return InstInitial();
    }
}

impl Installer for InstInitial {
    async fn matches(&self, toolchain: &crate::registry::Toolchain) -> Result<bool> {
        let moonhome = global().moonhome.clone();

        // use checksums of `.moon/bin/*` only to determine version (ignore `.moon/lib/core`)
        for binary in &toolchain.bin {
            use sha2::{Sha256, Digest};

            if !binary.checksum.starts_with("sha256:") {
                return Err(anyhow!("registry error: file {} has an invalid checksum", binary.filename));
            }

            let localpath = moonhome.join("bin").join(&binary.filename);
            let localfile = std::fs::read(&localpath)
                .with_context(|| { format!("error reading file {}", binary.filename) })?;
            let hash = format!("sha256:{}", base16ct::lower::encode_string(&Sha256::digest(&localfile)));
            
            if !(binary.checksum == hash) {
                return Ok(false);
            }
        }

        return Ok(true);
    }

    async fn install(&self, registry: &crate::registry::Registry, toolchain: &crate::registry::Toolchain) -> Result<()> {
        let client = reqwest::Client::new();

        // download all binaries from registry
        let url_prefix = Url::parse(&registry.downloadfrom)?.join(&format!("{}/{}/", toolchain.name, arch()))?;
        let index_download = Arc::new(AtomicI32::new(1));
        let index_download_end = Arc::new(AtomicI32::new(1));
        let mut tasks = tokio::task::JoinSet::new();
        let binary_file_count = toolchain.bin.len();
        for binary in &toolchain.bin {
            let index_download = index_download.clone();
            let index_download_end = index_download_end.clone();
            let url = url_prefix.join(&binary.downloadfrom)?;
            let client = client.clone();
            let binary = binary.clone();
            tasks.spawn(async move {
                use sha2::{Sha256, Digest};
                let index_download = index_download.fetch_add(1, SeqCst);

                // download a binary
                println!("downloading [bin {} / {}] {} ...", 
                    index_download,
                    binary_file_count,
                    &url,
                );
                let download_start = std::time::Instant::now();
                let response = client.get(url).send().await?.error_for_status()?;
                let download_duration = download_start.elapsed();
                let compressed = response.bytes().await?;
                
                // xz decompress
                let mut xzdecoder = xz2::bufread::XzDecoder::new(&compressed[..]);
                let mut filecontent = vec![];
                xzdecoder.read_to_end(&mut filecontent)?;

                // check checksum
                let hash = format!("sha256:{}", base16ct::lower::encode_string(&Sha256::digest(&filecontent)));
                if hash != binary.checksum {
                    return Err(anyhow!("checksum check for {} failed!", &binary.filename));
                }

                let index_download_end = index_download_end.fetch_add(1, SeqCst);
                println!("downloaded [bin {} / {}] {} ({:.2} KiB/s) ...", 
                    index_download_end,
                    binary_file_count,
                    &binary.filename,
                    (compressed.len() as f64) / download_duration.as_secs_f64() / 1024f64,
                );

                Ok::<_, anyhow::Error>((binary, filecontent))
            });
        }
        let mut binary_files = vec![];
        while let Some(res) = tasks.join_next().await {
            let (fileinfo, filecontent) = res.context("internal error: unable to run download task")??;
            binary_files.push((fileinfo, filecontent));
        }

        // download core from registry
        let url_prefix = Url::parse(&registry.downloadfrom)?.join(&format!("{}/{}/", toolchain.name, "multiarch"))?;
        let (core_file, mut core_archive) = {
            let core = toolchain.core.get(0).context("registry error: core not found")?;
            let url = url_prefix.join(&core.downloadfrom)?;
            let client = client.clone();
            let core = core.clone();
            tokio::spawn(async move {
                use sha2::{Sha256, Digest};

                // download a binary
                println!("downloading [lib 1 / 1] {} ...", &url);
                let download_start = std::time::Instant::now();
                let response = client.get(url).send().await?.error_for_status()?;
                let download_duration = download_start.elapsed();
                let zip_content = response.bytes().await?;

                // check checksum
                let hash = format!("sha256:{}", base16ct::lower::encode_string(&Sha256::digest(&zip_content)));
                if hash != core.checksum {
                    return Err(anyhow!("checksum check for {} failed!", &core.filename));
                }

                println!("downloaded [lib 1 / 1] {} ({:.2} KiB/s) ...", 
                    &core.filename,
                    (zip_content.len() as f64) / download_duration.as_secs_f64() / 1024f64,
                );

                let archive = zip::ZipArchive::new(std::io::Cursor::new(zip_content))?;

                Ok::<_, anyhow::Error>((core, archive))
            }).await.context("internal error: unable to run download task")??
        };

        // install all binaries to moonhome/bin
        let moonhome = global().moonhome.clone();
        let binary_path = moonhome.join("bin/");
        const CORRUPT: &'static str = "(current installation may be corrupted)";
        for (index, (fileinfo, filecontent)) in binary_files.iter().enumerate() {
            let filepath = binary_path.join(&fileinfo.filename);
            println!("installing [bin {} / {}] {} ...", 
                index + 1,
                binary_file_count,
                filepath.display());
            let mut infile = std::io::Cursor::new(filecontent);
            let mut outfile = std::fs::File::create(&filepath)
                .with_context(|| format!("install error: failed to create {} {}", filepath.display(), CORRUPT))?;
            std::io::copy(&mut infile, &mut outfile)
                .with_context(|| format!("install error: failed to write {} {}", filepath.display(), CORRUPT))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&filepath, std::fs::Permissions::from_mode(0o755))
                    .with_context(|| format!("install error: failed to set permission to {} {}", filepath.display(), CORRUPT))?;
            }
        }
        println!("succesfully installed binaries.");

        // extract core to moonhome/lib
        let lib_path = moonhome.join("lib/");
        println!("installing [core 1 / 1] {} ...", core_file.filename);
        let mut extracted_file_count = 0;
        for i in 0..(core_archive.len()) {
            let mut file = core_archive.by_index(i)
                .with_context(|| format!("extract error: failed to read file #{} from archive {}", i, CORRUPT))?;
            let outpath = match file.enclosed_name() {
                Some(path) => lib_path.join(&path),
                None => continue,
            };

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)
                    .with_context(|| format!("extract error: failed to create {} {}", outpath.display(), CORRUPT))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)
                            .with_context(|| format!("extract error: failed to create {} {}", outpath.display(), CORRUPT))?
                    }
                }
                if global().verbose || extracted_file_count < 5 {
                    println!("extract to {}", outpath.display());
                } else if (!global().verbose) && extracted_file_count == 5 {
                    println!(" (further extracted files omitted)");
                }
                let mut outfile = std::fs::File::create(&outpath)
                    .with_context(|| format!("extract error: failed to create {} {}", outpath.display(), CORRUPT))?;
                std::io::copy(&mut file, &mut outfile)
                    .with_context(|| format!("extract error: failed to write {} {}", outpath.display(), CORRUPT))?;

                extracted_file_count += 1;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))
                    .with_context(|| format!("extract error: failed to set permission to {} {}", outpath.display(), CORRUPT))?;
                }
            }
        }
        println!("succesfully extracted core library.");

        // bundle core in moonhome/lib
        let moon_path = binary_path.join(crate::global::moon_executable_name());
        let core_path = lib_path.join("core");
        let mut command = std::process::Command::new(&moon_path);
        command.args(["bundle", "--all"])
            .current_dir(&core_path)
            .env_clear()
            .env("PATH", &binary_path);
        println!("bundling core library (run {} in {})", {
            let mut r = command.get_program().to_string_lossy().to_string();
            for command_arg in command.get_args() {
                r.push_str(" ");
                r.push_str(&command_arg.to_string_lossy());
            }
            r
        }, command.get_current_dir().unwrap().display());
        let output = command
            .output()
            .context("error bundling core library")?;
        if (!output.status.success()) || global().verbose {
            use std::io::Write;
            let _ = std::io::stdout().write_all(&output.stdout);
            let _ = std::io::stderr().write_all(&output.stderr);
        }
        if !output.status.success() {
            return Err(anyhow!("failed to bundle core library (exit code: {})",
                output.status.code().unwrap_or(-1)));
        }
        println!("succesfully bundled core library.");
        println!("succesfully installed libraries.");

        println!("sucessfully installed toolchain {}.", &toolchain.name);

        Ok(())
    }
}
