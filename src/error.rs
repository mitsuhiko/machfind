//! Central error handling for the symbol server
use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);
        WalkDir(walkdir::Error);
        MachO(mach_object::MachError);
    }
}
