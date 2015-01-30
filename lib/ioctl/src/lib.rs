#![feature(libc)]

extern crate libc;

use libc::c_int;

const IOC_WRITE: i32 = 1;
const IOC_READ:  i32 = 2;

const IOC_NRBITS:   usize = 8;
const IOC_TYPEBITS: usize = 8;
const IOC_SIZEBITS: usize = 14;

const IOC_NRSHIFT:   usize = 0us;
const IOC_TYPESHIFT: usize = IOC_NRSHIFT   + IOC_NRBITS;
const IOC_SIZESHIFT: usize = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT:  usize = IOC_SIZESHIFT + IOC_SIZEBITS;

#[inline]
pub fn ioc(dir: i32, ty: i32, nr: i32, size: i32) -> i32 {
    (dir  << IOC_DIRSHIFT)  |
    (ty   << IOC_TYPESHIFT) |
    (nr   << IOC_NRSHIFT)   |
    (size << IOC_SIZESHIFT)
}

#[inline]
pub fn iowr(magic: i32, nr: i32, size: usize) -> i32 {
    ioc(IOC_READ | IOC_WRITE, magic, nr, size as i32)
}

#[link(name = "c")]
extern "C" {
    pub fn ioctl(fd: c_int, command: c_int, ...) -> c_int;
}

#[macro_export]
macro_rules! ioctl {
    ($fd:expr, $command:expr, $($arg:expr),+) => (
        {
            let result = ioctl::ioctl($fd, $command, $($arg),+) as isize;

            if result < 0 {
                Err(::std::old_io::IoError::last_error())
            } else {
                Ok(result)
            }
        }
    )
}
