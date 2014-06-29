#![feature(macro_rules, unsafe_destructor)]
extern crate libc;
extern crate native;

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

fn main() {
    gcrypt::init();

    let dupes_rx = hash_check::spawn_workers(WORKER_COUNT,
        create_size_check().size_groups()
    );

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

fn create_size_check() -> size_check::SizeCheck {
    let mut check = size_check::new_check(MIN_FILE_SIZE);

    for base_dir in std::os::args().move_iter().skip(1) {
        let mut stderr = stdio::stderr();

        let on_err = |err: IoError| {
            (writeln!(stderr, "WARNING: {}", err)).unwrap();
        };

        check.add_base_dir(&Path::new(base_dir), on_err).unwrap();
    }

    check
}
