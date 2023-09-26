use std::env;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process;

use crate::error::Result;

use clap::{value_parser, Arg, Command};
use mach_object::{LoadCommand, MachCommand, OFile};
use memmap::MmapOptions;
use uuid::Uuid;
use walkdir::WalkDir;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn execute() -> Result<()> {
    let matches = Command::new("machfind")
        .version(VERSION)
        .about("This tool finds debug symbols by UUID.")
        .author(AUTHORS)
        .arg(
            Arg::new("uuid")
                .short('u')
                .long("uuid")
                .help("The UUID to find")
                // .index(1)
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .help("The path to start the search at")
                .default_value(env::current_dir().unwrap().into_os_string())
                // .index(2)
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    let base = matches.get_one::<PathBuf>("path").unwrap().as_path();
    let uuid: Uuid = match matches.get_one::<String>("uuid").unwrap().parse() {
        Ok(value) => value,
        Err(_) => {
            return Err("Invalid UUID".into());
        }
    };

    let wd = WalkDir::new(base);
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
            let mut cause = err.source();
            while let Some(the_cause) = cause {
                println!("  caused by: {}", the_cause);
                cause = the_cause.source();
            }
            process::exit(1);
        }
    }
}

fn get_uuids(path: &Path) -> Result<Vec<Uuid>> {
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let mut cursor = Cursor::new(mmap.as_ref());
    let ofile = OFile::parse(&mut cursor)?;
    let mut uuids = vec![];

    match ofile {
        OFile::FatFile { ref files, .. } => {
            for (_, file) in files {
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
    if let OFile::MachFile { commands, .. } = file {
        for MachCommand(load_cmd, _) in commands {
            if let &LoadCommand::Uuid(uuid) = load_cmd {
                uuids.push(uuid);
            }
        }
    }
}
