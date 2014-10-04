use crypto::digest::Digest;
use crypto::md5::Md5;

use std::io::{File, IoError, EndOfFile};

pub struct FileHasher {
    buffer: Vec<u8>,
    hasher: Md5,
}

impl FileHasher {
    pub fn hash_whole_file(&mut self, mut file: File) -> Vec<u8> {
        self.hasher.reset();

        loop {
            match file.read(self.buffer.as_mut_slice()) {
                Ok(count) => self.hasher.input(self.buffer.slice_to(count)),

                Err(IoError { kind: EndOfFile, ..}) => break,
                Err(err) => fail!("Error while hashing file: {}", err),
            }
        }

        let mut result = Vec::from_elem(self.hasher.block_size(), 0u8);
        self.hasher.result(result.as_mut_slice());

        result
    }
}

pub fn new(buffer_size: uint) -> FileHasher {
    let mut buffer = Vec::with_capacity(buffer_size);
    unsafe { buffer.set_len(buffer_size) }

    FileHasher {
        buffer: buffer,
        hasher: Md5::new(),
    }
}
