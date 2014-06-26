#![feature(macro_rules, unsafe_destructor)]

use std::io::{File, IoError, stdio};
use std::collections::HashMap;

#[allow(dead_code)]
mod gcrypt;
mod filehasher;
mod size_check;
mod btrfs;

static BUFFER_SIZE:  uint = 64 * 1024;
static WORKER_COUNT: uint = 4;

static MIN_FILE_SIZE: uint = 4 * 1024;

fn main() {
    gcrypt::init();

    let mut check = size_check::new_check(MIN_FILE_SIZE);
    for base_dir in std::os::args().move_iter().skip(1) {
        let mut stderr = stdio::stderr();

        let on_err = |err: IoError| {
            (writeln!(stderr, "WARNING: {}", err)).unwrap();
        };

        check.add_base_dir(&Path::new(base_dir), on_err).unwrap();
    }

    let mut buckets = Vec::from_fn(WORKER_COUNT, |_|  Vec::new());

    let mut worker_index = 0;
    for (_, paths) in check.size_groups() {
        buckets.get_mut(worker_index).push(paths);
        worker_index = (worker_index + 1) % WORKER_COUNT;
    }

    let (dupes_tx, dupes_rx) = channel();
    for bucket in buckets.move_iter() {
        let dupes_tx = dupes_tx.clone();

        spawn(proc() {
            let mut hasher = filehasher::new(BUFFER_SIZE);

            for paths in bucket.move_iter() {
                let mut paths_by_digest = HashMap::with_capacity(paths.len());

                for path in paths.move_iter() {
                    let file   = File::open(&path).unwrap();
                    let digest = hasher.hash_whole_file(file);

                    let dupes = paths_by_digest.find_or_insert_with(digest, |_| vec!());
                    dupes.push(path);
                }

                for (digest, paths) in paths_by_digest.move_iter() {
                    if paths.len() > 1 {
                        dupes_tx.send((digest, paths));
                    }
                }
            }
        });
    }

    drop(dupes_tx);

    for (_, mut paths) in dupes_rx.iter() {
        for path in paths.iter() {
            println!("- {}", path.display());
        }

        let source = paths.pop().unwrap();
        let dedup  = btrfs::new_dedup(source, paths.as_slice());

        let deduped = dedup.perform();
        println!("Deduped {} bytes\n", deduped);
    }
}
