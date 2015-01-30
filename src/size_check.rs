use std::collections::{HashMap, HashSet, BinaryHeap};
use std::collections::hash_map::Entry;

use std::sync::Arc;
use std::old_io::{FileType, IoResult, IoError, FileStat};
use std::old_io::fs::PathExtensions;
use std::old_io;

pub struct SizeCheck {
    min_size: usize,
    groups:   HashMap<usize, Vec<StatedPath>>
}

pub fn new_check(min_size: usize) -> SizeCheck {
    SizeCheck { groups: HashMap::new(), min_size: min_size }
}

impl SizeCheck {
    #[must_use]
    pub fn add_base_dir<F: FnMut(IoError)>(&mut self, dir: Arc<Path>, mut on_err: F) -> IoResult<()> {
        for file in try!(recurse_directory(&dir)) {
            match file {
                Ok(stated_path) => {
                    let size = stated_path.stat.size as usize;

                    if size < self.min_size { continue; }

                    match self.groups.entry(size) {
                        Entry::Vacant(entry) => {
                            entry.insert(vec!(stated_path));
                        },

                        Entry::Occupied(entry) => {
                            entry.into_mut().push(stated_path);
                        },
                    };
                },

                Err(err) => {
                    on_err(err);
                }
            }
        }

        Ok(())
    }

    pub fn size_groups(self) -> SizeGroups {
        let sizes = self.groups.keys()
            .map(|n| *n)
            .collect::<BinaryHeap<usize>>()
            .into_sorted_vec();

        SizeGroups {
            sorted_sizes_iter: sizes,
            size_groups: self.groups,
        }
    }
}

pub struct SizeGroups {
    sorted_sizes_iter: Vec<usize>,
    size_groups: HashMap<usize, Vec<StatedPath>>
}

impl Iterator for SizeGroups {
    type Item = Vec<Arc<Path>>;

    fn next(&mut self) -> Option<Vec<Arc<Path>>> {
        for size in self.sorted_sizes_iter.drain().rev() {
            let stated_paths = self.size_groups.remove(&size).unwrap();
            let unique_stated_paths = remove_repeated_inodes(stated_paths);

            if unique_stated_paths.len() < 2 { continue; }

            let unique_paths = unique_stated_paths.into_iter().map(|stated_path| {
                stated_path.path
            }).collect();

            return Some(unique_paths);
        }

        None
    }
}

fn remove_repeated_inodes(mut stated_paths: Vec<StatedPath>) -> Vec<StatedPath> {
    let mut found = HashSet::with_capacity(stated_paths.len());

    stated_paths.retain(|path| {
        let inode = path.stat.unstable.inode;

        // insert returns false if the value was already on the set
        found.insert(inode)
    });

    stated_paths
}

fn recurse_directory(dir: &Arc<Path>) -> IoResult<FilesBelow> {
    let stat = try!(dir.stat());

    match stat.kind {
        FileType::Directory => Ok(FilesBelow { stack: vec!(dir.clone()) }),
        _  => {
            Err(IoError {
                kind: old_io::MismatchedFileTypeForOperation,
                desc: "Not a directory!",
                detail: Some(format!("{}", dir.display())),
            })
        }
    }
}

struct FilesBelow {
    stack: Vec<Arc<Path>>,
}

struct StatedPath {
    path: Arc<Path>,
    stat: FileStat,
}

impl Iterator for FilesBelow {
    type Item = IoResult<StatedPath>;

    fn next(&mut self) -> Option<IoResult<StatedPath>> {
        use std::old_io::fs;

        loop {
            let current = match self.stack.pop() {
                Some(path) => path,
                None       => return None,
            };

            let stat = match current.lstat() {
                Ok(stat) => stat,
                Err(err) => return Some(Err(err)),
            };

            match stat.kind {
                FileType::Directory => {
                    let dir_contents = match fs::readdir(& *current) {
                        Ok(contents) => contents,
                        Err(err)     => return Some(Err(err)),
                    };

                    let children = dir_contents.into_iter().map(|child| {
                        Arc::new(child)
                    });

                    self.stack.extend(children);
                    continue;
                },

                FileType::RegularFile => {
                    let stated_path = StatedPath {
                        path: current,
                        stat: stat,
                    };

                    return Some(Ok(stated_path));
                },

                _ => continue,
            }
        }
    }
}
