extern crate libc;

#[macro_use]
extern crate ioctl;
#[macro_use]
extern crate log;

use std::old_io::{File, FileMode, FileAccess};
use std::sync::Arc;
use std::os::unix::prelude::*;

#[allow(non_camel_case_types)]
mod bindings;

pub struct Dedup<'a> {
    source: Arc<Path>,
    destinations: &'a [Arc<Path>]
}

pub fn new_dedup<'a>(source: Arc<Path>, destinations: &'a [Arc<Path>]) -> Dedup<'a> {
    Dedup {
        source: source,
        destinations: destinations
    }
}

impl<'a> Dedup<'a> {
    pub fn perform(self) -> usize {
        let source_file = {
            match File::open(&*self.source) {
                Ok(file) => file,
                Err(..)  => panic!("Couldn't open file {} for reading", self.source.display()),
            }
        };

        let dest_count = self.destinations.len();
        let dest_files = self.destinations.iter().filter_map(|dest_path| {
            File::open_mode(&**dest_path, FileMode::Open, FileAccess::ReadWrite).ok()
        }).collect::<Vec<_>>();

        let file_size = match source_file.stat() {
            Ok(stat) => stat.size,
            Err(..)  => panic!("Couldn't get source file ({}) size", self.source.display()),
        };

        if file_size < 4096 {
            return 0;
        }

        let mut same = bindings::ExtentSame::new(dest_count);

        same.args().logical_offset = 0;
        same.args().length = file_size - (file_size % 4096);

        for (file, info) in dest_files.iter().zip(same.infos().iter_mut()) {
            info.fd = file.as_raw_fd() as i64;
            info.logical_offset = 0;
        }

        let mut total_dedup = 0us;

        loop {
            let errored = unsafe {
                let result = bindings::btrfs_extent_same(source_file.as_raw_fd(), same.args());

                match result {
                    Err(err) => { warn!("Error: {}", err); true },
                    Ok(_)  => {
                        same.infos().iter().any(|info| info.status != 0)
                    }
                }
            };

            if errored { break; }

            let offset = same.infos()[0].bytes_deduped;
            assert!(same.infos().tail().iter().all(|info| info.bytes_deduped == offset));

            total_dedup += (offset as usize) * dest_count;

            if same.args().length < offset || offset == 0 { break; }

            same.args().logical_offset += offset;
            same.args().length -= offset;
            if same.args().length < 1 { break; }

            for info in same.infos().iter_mut() {
                info.logical_offset += offset;
            }
        }

        total_dedup
    }
}
