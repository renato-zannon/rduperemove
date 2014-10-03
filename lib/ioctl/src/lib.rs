extern crate libc;

use libc::c_int;

static IOC_WRITE: i32 = 1;
static IOC_READ:  i32 = 2;

static IOC_NRBITS:   uint = 8;
static IOC_TYPEBITS: uint = 8;
static IOC_SIZEBITS: uint = 14;

static IOC_NRSHIFT:   uint = 0u;
static IOC_TYPESHIFT: uint = IOC_NRSHIFT   + IOC_NRBITS;
static IOC_SIZESHIFT: uint = IOC_TYPESHIFT + IOC_TYPEBITS;
static IOC_DIRSHIFT:  uint = IOC_SIZESHIFT + IOC_SIZEBITS;

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
