use filehasher;
use std::collections::HashMap;
use std::io::File;
use std::sync::Future;

static BUFFER_SIZE:  uint = 64 * 1024;

pub fn spawn_workers<T: Iterator<Vec<Path>> + Send>(count: uint, groups: T) -> Receiver<Vec<Path>> {
    let (results_tx, results_rx) = channel();

    spawn(proc() {
        let workers_txs = Vec::from_fn(count, |_| {
            let (worker_tx, worker_rx) = channel();

            let worker_results_tx = results_tx.clone();
            spawn(proc() worker(worker_rx, worker_results_tx));

            worker_tx
        });

        let workers_cycle = workers_txs.iter().cycle();

        for (size_group, worker_tx) in groups.zip(workers_cycle) {
            worker_tx.send(size_group);
        }
    });

    results_rx
}

fn worker(rx: Receiver<Vec<Path>>, tx: Sender<Vec<Path>>) {
    for paths in rx.iter() {
        let paths_by_digest = digest_files(paths);

        for (_, paths) in paths_by_digest.into_iter() {
            if paths.len() > 1 {
                tx.send(paths);
            }
        }
    }
}

fn digest_files(paths: Vec<Path>) -> HashMap<Vec<u8>, Vec<Path>> {
    let mut paths_by_digest = HashMap::with_capacity(paths.len());

    let future_digests = paths.into_iter().map(|path| {
        Future::spawn(proc() {
            let mut hasher = filehasher::new(BUFFER_SIZE);

            let file = File::open(&path).unwrap();
            (path, hasher.hash_whole_file(file))
        })
    }).collect::<Vec<Future<_>>>();

    for future in future_digests.into_iter() {
        let (path, digest): (Path, Vec<u8>) = future.unwrap();

        let dupes = paths_by_digest.find_or_insert_with(digest, |_| vec!());
        dupes.push(path);
    }

    paths_by_digest
}
