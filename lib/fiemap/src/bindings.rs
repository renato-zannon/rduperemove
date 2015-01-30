use ioctl;
use std::{raw, mem, ptr, u64};
use std::rt::heap;
use libc::c_int;
use std::old_io::IoResult;

pub use self::flags::Flags as FiemapFlags;
pub use self::extent_flags::ExtentFlags;

const FIEMAP_IOCTL_MAGIC: i32 = 'f' as i32;

pub unsafe fn fiemap_ioctl(fd: c_int, map: &mut fiemap) -> IoResult<int> {
    let fiemap_command = ioctl::iowr(
        FIEMAP_IOCTL_MAGIC,
        11,
        mem::size_of::<fiemap>()
    );

    ioctl!(fd, fiemap_command as c_int, map)
}

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct fiemap_extent {
    /* logical offset in bytes for the start of the extent from the beginning of the file */
    fe_logical: u64,

    /* physical offset in bytes for the start of the extent from the beginning of the disk */
    fe_physical: u64,

    /* length in bytes for this extent */
    fe_length: u64,
    fe_reserved64: [u64; 2],

    /* FIEMAP_EXTENT_* flags for this extent */
    fe_flags: u32,
    fe_reserved: [u32; 3],
}

impl fiemap_extent {
    pub fn flags(&self) -> ExtentFlags {
        ExtentFlags::from_bits_truncate(self.fe_flags)
    }
}

impl PartialEq for fiemap_extent {
    fn eq(&self, other: &fiemap_extent) -> bool {
        self.fe_logical == other.fe_logical &&
            self.fe_physical == other.fe_physical &&
            self.fe_length   == other.fe_length &&
            self.fe_flags    == other.fe_flags
    }
}

#[repr(C)]
pub struct fiemap {
    /* logical offset (inclusive) at which to start mapping (in) */
    pub fm_start: u64,

    /* logical length of mapping which userspace wants (in) */
    pub fm_length: u64,

    /* FIEMAP_FLAG_* flags for request (in/out) */
    pub fm_flags: u32,

    /* number of extents that were mapped (out) */
    pub fm_mapped_extents: u32,

    /* size of fm_extents array (in) */
    pub fm_extent_count: u32,
    pub fm_reserved: u32,

    /* array of mapped extents (out) */
    pub fm_extents: [fiemap_extent; 0],
}

impl fiemap {
    pub fn flags(&self) -> FiemapFlags {
        FiemapFlags::from_bits_truncate(self.fm_flags)
    }
}

pub mod flags {
    bitflags!(
        flags Flags: u32 {
            /* sync file data before map */
            const SYNC = 0x00000001,

            /* map extended attribute tree */
            const XATTR = 0x00000002,

            /* request caching of the extents */
            const CACHE = 0x00000004,

            const COMPAT = SYNC.bits | XATTR.bits
        }
    );
}

pub mod extent_flags {
    bitflags!(
        flags ExtentFlags: u32 {
            /* Last extent in file. */
            const LAST = 0x00000001,

            /* Data location unknown. */
            const UNKNOWN = 0x00000002,

            /* Location still pending. Sets EXTENT_UNKNOWN. */
            const DELALLOC = 0x00000004,

            /* Data can not be read while fs is unmounted */
            const ENCODED = 0x00000008,

            /* Data is encrypted by fs. Sets EXTENT_NO_BYPASS. */
            const DATA_ENCRYPTED = 0x00000080,

            /* Extent offsets may not be block aligned. */
            const NOT_ALIGNED = 0x00000100,

            /* Data mixed with metadata. Sets EXTENT_NOT_ALIGNED.*/
            const DATA_INLINE = 0x00000200,

            /* Multiple files in block. Sets EXTENT_NOT_ALIGNED.*/
            const DATA_TAIL = 0x00000400,

            /* Space allocated, but no data (i.e. zero). */
            const UNWRITTEN = 0x00000800,

            /* File does not natively support extents. Result merged for efficiency. */
            const MERGED = 0x00001000,

            /* Space shared with other files. */
            const SHARED = 0x00002000
        }
    );
}

pub struct FiemapRequest {
    allocation: *mut u8,
    allocation_size: uint,
}

impl FiemapRequest {
    pub fn new(fd: c_int) -> IoResult<FiemapRequest> {
        unsafe {
            // Allocate (and zero) an initial fiemap struct
            let mut alloc = heap::allocate(mem::size_of::<fiemap>(), mem::min_align_of::<fiemap>());
            let mut map: &mut fiemap = mem::transmute(alloc);
            ptr::zero_memory(map as *mut fiemap, 1);

            // We want all extents
            map.fm_length = u64::MAX;

            // Ask the FS how many extents there are
            try!(fiemap_ioctl(fd, map));

            let extent_count = map.fm_mapped_extents as uint;

            let alloc_size = mem::size_of::<fiemap>() +
                extent_count * mem::size_of::<fiemap_extent>();

            // Extend the allocation to accomodate the extents
            alloc = heap::reallocate(
                alloc,
                mem::size_of::<fiemap>(),
                alloc_size,
                mem::min_align_of::<fiemap>(),
            );

            // This may have changed after the reallocation
            map = mem::transmute(alloc);
            map.fm_extent_count = extent_count as u32;
            map.fm_mapped_extents = 0;

            let extents_ptr = alloc.offset(mem::size_of::<fiemap>() as int) as *mut fiemap_extent;
            ptr::zero_memory(extents_ptr, extent_count);

            try!(fiemap_ioctl(fd, map));

            Ok(FiemapRequest {
                allocation: alloc,
                allocation_size: alloc_size,
            })
        }
    }

    pub fn fiemap(&mut self) -> &mut fiemap {
        unsafe { mem::transmute(self.allocation) }
    }

    pub fn extents(&mut self) -> &mut [fiemap_extent] {
        unsafe {
            let extents_ptr = self.allocation.offset(mem::size_of::<fiemap>() as int);
            let count = self.fiemap().fm_extent_count as uint;

            mem::transmute(raw::Slice {
                data: extents_ptr as *const fiemap_extent,
                len:  count,
            })
        }
    }
}

impl Drop for FiemapRequest {
    fn drop(&mut self) {
        unsafe {
            heap::deallocate(
                self.allocation,
                self.allocation_size,
                mem::min_align_of::<fiemap>(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use test_helpers;
    use super::FiemapRequest;

    #[test]
    fn test_create_fiemap_request() {
        let (_tempdir, file) = test_helpers::create_tempfile();
        let _request = FiemapRequest::new(file.fd()).unwrap();
    }

    #[test]
    fn test_drop_fiemap_request() {
        let (_tempdir, file) = test_helpers::create_tempfile();
        drop(FiemapRequest::new(file.fd()).unwrap());
    }
}
