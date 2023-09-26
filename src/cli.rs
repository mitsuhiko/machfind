#![cfg(feature = "cargo")]

use std::env;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process;

use error::Result;

use clap::{crate_authors, crate_version, value_parser, Command};
use mach_object::{LoadCommand, MachCommand, OFile};
use memmap;
use uuid::Uuid;
use walkdir::WalkDir;

fn execute() -> Result<()> {
    let matches = Command::new("machfind")
        .version(crate_version!())
        .about("This tool finds debug symbols by UUID.")
        .author(crate_authors!("\n"))
        .arg(
            arg!(
                -u --uuid <UUID> "The UUID to find"
            )
            .index(1)
            .required(true),
        )
        .value_parser(value_parser!(String))
        .arg(
            arg!(
                -p --path <PATH> "The path to start the search at (defaults to '.')"
            )
            .default_value(env::current_dir()?)
            .index(2)
            .required(false),
        )
        .value_parser(value_parser!(PathBuf))
        .get_matches();

    let base = matches.get_one("uuid");
    let uuid: Uuid = match matches.get_one("uuid").unwrap().parse() {
        Ok(value) => value,
        Err(_) => {
            return Err("Invalid UUID".into());
        }
    };

    let wd = WalkDir::new(&base);
    for dir_ev in wd {
        let dir = dir_ev?;
        let md = dir.metadata()?;
        if md.is_file() && md.len() > 0 {
            if let Ok(uuids) = get_uuids(dir.path()) {
                if uuids.contains(&uuid) {
                    println!("Found {}", dir.path().display());
                }
            }
        }
    }

    Ok(())
}

pub fn main() {
    match execute() {
        Ok(()) => {}
        Err(err) => {
            use std::error::Error;
            println!("error: {}", err);
            let mut cause = err.cause();
            while let Some(the_cause) = cause {
                println!("  caused by: {}", the_cause);
                cause = the_cause.cause();
            }
            process::exit(1);
        }
    }
}

fn get_uuids(path: &Path) -> Result<Vec<Uuid>> {
    let mmap = memmap::Mmap::open_path(path, memmap::Protection::Read)?;
    let mut cursor = Cursor::new(unsafe { mmap.as_slice() });
    let ofile = OFile::parse(&mut cursor)?;
    let mut uuids = vec![];

    match ofile {
        OFile::FatFile { ref files, .. } => {
            for &(_, ref file) in files {
                extract_uuids(&mut uuids, file);
            }
        }
        OFile::MachFile { .. } => {
            extract_uuids(&mut uuids, &ofile);
        }
        _ => {}
    }

    Ok(uuids)
}

fn extract_uuids<'a>(uuids: &'a mut Vec<Uuid>, file: &'a OFile) {
    if let &OFile::MachFile { ref commands, .. } = file {
        for &MachCommand(ref load_cmd, _) in commands {
            match load_cmd {
                &LoadCommand::Uuid(uuid) => {
                    uuids.push(uuid);
                }
                _ => {}
            }
        }
    }
}
