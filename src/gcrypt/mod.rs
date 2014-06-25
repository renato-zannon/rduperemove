extern crate libc;
use self::libc::{size_t, c_void};

use std::path::BytesContainer;
pub use self::bindings::{init, algos, GcryptMdAlgo};

#[allow(non_camel_case_types, dead_code)]
mod bindings;

pub struct Hash {
    handle: bindings::gcrypt_md_handle,
    algo: GcryptMdAlgo,
}

impl Hash {
    pub fn new(algo: GcryptMdAlgo) -> Hash {
        use std::ptr;

        unsafe {
            let mut handle = ptr::mut_null();
            let handle_ptr = (&mut handle) as *mut bindings::gcrypt_md_handle;
            bindings::gcry_md_open(handle_ptr, algo, 0);

            Hash { handle: handle, algo: algo }
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        unsafe {
            bindings::gcry_md_write(
                self.handle,
                bytes.as_ptr() as *c_void,
                bytes.len() as size_t
            );
        }
    }

    pub fn size(&self) -> uint {
        unsafe {
            bindings::gcry_md_get_algo_dlen(self.algo) as uint
        }
    }

    pub fn read<U>(self, f: |&[u8]| -> U) -> U {
        use std::slice::raw::buf_as_slice;

        let size = self.size();

        unsafe {
            let buf = bindings::gcry_md_read(self.handle, self.algo);
            buf_as_slice(buf, size, f)
        }
    }

    pub fn read_hex(self) -> String {
        self.read(|result| bytes_to_hex(result))
    }
}

impl Drop for Hash {
    fn drop(&mut self) {
        unsafe {
            bindings::gcry_md_close(self.handle)
        }
    }
}

pub fn digest<C: BytesContainer>(message: C, algo: GcryptMdAlgo) -> Vec<u8> {
    let bytes     = message.container_as_bytes();
    let bytes_ptr = bytes.as_ptr() as *c_void;

    unsafe {
        let size       = bindings::gcry_md_get_algo_dlen(algo);
        let mut buffer = Vec::with_capacity(size as uint);

        bindings::gcry_md_hash_buffer(
            algo,
            buffer.as_mut_ptr() as *mut c_void,
            bytes_ptr,
            bytes.len() as size_t
        );

        buffer
    }
}

#[inline]
pub fn hex_digest<C: BytesContainer>(message: C, algo: GcryptMdAlgo) -> String {
    bytes_to_hex(digest(message, algo).as_slice())
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::{str, u8};

    let mut string = String::with_capacity(bytes.len() * 2);

    for &number in bytes.iter() {
        u8::to_str_bytes(number, 16, |bytes| {
            let slice = str::from_utf8(bytes).unwrap();

            if slice.len() == 1 {
                string.push_str("0");
            }

            string.push_str(slice);
        });
    }

    string
}
