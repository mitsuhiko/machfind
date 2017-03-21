use std::process;
use std::env;
use std::path::{Path, PathBuf};
use std::io::Cursor;

use error::Result;

use memmap;
use uuid::Uuid;
use walkdir::WalkDir;
use clap::{App, AppSettings, Arg};
use mach_object::{OFile, LoadCommand, MachCommand};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");


fn execute() -> Result<()> {
    let app = App::new("machfind")
        .version(VERSION)
        .about("This tool finds debug symbols by UUID.")
        .setting(AppSettings::ColorNever)
        .arg(Arg::with_name("uuid")
             .value_name("UUID")
             .index(1)
             .required(true)
             .help("The UUID to find"))
        .arg(Arg::with_name("path")
             .value_name("PATH")
             .index(2)
             .required(false)
             .help("The path to start the search at (defaults to '.')"));

    let matches = app.get_matches();

    let base = match matches.value_of("path") {
        Some(value) => PathBuf::from(value),
        None => env::current_dir()?,
    };
    let uuid : Uuid = match matches.value_of("uuid").unwrap().parse() {
        Ok(value) => value,
        Err(_) => { return Err("Invalid UUID".into()); }
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
        Ok(()) => {},
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
                },
                _ => {}
            }
        }
    }
}
