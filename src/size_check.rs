use std::collections::{HashMap, PriorityQueue};
use std::io::{TypeFile, IoResult, IoError};
use std::{vec, io, iter};

pub struct SizeCheck {
    min_size: uint,
    groups:   HashMap<uint, Vec<Path>>
}

pub fn new_check(min_size: uint) -> SizeCheck {
    SizeCheck { groups: HashMap::new(), min_size: min_size }
}

impl SizeCheck {
    #[must_use]
    pub fn add_base_dir(&mut self, dir: &Path, on_err: |IoError|) -> IoResult<()> {
        for file in try!(recurse_directory(dir)) {
            match file {
                Ok(SizedFile { path: path, size: size }) => {
                    if size < self.min_size { continue; }

                    let paths = self.groups.find_or_insert_with(size, |_| {
                        Vec::new()
                    });

                    paths.push(path);
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
            .collect::<PriorityQueue<uint>>()
            .into_sorted_vec();

        SizeGroups {
            sorted_sizes_iter: sizes.move_iter().rev(),
            size_groups: self.groups,
        }
    }
}

pub struct SizeGroups {
    sorted_sizes_iter: iter::Rev<vec::MoveItems<uint>>,
    size_groups: HashMap<uint, Vec<Path>>
}

impl<'a> Iterator<Vec<Path>> for SizeGroups {
    fn next(&mut self) -> Option<Vec<Path>> {
        for size in self.sorted_sizes_iter {
            let paths = self.size_groups.pop(&size).unwrap();

            if paths.len() > 1 {
                return Some(paths);
            }
        }

        None
    }
}

fn recurse_directory(dir: &Path) -> IoResult<FilesBelow> {
    let stat = try!(dir.stat());

    match stat.kind {
        io::TypeDirectory => Ok(FilesBelow { stack: vec!(dir.clone()) }),
        _  => {
            Err(IoError {
                kind: io::MismatchedFileTypeForOperation,
                desc: "Not a directory!",
                detail: Some(format!("{}", dir.display())),
            })
        }
    }
}

struct FilesBelow {
    stack: Vec<Path>,
}

struct SizedFile {
    path: Path,
    size: uint,
}

impl<'a> Iterator<IoResult<SizedFile>> for FilesBelow {
    fn next(&mut self) -> Option<IoResult<SizedFile>> {
        use std::io::fs;

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
                io::TypeDirectory => {
                    let dir_contents = match fs::readdir(&current) {
                        Ok(contents) => contents,
                        Err(err)     => return Some(Err(err)),
                    };

                    self.stack.push_all_move(dir_contents);
                    continue;
                },

                io::TypeFile => {
                    return Some(Ok(SizedFile { path: current, size: stat.size as uint }));
                },

                _ => continue,
            }
        }
    }
}
