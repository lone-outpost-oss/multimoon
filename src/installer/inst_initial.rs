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
            let localfile = match std::fs::read(&localpath) {
                Ok(content) => content,
                Err(err) => match err.kind() {
                    std::io::ErrorKind::NotFound => { 
                        return Ok(false); // doesn't match if some binary files missing
                    },
                    _ => { 
                        let e = Into::<anyhow::Error>::into(err)
                            .context(format!("error reading file {}", binary.filename));
                        return Err(e); 
                    }
                },
            };
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
        let binary_path = moonhome.join("bin");
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
        let lib_path = moonhome.join("lib");
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

        // set times of files and directories (best effort)
        // (first pass for files, second pass for directories)
        {
            for i in 0..(core_archive.len()) {
                let (file, outpath) = match core_archive.by_index(i) {
                    Ok(file) => match file.enclosed_name() {
                        Some(path) => (file, lib_path.join(&path)),
                        None => continue,
                    },
                    Err(_) => continue,
                };
                if file.is_file() {
                    let time = filetime::FileTime::from_unix_time(timestamp_from_zipfile(file, toolchain.last_modified), 0);
                    let _ = filetime::set_file_times(&outpath, time, time);
                }
            }
            for i in 0..(core_archive.len()) {
                let (file, outpath) = match core_archive.by_index(i) {
                    Ok(file) => match file.enclosed_name() {
                        Some(path) => (file, lib_path.join(&path)),
                        None => continue,
                    },
                    Err(_) => continue,
                };
                if file.is_dir() {
                    let time = filetime::FileTime::from_unix_time(timestamp_from_zipfile(file, toolchain.last_modified), 0);
                    let _ = filetime::set_file_times(&outpath, time, time);
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

        if let Err(e) = add_path_to_shell(&binary_path) {
            println!("error adding moonbit bin path {} to current shell config: {}",
                binary_path.display(),
                e
            );
            println!(" (you may have to add to your PATH manually)");
        }

        println!("sucessfully installed toolchain {}.", &toolchain.name);

        Ok(())
    }
}

fn timestamp_from_zipfile(file: zip::read::ZipFile, fallback: i64) -> i64 {
    use chrono::TimeZone;
    let z = file.last_modified();
    let (y, mo, d, h, mn, s) = (z.year().into(), z.month().into(), 
        z.day().into(), z.hour().into(), z.minute().into(), z.second().into());
    chrono::offset::Local
        .with_ymd_and_hms(y, mo, d, h, mn, s)
        .earliest()
        .map(|t| t.timestamp())
        .unwrap_or(fallback)
}

fn add_path_to_shell<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    #[cfg(windows)]
    {
        use winreg::{enums::*, RegKey};
        let path_str = path.as_ref().to_str().context("unsupported path name")?;

        const ERR_READ: &'static str = "cannot read registry";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_QUERY_VALUE | KEY_SET_VALUE)
            .context(ERR_READ)?;
        let path = env.get_value::<String, _>("Path").context(ERR_READ)?;

        if path.contains(path_str) {
            println!("{} has already been configured in user PATH environment variable.", path_str);
        } else {
            println!("adding {} to user PATH environment variable", path_str);
            const ERR_WRITE: &'static str = "cannot write registry";
            let path_new = format!("{};{}", path_str, &path);
            env.set_value("Path", &path_new).context(ERR_WRITE)?;
        }

        Ok(())
    }

    #[cfg(unix)]
    {
        let path_str = path.as_ref().to_str().context("unsupported path name")?;

        const ERR_DETECT: &'static str = "cannot detect current shell";
        let shell_path = PathBuf::from(std::env::var("SHELL").context(ERR_DETECT)?);
        let shell_filename = shell_path.file_name().context(ERR_DETECT)?.to_str().context(ERR_DETECT)?;
        let shell_config_path = global().home.join(match shell_filename {
            "bash" => ".bashrc",
            "zsh" => ".zshrc",
            "fish" => ".config/fish/config.fish",
            _ => ".profile",
        });
        let shell_config_path_str = shell_config_path.to_str().context("unsupported path name")?;

        const ERR_READ: &'static str = "cannot read shell config file";
        let mut shell_config_content = std::fs::read_to_string(&shell_config_path).context(ERR_READ)?;

        if shell_config_content.contains(path_str) {
            println!("{} has already been configured in PATH of shell config {}.", path_str, shell_config_path_str);
        } else {
            println!("adding {} to the PATH of current shell config: {}", path_str, shell_config_path_str);
            const ERR_WRITE: &'static str = "cannot write shell config file";
            shell_config_content.push('\n');
            shell_config_content.push_str(&format!("export PATH=\"{}:$PATH\"\n", path_str));
            std::fs::write(&shell_config_path, &shell_config_content).context(ERR_WRITE)?;
        }

        Ok(())
    }

    #[cfg(not(any(windows, unix)))]
    {
        compile_error!("unsupported platform")
    }
}
