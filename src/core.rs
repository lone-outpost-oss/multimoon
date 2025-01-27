//! Operations for installing core library.

use walkdir::WalkDir;
use std::{io::{Cursor, Read, Seek, Write}, time::UNIX_EPOCH};
use anyhow::Context;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct ExtractOptions {
    pub fallback_timestamp: i64,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self { fallback_timestamp: chrono::Utc::now().timestamp() }
    }
}

pub async fn archive<P: AsRef<Path>>(lib_path: P) -> Result<zip::ZipArchive<Cursor<Vec<u8>>>> {
    let lib_path = lib_path.as_ref();
    let lib_core_path = lib_path.join("core");

    println!("archiving {}", lib_core_path.display());

    // check is core
    if !(lib_core_path.exists()) {
        return Err(anyhow!("archive error: core path {} doesn't exist", lib_core_path.display()));
    }
    let moon_mod_json_path = lib_core_path.join("moon.mod.json");
    if !(moon_mod_json_path.exists()) {
        return Err(anyhow!("archive error: core json {} is missing", moon_mod_json_path.display()));
    }

    // create archive
    let mut zip = zip::ZipWriter::new(Cursor::new(vec![]));
    zip.set_comment("backup of MoonBit core, generated by MultiMoon");

    let ignore_target_path = PathBuf::from("core").join("target");
    let mut archived_count = 0;
    let walk = WalkDir::new(lib_core_path).sort_by_file_name();
    for entry in walk {
        let entry = entry?;
        let path_abs = entry.path();
        let path_rel = path_abs.strip_prefix(lib_path)?;
        let path_rel_str = path_rel.to_str().map(str::to_owned)
            .with_context(|| format!("path {} is not supported", path_rel.display()))?;

        // create options (use unix permissions of original file/dir)
        let options = {
            let o = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::DEFLATE)
                .compression_level(Some(6));
            let o = {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(path_abs)?;
                    let mode = metadata.permissions().mode();
                    o.unix_permissions(mode)
                }
                #[cfg(not(unix))]
                { o }
            };
            o
        };

        // ignore any target file
        if path_rel.starts_with(&ignore_target_path) {
            continue;
        }

        let (add_file, add_dir) = (path_abs.is_file(), (path_abs.is_dir() && (!path_rel.as_os_str().is_empty())));

        if add_file || add_dir {
            if global().verbose || archived_count < 5 {
                println!("archiving {}", path_rel.display());
            } else if (!global().verbose) && archived_count == 5 {
                println!(" (further archived files omitted)");
            }
            archived_count += 1;
        }

        if add_file {
            zip.start_file(path_rel_str, options)?;
            let mut file_buf = Vec::new();
            let mut file = std::fs::File::open(path_abs)?;
            file.read_to_end(&mut file_buf)?;
            zip.write_all(&file_buf)?;
        } else if add_dir {
            zip.add_directory(path_rel_str, options)?;
        }
    }

    let zip_buf: zip::ZipArchive<Cursor<Vec<u8>>> = zip.finish_into_readable()?;
    Ok(zip_buf)
}

pub async fn extract_verbose<P, R>(lib_path: P, archive: &mut zip::ZipArchive<R>, options: &ExtractOptions) -> Result<()>
    where P: AsRef<Path>, R: Read + Seek
{
    use crate::common::timestamp_from_zipfile;
    const CORRUPT: &'static str = "(current installation may be corrupted)";
    let lib_path = lib_path.as_ref();
    let lib_core_path = lib_path.join("core");

    let mut extracted_file_count = 0;
    for i in 0..(archive.len()) {
        let mut file = archive.by_index(i)
            .with_context(|| format!("extract error: failed to read file #{} from archive {}", i, CORRUPT))?;
        let outpath = match file.enclosed_name() {
            Some(path) => lib_path.join(&path),
            None => continue,
        };
        if !outpath.starts_with(&lib_core_path) {
            return Err(anyhow!("extract error: extracted path {} is not within core path {}! (invalid core archive?) {}", 
                outpath.display(), lib_core_path.display(), CORRUPT));
        }

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
        for i in 0..(archive.len()) {
            let (file, outpath) = match archive.by_index(i) {
                Ok(file) => match file.enclosed_name() {
                    Some(path) => (file, lib_path.join(&path)),
                    None => continue,
                },
                Err(_) => continue,
            };
            if file.is_file() {
                let time = filetime::FileTime::from_unix_time(timestamp_from_zipfile(file, options.fallback_timestamp), 0);
                let _ = filetime::set_file_times(&outpath, time, time);
            }
        }
        for i in 0..(archive.len()) {
            let (file, outpath) = match archive.by_index(i) {
                Ok(file) => match file.enclosed_name() {
                    Some(path) => (file, lib_path.join(&path)),
                    None => continue,
                },
                Err(_) => continue,
            };
            if file.is_dir() {
                let time = filetime::FileTime::from_unix_time(timestamp_from_zipfile(file, options.fallback_timestamp), 0);
                let _ = filetime::set_file_times(&outpath, time, time);
            }
        }
    }

    Ok(())
}

pub async fn list() -> Result<Vec<String>> {
    fn process_direntry(entry: Result<std::fs::DirEntry, std::io::Error>) -> Result<Option<(String, i64)>> {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_file() {
            let file_name = entry.file_name().into_string()
                .map_err(|osstr| anyhow!("unsupported file name {}", osstr.to_string_lossy()))?;
            if let Some(backup_name) = file_name.strip_suffix(".zip") {
                let timestamp = entry.metadata().ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|systime| systime.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs() as i64)
                    .unwrap_or(0);
                let disp_name = format!("{}", backup_name);
                return Ok(Some((disp_name, timestamp)));
            }
        }
        Ok(None)
    }

    let mut result = vec![];
    let readdir = std::fs::read_dir(core_backups_path())?;
    for entry in readdir {
        match process_direntry(entry) {
            Ok(Some(backup_name)) => {
                result.push(backup_name);
            },
            Ok(None) => (),
            Err(err) => {
                println!("ignoring a file due to error: {}", err)
            },
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    
    Ok(result.into_iter().map(|(filename, _)| filename).collect())
}

#[inline(always)]
pub async fn extract<P, R>(lib_path: P, archive: &mut zip::ZipArchive<R>) -> Result<()>
    where P: AsRef<Path>, R: Read + Seek
{
    extract_verbose(lib_path, archive, &(ExtractOptions::default())).await
}

pub fn core_backups_path() -> PathBuf {
    global().multimoonhome.as_path().join("core-backups")
}
