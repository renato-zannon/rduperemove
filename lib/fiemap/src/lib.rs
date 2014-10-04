#![feature(phase)]

extern crate libc;
extern crate native;

#[phase(plugin, link)] extern crate ioctl;

#[cfg(not(ndebug))]
extern crate debug;

#[allow(non_camel_case_types)]
pub mod bindings;
