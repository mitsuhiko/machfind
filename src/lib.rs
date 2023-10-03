#![recursion_limit = "1024"]
extern crate clap;
extern crate mach_object;
extern crate memmap;
extern crate uuid;
extern crate walkdir;
#[macro_use]
extern crate error_chain;

mod cli;
mod error;

pub use cli::main;
