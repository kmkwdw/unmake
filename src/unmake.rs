//! CLI unmake tool

extern crate die;
extern crate getopts;
extern crate unmake;

use die::{die, Die};
use std::env;
use std::fs;
use std::path;

/// CLI entrypoint
fn main() {
    let brief: String = format!("Usage: {} <OPTIONS> <makefile>", env!("CARGO_PKG_NAME"));

    let mut opts: getopts::Options = getopts::Options::new();
    opts.optflag("h", "help", "print usage info");
    opts.optflag("v", "version", "print version info");

    let usage: String = opts.usage(&brief);
    let arguments: Vec<String> = env::args().collect();
    let optmatches: getopts::Matches = opts.parse(&arguments[1..]).die(&usage);

    if optmatches.opt_present("h") {
        die!(0; usage);
    }

    if optmatches.opt_present("v") {
        die!(0; format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));
    }

    let path_strings: Vec<String> = optmatches.free;

    if path_strings.len() != 1 {
        die!(1; usage);
    }

    let path_string: &String = path_strings.get(0).die(&usage);
    let p: &path::Path = path::Path::new(path_string);
    let md: fs::Metadata = fs::metadata(p).die("unable to access file path");

    if md.is_dir() {
        die!(1; usage);
    }

    let makefile_str: &str = &fs::read_to_string(p).die("unable to read makefile");

    if let Err(err) = unmake::parse_posix(makefile_str) {
        die!(err);
    };
}
