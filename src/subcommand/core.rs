//! Subcommands under core.

use crate::{core::{archive, core_backups_path, extract}, prelude::*};

pub async fn list() -> Result<()> {
    let result = crate::core::list().await?;
    for item in result {
        println!("{}", item);
    }
    Ok(())
}

pub async fn backup(args: &crate::cmdline::CoreBackupArgs) -> Result<()> {
    use std::io::Seek;
    println!("MoonBit homedir: {}", global().moonhome.display());
    use_multimoon_home().await?;
    
    // generate zip archive
    let lib_path = global().moonhome.join("lib");
    let archive = archive(&lib_path).await?;
    let mut archive_reader = archive.into_inner();
    archive_reader.seek(std::io::SeekFrom::Start(0))?;

    // backup name, specified or automatically generated
    let backup_name = std::iter::once(args.name.clone().unwrap_or_else(|| {
        let datetime = chrono::Local::now();
        datetime.format("%Y%m%d-%H%M%S").to_string()
    })).map(|s| {
        s.strip_suffix(".zip").map(|rest| rest.to_string()).unwrap_or(s)
    }).next().unwrap();
    
    // write archive to disk
    let write_path = core_backups_path().join(format!("{}.zip", backup_name));
    println!("writing backup file {}", write_path.display());
    let mut write_file = std::fs::File::create_new(write_path)?;
    std::io::copy(&mut archive_reader, &mut write_file)?;

    println!("core backup complete. backup name: {}", &backup_name);

    Ok(())
}

pub async fn restore(args: &crate::cmdline::CoreRestoreArgs) -> Result<()> {
    println!("MoonBit homedir: {}", global().moonhome.display());
    use_multimoon_home().await?;
    
    // load zip archive from disk
    let backup_name = std::iter::once(args.name.clone()).map(|s| {
        s.strip_suffix(".zip").map(|rest| rest.to_string()).unwrap_or(s)
    }).next().unwrap();
    let read_path = core_backups_path().join(format!("{}.zip", &backup_name));
    println!("reading backup file {}", read_path.display());
    let archive_file = std::fs::File::open(&read_path)?;
    let mut archive = zip::ZipArchive::new(archive_file)?;

    let lib_path = global().moonhome.join("lib");
    println!("extracting core to lib path {}", lib_path.display());
    extract(lib_path, &mut archive).await?;

    println!("core restored from backup {}.", &backup_name);

    Ok(())
}

async fn use_multimoon_home() -> Result<()> {
    let multimoonhome = global().multimoonhome.as_path();
    println!("MultiMoon storage dir: {}", multimoonhome.display());
    std::fs::create_dir_all(multimoonhome)?;
    let core_backups_path = multimoonhome.join("core-backups");
    std::fs::create_dir_all(&core_backups_path)?;
    Ok(())
}
