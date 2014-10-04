#![feature(macro_rules, concat_idents)]

extern crate libc;
extern crate ioctl;
extern crate native;

#[cfg(not(ndebug))]
extern crate debug;

#[allow(non_camel_case_types)]
pub mod bindings;
