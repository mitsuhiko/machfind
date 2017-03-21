#![recursion_limit = "1024"]
extern crate uuid;
extern crate clap;
extern crate mach_object;
extern crate walkdir;
extern crate memmap;
#[macro_use] extern crate error_chain;

pub use cli::main;

mod error;
mod cli;
