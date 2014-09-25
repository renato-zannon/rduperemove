#![crate_name = "rduperemove"]
#![feature(macro_rules)]
extern crate libc;
extern crate native;
extern crate getopts;

use std::io::{IoError, stdio};

#[allow(dead_code)]
mod gcrypt;
mod filehasher;
mod size_check;
mod hash_check;
mod ioctl;
mod btrfs;

static WORKER_COUNT: uint = 4;
static MIN_FILE_SIZE: uint = 4 * 1024;

struct Configuration {
    base_dirs:     Vec<Path>,
    worker_count:  uint,
    min_file_size: uint,
}

fn main() {
    gcrypt::init();

    let config     = parse_options();
    let size_check = create_size_check(config.base_dirs, config.min_file_size);

    let dupes_rx = hash_check::spawn_workers(config.worker_count, size_check.size_groups());

    for mut paths in dupes_rx.iter() {
        for path in paths.iter() {
            println!("- {}", path.display());
        }

        let source = paths.pop().unwrap();

        let dedup  = btrfs::new_dedup(source, paths.as_slice());
        let deduped = dedup.perform();

        println!("Deduped {} bytes\n", deduped);
    }
}

fn create_size_check(base_dirs: Vec<Path>, min_file_size: uint) -> size_check::SizeCheck {
    let mut check = size_check::new_check(min_file_size);

    for base_dir in base_dirs.into_iter() {
        let mut stderr = stdio::stderr();

        let on_err = |err: IoError| {
            (writeln!(stderr, "WARNING: {}", err)).unwrap();
        };

        check.add_base_dir(&base_dir, on_err).unwrap();
    }

    check
}

fn parse_options() -> Configuration {
    use std::os;
    use getopts::{optopt, getopts};

    let options = [
        optopt("w", "worker",         "Number of base workers", "WORKERS"),
        optopt("s", "min-file-size",  "The minimal file size",  "SIZE IN BYTES"),
    ];

    let matches = match getopts(os::args().tail(), options) {
        Ok(m)  => m,
        Err(f) => fail!("{}", f),
    };

    let worker_count  = matches.opt_str("w").and_then(parse).unwrap_or(WORKER_COUNT);
    let min_file_size = matches.opt_str("s").and_then(parse).unwrap_or(MIN_FILE_SIZE);

    let base_dirs = matches.free.into_iter().map(|base_dir| Path::new(base_dir)).collect();

    return Configuration {
        worker_count: worker_count,
        min_file_size: min_file_size,
        base_dirs: base_dirs,
    };

    fn parse(s: String) -> Option<uint> {
        std::from_str::from_str(s.as_slice())
    }
}
