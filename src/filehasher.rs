use crypto::digest::Digest;
use crypto::md5::Md5;

use std::io::{File, IoError, EndOfFile};
use std::iter;

pub struct FileHasher {
    buffer: Vec<u8>,
    hasher: Md5,
}

impl FileHasher {
    pub fn hash_whole_file(&mut self, mut file: File) -> Vec<u8> {
        self.hasher.reset();

        loop {
            match file.read(&mut self.buffer[]) {
                Ok(count) => self.hasher.input(&self.buffer[..count]),

                Err(IoError { kind: EndOfFile, ..}) => break,
                Err(err) => panic!("Error while hashing file: {}", err),
            }
        }

        let block_size = self.hasher.block_size();

        let mut result: Vec<_> = iter::repeat(0u8).take(block_size).collect();
        self.hasher.result(&mut result[]);

        result
    }
}

pub fn new(buffer_size: usize) -> FileHasher {
    let mut buffer = Vec::with_capacity(buffer_size);
    unsafe { buffer.set_len(buffer_size) }

    FileHasher {
        buffer: buffer,
        hasher: Md5::new(),
    }
}
