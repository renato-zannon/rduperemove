use filehasher;

use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};

use std::sync::Arc;
use std::io::{IoError, File};

static BUFFER_SIZE:  uint = 64 * 1024;

struct SizeGroup {
    paths: Vec<Arc<Path>>,
    paths_per_digest: HashMap<Vec<u8>, Vec<uint>>,
    remaining: uint,
}

struct DigestJob {
    id: (uint, uint),
    path: Arc<Path>,
}

struct DigestJobResult {
    id: (uint, uint),
    result: DigestResult,
}

enum DigestResult {
    ResultSuccessful(Vec<u8>),
    ResultError(IoError),
}

pub fn spawn_workers<Iter>(count: uint, iter: Iter) -> Receiver<Vec<Arc<Path>>>
    where Iter: Iterator<Vec<Arc<Path>>> + Send
{
    let (results_tx, results_rx) = channel();

    spawn(proc() spawn_workers_manager(count, iter, results_tx));
    results_rx
}


fn spawn_workers_manager<Iter>(count: uint, iter: Iter, results_tx: Sender<Vec<Arc<Path>>>)
    where Iter: Iterator<Vec<Arc<Path>>> + Send
{

    let (job_results_tx, job_results_rx) = channel();

    let worker_txs  = spawn_worker_txs(count, job_results_tx);
    let size_groups = seed_workers(worker_txs, iter);

    listen_for_responses(size_groups, job_results_rx, results_tx);
}


fn listen_for_responses(
    mut size_groups: Vec<SizeGroup>,
    job_results_rx: Receiver<DigestJobResult>,
    results_tx: Sender<Vec<Arc<Path>>>)
{
    for job_result in job_results_rx.iter() {
        let (group_id, path_id) = job_result.id;

        let ref mut group = size_groups.as_mut_slice()[group_id];

        match job_result.result {
            ResultSuccessful(digest) => {
                match group.paths_per_digest.entry(digest) {
                    Occupied(entry) => {
                        entry.into_mut().push(path_id);
                    },

                    Vacant(entry) => {
                        entry.set(vec![path_id]);
                    }
                }
            },

            ResultError(err) => {
                error!("Error while trying to digest path: {}", err);
            }
        }

        if group.remaining > 1 {
            group.remaining -= 1;
            continue;
        }

        for (_, path_ids) in group.paths_per_digest.iter() {
            if path_ids.len() < 2 { continue; }

            let paths: Vec<Arc<Path>> = path_ids.iter().map(|&path_id| {
                group.paths[path_id].clone()
            }).collect();

            results_tx.send(paths);
        }
    }
}

fn spawn_worker_txs(count: uint, job_results_tx: Sender<DigestJobResult>) -> Vec<Sender<DigestJob>> {
    Vec::from_fn(count, |_| {
        let (worker_tx, worker_rx) = channel();

        let worker_job_results_tx = job_results_tx.clone();
        spawn(proc() worker(worker_rx, worker_job_results_tx));

        worker_tx
    })
}

fn seed_workers<Iter>(worker_txs: Vec<Sender<DigestJob>>, iter: Iter) -> Vec<SizeGroup>
    where Iter: Iterator<Vec<Arc<Path>>> + Send
{
    let workers_cycle = worker_txs.iter().cycle();

    let mut size_groups = Vec::new();

    for (group_id, paths) in iter.enumerate() {
        {
            let mut cycle = paths.iter().zip(workers_cycle).enumerate();

            for (path_id, (path, worker_tx)) in cycle {
                let job = DigestJob {
                    id: (group_id, path_id),
                    path: path.clone(),
                };

                worker_tx.send(job);
            }
        }

        size_groups.push(SizeGroup {
            remaining: paths.len(),
            paths: paths,
            paths_per_digest: HashMap::new()
        });
    }

    size_groups
}

fn worker(rx: Receiver<DigestJob>, tx: Sender<DigestJobResult>) {
    let mut hasher = filehasher::new(BUFFER_SIZE);

    for DigestJob { id: id, path: path } in rx.iter() {
        let result = match File::open(& *path) {
            Ok(file) => {
                let digest = hasher.hash_whole_file(file);
                ResultSuccessful(digest)
            },

            Err(err) => {
                ResultError(err)
            }
        };

        tx.send(DigestJobResult { id: id, result: result });
    }
}
