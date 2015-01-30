#![crate_name = "rduperemove"]
#![feature(plugin)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

extern crate btrfs;
extern crate crypto;
extern crate deque;

#[macro_use]
extern crate log;

#[plugin] #[no_link]
extern crate docopt_macros;

use std::old_io::{IoError, stdio};
use std::os;
use std::sync::Arc;

mod filehasher;
mod size_check;
mod hash_check;

const MIN_FILE_SIZE: usize = 4 * 1024;

struct Configuration {
    base_dirs:     Vec<Path>,
    worker_count:  usize,
    min_file_size: usize,
}

docopt!(CommandLineOptions, "
rduperemove - Whole-file deduplication for BTRFS filesystems on (Linux 3.13+).

Usage: rduperemove [options] <path>...
       rduperemove (-h|--help)

Options:
    <path>...                           One or more directories (on the same btrfs filesystem) \
                                        to deduplicate.
    -w <count>, --worker-count <count>  Number of workers threads to use [default: 4]
    -s <size>, --min-file-size <size>   Minimum file size to consider for deduplication [default: 4096]
    -h, --help                          Show this message
", flag_min_file_size: usize, flag_worker_count: usize);

fn main() {
    // hacky way to set up the default logging level. See
    // http://stackoverflow.com/questions/26142232/rust-change-the-default-log-level
    match os::getenv("RUST_LOG") {
        Some(_) => (),
        None    => os::setenv("RUST_LOG", "warn")
    };

    let config     = parse_options();
    let size_check = create_size_check(config.base_dirs, config.min_file_size);

    let dupes_rx = hash_check::spawn_workers(config.worker_count, size_check.size_groups());

    for mut paths in dupes_rx.iter() {
        for path in paths.iter() {
            println!("- {}", path.display());
        }

        let source = paths.pop().unwrap();

        let dedup  = btrfs::new_dedup(source, &paths[]);
        let deduped = dedup.perform();

        println!("Deduped {} bytes\n", deduped);
    }
}

fn create_size_check(base_dirs: Vec<Path>, min_file_size: usize) -> size_check::SizeCheck {
    let mut check = size_check::new_check(min_file_size);

    for base_dir in base_dirs.into_iter() {
        let mut stderr = stdio::stderr();

        let on_err = move |&mut: err: IoError| {
            (writeln!(&mut stderr, "WARNING: {}", err)).unwrap();
        };

        check.add_base_dir(Arc::new(base_dir), on_err).unwrap();
    }

    check
}

fn parse_options() -> Configuration {
    let options: CommandLineOptions = CommandLineOptions::docopt()
        .decode()
        .unwrap_or_else(|e| e.exit());

    let min_file_size = if options.flag_min_file_size >= MIN_FILE_SIZE {
         options.flag_min_file_size
     } else {
         warn!("Btrfs can't deduplicate files smaller than 4096 bytes. \
                Using that instead of the passed {}", options.flag_min_file_size);
         MIN_FILE_SIZE
     };

    let base_dirs = options.arg_path.into_iter().map(|base_dir| Path::new(base_dir)).collect();

    Configuration {
        worker_count: options.flag_worker_count,
        min_file_size: min_file_size,
        base_dirs: base_dirs,
    }
}
