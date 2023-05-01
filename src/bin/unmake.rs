//! CLI unmake tool

extern crate die;
extern crate getopts;
extern crate unmake;
extern crate walkdir;

use self::unmake::{inspect, warnings};
use die::{die, Die};
use std::env;
use std::fs;
use std::path;

lazy_static::lazy_static! {
    /// DIRECTORY_EXCLUSIONS
    pub static ref DIRECTORY_EXCLUSIONS: Vec<String> = vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "vendor".to_string(),
    ];
}

/// CLI entrypoint
fn main() {
    let brief: String = format!(
        "Usage: {} <OPTIONS> <path> [<path> ...]",
        env!("CARGO_PKG_NAME")
    );

    let mut opts: getopts::Options = getopts::Options::new();
    opts.optopt("i", "inspect", "summarize file details", "<makefile>");
    opts.optflag("d", "debug", "emit additional logs");
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

    let debug: bool = optmatches.opt_present("d");

    if optmatches.opt_present("i") {
        let pth_string = optmatches.opt_str("i").die(&usage);
        let pth: &path::Path = path::Path::new(&pth_string);
        let metadata: inspect::Metadata =
            inspect::analyze(pth).die(&format!("error: unable to read {}", pth_string));
        println!("{}", metadata);
        die!(0);
    }

    let pth_strings: Vec<String> = optmatches.free;

    if pth_strings.is_empty() {
        die!(1; usage);
    }

    let mut found_quirk = false;

    let mut action = |p: &path::Path| {
        let pth_string: String = p.display().to_string();
        let metadata: unmake::inspect::Metadata =
            unmake::inspect::analyze(p).die(&format!("error: unable to read {}", pth_string));

        if !metadata.is_makefile {
            return;
        }

        if metadata.is_machine_generated {
            if debug {
                eprintln!(
                    "debug: skipping {}: likely machine-generated by {}",
                    pth_string, metadata.build_system
                );
            }

            return;
        }

        if metadata.build_system != "make" {
            if debug {
                eprintln!(
                    "debug: skipping {}: non-strict implementation {}",
                    pth_string, metadata.build_system
                );
            }

            return;
        }

        let makefile_str: &str =
            &fs::read_to_string(p).die(&format!("error: unable to read {}", pth_string));

        let warnings_result: Result<Vec<warnings::Warning>, String> =
            warnings::lint(metadata, makefile_str);

        if let Err(err) = warnings_result {
            found_quirk = true;
            println!("{}", err);
            return;
        }

        let warnings: Vec<warnings::Warning> = warnings_result.unwrap();

        if !warnings.is_empty() {
            found_quirk = true;
        }

        for warning in warnings {
            println!("{}", warning);
        }
    };

    for pth_string in pth_strings {
        let pth: &path::Path = path::Path::new(&pth_string);

        if pth.is_dir() {
            let walker = walkdir::WalkDir::new(pth)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|e| {
                    !DIRECTORY_EXCLUSIONS
                        .contains(&e.file_name().to_str().unwrap_or("").to_string())
                });

            for entry_result in walker {
                let entry: walkdir::DirEntry = entry_result.unwrap();
                let child_pth: &path::Path = entry.path();

                if child_pth.is_dir() || child_pth.is_symlink() {
                    continue;
                }

                action(child_pth);
            }
        } else {
            action(pth);
        }
    }

    if found_quirk {
        die!(1);
    }
}
