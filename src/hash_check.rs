use filehasher;

use std::collections::TreeMap;
use std::collections::SmallIntMap;

use std::sync::Arc;
use std::sync::deque::{mod, BufferPool};
use std::io::{IoError, File};

const BUFFER_SIZE:  uint = 64 * 1024;

struct SizeGroup {
    paths: Vec<Arc<Path>>,
    paths_per_digest: TreeMap<Vec<u8>, Vec<uint>>,
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

    spawn(proc() {
        let (job_results_tx, job_results_rx) = channel();

        let pool = BufferPool::new();
        let (w, stealer) = pool.deque();

        let size_groups = seed_workers(w, iter);

        for _ in range(0, count) {
            let stealer = stealer.clone();
            let worker_job_results_tx = job_results_tx.clone();

            spawn(proc() worker(stealer, worker_job_results_tx));
        }
        drop(job_results_tx);

        listen_for_responses(size_groups, job_results_rx, results_tx);
    });

    results_rx
}


fn listen_for_responses(
    mut size_groups: SmallIntMap<SizeGroup>,
    job_results_rx: Receiver<DigestJobResult>,
    results_tx: Sender<Vec<Arc<Path>>>)
{
    for job_result in job_results_rx.iter() {
        let (group_id, path_id) = job_result.id;

        let remaining = {
            let group: &mut SizeGroup = size_groups
                .find_mut(&group_id)
                .expect("Incomplete size group was removed!");

            match job_result.result {
                ResultSuccessful(digest) => {
                    let ref mut map = group.paths_per_digest;

                    let added = match map.find_mut(&digest) {
                        Some(v) => {
                            v.push(path_id);
                            true
                        },

                        None => false
                    };

                    if !added {
                        map.insert(digest, vec![path_id]);
                    }
                },

                ResultError(err) => {
                    error!("Error while trying to digest path: {}", err);
                }
            }

            group.remaining -= 1;
            group.remaining
        };

        if remaining > 0 {
            continue;
        } else {
            let group = size_groups.pop(&group_id).unwrap();

            for (_, path_ids) in group.paths_per_digest.iter() {
                if path_ids.len() < 2 { continue; }

                let paths: Vec<Arc<Path>> = path_ids.iter().map(|&path_id| {
                    group.paths[path_id].clone()
                }).collect();

                results_tx.send(paths);
            }
        }
    }
}

fn seed_workers<Iter>(worker: deque::Worker<DigestJob>, iter: Iter) -> SmallIntMap<SizeGroup>
    where Iter: Iterator<Vec<Arc<Path>>> + Send
{
    let mut size_groups: SmallIntMap<SizeGroup>;

    size_groups = iter.enumerate().map(|(group_id, paths)| {
        {
            for (path_id, path) in paths.iter().enumerate() {
                let job = DigestJob {
                    id: (group_id, path_id),
                    path: path.clone(),
                };

                worker.push(job);
            }
        }

        let group = SizeGroup {
            remaining: paths.len(),
            paths: paths,
            paths_per_digest: TreeMap::new()
        };

        (group_id, group)
    }).collect();

    size_groups
}

fn worker(stealer: deque::Stealer<DigestJob>, tx: Sender<DigestJobResult>) {
    let mut hasher = filehasher::new(BUFFER_SIZE);

    loop {
        let DigestJob { id: id, path: path } = match stealer.steal() {
            deque::Empty     => break,
            deque::Abort     => continue,
            deque::Data(job) => job,
        };

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
