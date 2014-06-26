extern crate libc;
use std::mem;
use std::rt::heap;
use std::{raw, ptr};
use self::libc::c_int;

static IOC_WRITE: i32 = 1;
static IOC_READ:  i32 = 2;

static IOC_NRBITS:   i32 = 8;
static IOC_TYPEBITS: i32 = 8;
static IOC_SIZEBITS: i32 = 14;

static IOC_NRSHIFT:   i32 = 0;
static IOC_TYPESHIFT: i32 = IOC_NRSHIFT   + IOC_NRBITS;
static IOC_SIZESHIFT: i32 = IOC_TYPESHIFT + IOC_TYPEBITS;
static IOC_DIRSHIFT:  i32 = IOC_SIZESHIFT + IOC_SIZEBITS;

#[link(name = "c")]
extern "C" {
    fn ioctl(fd: c_int, command: c_int, ...) -> c_int;
}

static BTRFS_IOCTL_MAGIC: i32 = 0x94;

macro_rules! ioc(
    ($dir:expr, $ty:expr, $nr:expr, $size:expr) => {
        ($dir  << IOC_DIRSHIFT)  |
        ($ty   << IOC_TYPESHIFT) |
        ($nr   << IOC_NRSHIFT)   |
        ($size << IOC_SIZESHIFT)
    }
)

macro_rules! iowr(
    ($magic:expr, $nr:expr, $size:expr) => {
        ioc!(IOC_READ | IOC_WRITE, $magic, $nr, $size as i32)
    }
)

#[inline]
pub unsafe fn btrfs_extent_same(fd: c_int, same: &mut btrfs_ioctl_same_args) -> int {
    let btrfs_ioc_file_extent_same = iowr!(
        BTRFS_IOCTL_MAGIC,
        54,
        mem::size_of::<btrfs_ioctl_same_args>()
    );

    ioctl(fd as c_int, btrfs_ioc_file_extent_same as c_int, same) as int
}

#[repr(C)]
pub struct btrfs_ioctl_same_args {
    pub logical_offset: u64,  /* in - start of extent in source */
    pub length:         u64,  /* in - length of extent */
    pub dest_count:     u16,  /* in - total elements in info array */
    _reserved1:         u16,
    _reserved2:         u32,
    info:               [btrfs_ioctl_same_extent_info, ..0],
}

#[repr(C)]
pub struct btrfs_ioctl_same_extent_info {
    pub fd: i64,             /* in - destination file */
    pub logical_offset: u64, /* in - start of extent in destination */
    pub bytes_deduped:  u64, /* out - total # of bytes we were able to dedupe from this file */

    /* status of this dedupe operation:
     * 0 if dedup succeeds
     * < 0 for error
     * == BTRFS_SAME_DATA_DIFFERS if data differs
     */
    pub status: i32, /* out - see above description */
    pub reserved: u32,
}

pub struct ExtentSame<'a> {
    allocation: *mut u8,
    allocation_size: uint,

    pub args:  &'a mut btrfs_ioctl_same_args,
    pub infos: &'a mut [btrfs_ioctl_same_extent_info],
}

impl<'a> ExtentSame<'a> {
    pub fn new(info_count: uint) -> ExtentSame {
        let args_size  = ExtentSame::args_size();
        let infos_size = ExtentSame::infos_size(info_count);

        unsafe {
            let allocation_size = args_size + infos_size;
            let allocation = heap::allocate(
                allocation_size,
                mem::min_align_of::<btrfs_ioctl_same_args>(),
            );

            let args_ptr  = allocation as *mut btrfs_ioctl_same_args;
            let infos_ptr = allocation.offset(args_size as int) as *mut btrfs_ioctl_same_extent_info;

            ptr::write(args_ptr, btrfs_ioctl_same_args {
                logical_offset: 0,
                length:         0,
                dest_count:     info_count as u16,
                _reserved1:     0,
                _reserved2:     0,
                info:          [],
            });

            ptr::zero_memory(infos_ptr, info_count);

            ExtentSame {
                args:  mem::transmute(args_ptr),
                infos: mem::transmute(raw::Slice {
                    data: infos_ptr as *btrfs_ioctl_same_extent_info,
                    len: info_count
                }),

                allocation: allocation,
                allocation_size: allocation_size,
            }
        }
    }

    fn args_size() -> uint {
        mem::size_of::<btrfs_ioctl_same_args>()
    }

    fn infos_size(count: uint) -> uint {
        count * mem::size_of::<btrfs_ioctl_same_extent_info>()
    }
}

#[unsafe_destructor]
impl<'a> Drop for ExtentSame<'a> {
    fn drop(&mut self) {
        unsafe {
            heap::deallocate(
                self.allocation,
                self.allocation_size,
                mem::min_align_of::<btrfs_ioctl_same_args>(),
            );
        }
    }
}
