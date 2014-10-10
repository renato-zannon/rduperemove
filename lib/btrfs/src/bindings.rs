use std::mem;
use std::rt::heap;
use std::{raw, ptr};
use libc::c_int;
use std::io::IoResult;
use ioctl;

const BTRFS_IOCTL_MAGIC: i32 = 0x94;

#[inline]
pub unsafe fn btrfs_extent_same(fd: c_int, same: &mut btrfs_ioctl_same_args) -> IoResult<int> {
    let btrfs_ioc_file_extent_same = ioctl::iowr(
        BTRFS_IOCTL_MAGIC,
        54,
        ExtentSame::args_size()
    );

    ioctl!(fd as c_int, btrfs_ioc_file_extent_same as c_int, same)
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

pub struct ExtentSame {
    allocation: *mut u8,
}

impl ExtentSame {
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

            ExtentSame { allocation: allocation }
        }
    }

    pub fn args(&mut self) -> &mut btrfs_ioctl_same_args {
        unsafe { mem::transmute(self.allocation) }
    }

    pub fn infos(&mut self) -> &mut [btrfs_ioctl_same_extent_info] {
        unsafe {
            let args_size = ExtentSame::args_size();
            let infos_ptr = self.allocation.offset(args_size as int);
            let info_count = self.args().dest_count as uint;

            mem::transmute(raw::Slice {
                data: infos_ptr as *const btrfs_ioctl_same_extent_info,
                len: info_count
            })
        }
    }

    fn allocation_size(&mut self) -> uint {
        ExtentSame::args_size() +
            ExtentSame::infos_size(self.args().dest_count as uint)
    }

    fn args_size() -> uint {
        mem::size_of::<btrfs_ioctl_same_args>()
    }

    fn infos_size(count: uint) -> uint {
        count * mem::size_of::<btrfs_ioctl_same_extent_info>()
    }
}

impl Drop for ExtentSame {
    fn drop(&mut self) {
        unsafe {
            heap::deallocate(
                self.allocation,
                self.allocation_size(),
                mem::min_align_of::<btrfs_ioctl_same_args>(),
            );
        }
    }
}
