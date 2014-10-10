#![feature(macro_rules)]

extern crate libc;

use libc::c_int;

const IOC_WRITE: i32 = 1;
const IOC_READ:  i32 = 2;

const IOC_NRBITS:   uint = 8;
const IOC_TYPEBITS: uint = 8;
const IOC_SIZEBITS: uint = 14;

const IOC_NRSHIFT:   uint = 0u;
const IOC_TYPESHIFT: uint = IOC_NRSHIFT   + IOC_NRBITS;
const IOC_SIZESHIFT: uint = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT:  uint = IOC_SIZESHIFT + IOC_SIZEBITS;

#[inline]
pub fn ioc(dir: i32, ty: i32, nr: i32, size: i32) -> i32 {
    (dir  << IOC_DIRSHIFT)  |
    (ty   << IOC_TYPESHIFT) |
    (nr   << IOC_NRSHIFT)   |
    (size << IOC_SIZESHIFT)
}

#[inline]
pub fn iowr(magic: i32, nr: i32, size: uint) -> i32 {
    ioc(IOC_READ | IOC_WRITE, magic, nr, size as i32)
}

#[link(name = "c")]
extern "C" {
    pub fn ioctl(fd: c_int, command: c_int, ...) -> c_int;
}

#[macro_export]
macro_rules! ioctl(
    ($fd:expr, $command:expr, $($arg:expr),+) => (
        {
            let result = ioctl::ioctl($fd, $command, $($arg),+) as int;

            if result < 0 {
                Err(::std::io::IoError::last_error())
            } else {
                Ok(result)
            }
        }
    )
)
