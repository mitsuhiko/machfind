//! Central error handling for the symbol server
use std::io;

use mach_object;
use walkdir;


error_chain! {
    foreign_links {
        Io(io::Error);
        WalkDir(walkdir::Error);
        MachO(mach_object::Error);
    }
}
