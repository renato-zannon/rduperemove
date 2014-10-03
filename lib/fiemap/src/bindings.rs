use ioctl;
use std::mem;
use libc::c_int;

static FIEMAP_IOCTL_MAGIC: i32 = 'f' as i32;

pub unsafe fn fiemap_ioctl(fd: c_int, map: &mut fiemap) -> int {
    let fiemap_command = ioctl::iowr(
        FIEMAP_IOCTL_MAGIC,
        11,
        mem::size_of::<fiemap>()
    );

    ioctl::ioctl(fd, fiemap_command as c_int, map) as int
}

#[repr(C)]
pub struct fiemap_extent {
    /* logical offset in bytes for the start of the extent from the beginning of the file */
    fe_logical: u64,

    /* physical offset in bytes for the start of the extent from the beginning of the disk */
    fe_physical: u64,

    /* length in bytes for this extent */
    fe_length: u64,
    fe_reserved64: [u64, ..2],

    /* FIEMAP_EXTENT_* flags for this extent */
    fe_flags: u32,
    fe_reserved: [u32, ..3],
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
    pub fm_extents: [fiemap_extent, ..0],
}

pub mod flags {
    bitflags!(
        flags Flags: u32 {
            /* sync file data before map */
            static Sync = 0x00000001,

            /* map extended attribute tree */
            static Xattr = 0x00000002,

            /* request caching of the extents */
            static Cache = 0x00000004,

            static Compat = Sync.bits | Xattr.bits
        }
    )
}

pub mod extent_flags {
    bitflags!(
        flags ExtentFlags: u32 {
            /* Last extent in file. */
            static Last = 0x00000001,

            /* Data location unknown. */
            static Unknown = 0x00000002,

            /* Location still pending. Sets EXTENT_UNKNOWN. */
            static Delalloc = 0x00000004,

            /* Data can not be read while fs is unmounted */
            static Encoded = 0x00000008,

            /* Data is encrypted by fs. Sets EXTENT_NO_BYPASS. */
            static DataEncrypted = 0x00000080,

            /* Extent offsets may not be block aligned. */
            static NotAligned = 0x00000100,

            /* Data mixed with metadata. Sets EXTENT_NOT_ALIGNED.*/
            static DataInline = 0x00000200,

            /* Multiple files in block. Sets EXTENT_NOT_ALIGNED.*/
            static DataTail = 0x00000400,

            /* Space allocated, but no data (i.e. zero). */
            static Unwritten = 0x00000800,

            /* File does not natively support extents. Result merged for efficiency. */
            static Merged = 0x00001000,

            /* Space shared with other files. */
            static Shared = 0x00002000
        }
    )
}
