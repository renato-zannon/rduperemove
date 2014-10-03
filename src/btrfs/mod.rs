use native::io::file;
use std::rt::rtio;
use std::rt::rtio::RtioFileStream;
use std::sync::Arc;

#[allow(non_camel_case_types)]
mod ioctl;

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
    pub fn perform(self) -> uint {
        let mut source_fd = {
            let source = self.source.to_c_str();
            match file::open(&source, rtio::Open, rtio::Read) {
                Ok(fd)  => fd,
                Err(..) => fail!("Couldn't open file {} for reading", self.source.display()),
            }
        };

        let dest_count = self.destinations.len();
        let dest_fds = self.destinations.iter().filter_map(|dest_path| {
            let dest = dest_path.to_c_str();
            file::open(&dest, rtio::Open, rtio::ReadWrite).ok()
        }).collect::<Vec<file::FileDesc>>();

        let file_size = match source_fd.fstat() {
            Ok(stat) => stat.size,
            Err(..)  => fail!("Couldn't get source file ({}) size", self.source.display()),
        };

        if file_size < 4096 {
            return 0;
        }

        let mut same = ioctl::ExtentSame::new(dest_count);

        same.args().logical_offset = 0;
        same.args().length = file_size - (file_size % 4096);

        for (fd, info) in dest_fds.iter().zip(same.infos().iter_mut()) {
            info.fd = fd.fd() as i64;
            info.logical_offset = 0;
        }

        let mut total_dedup = 0u;

        loop {
            let res = unsafe {
                ioctl::btrfs_extent_same(source_fd.fd(), same.args())
            };

            if res != 0 || same.infos().iter().any(|info| info.status != 0) { break; }

            let offset = same.infos()[0].bytes_deduped;
            assert!(same.infos().tail().iter().all(|info| info.bytes_deduped == offset));

            total_dedup += (offset as uint) * dest_count;

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
