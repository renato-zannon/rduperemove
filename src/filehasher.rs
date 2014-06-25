use gcrypt;
use gcrypt::Hash;
use std::io::{File, IoError, EndOfFile};

pub struct FileHasher {
    buffer: Vec<u8>,
}

impl FileHasher {
    pub fn hash_whole_file(&mut self, mut file: File) -> Vec<u8> {
        let mut hash = Hash::new(gcrypt::algos::MD5);

        loop {
            match file.read(self.buffer.as_mut_slice()) {
                Ok(count) => hash.write(self.buffer.slice_to(count)),

                Err(IoError { kind: EndOfFile, ..}) => break,
                Err(err) => fail!("Error while hashing file: {}", err),
            }
        }

        hash.read(|result| Vec::from_slice(result))
    }
}

pub fn new(buffer_size: uint) -> FileHasher {
    let mut buffer = Vec::with_capacity(buffer_size);
    unsafe { buffer.set_len(buffer_size) }

    FileHasher { buffer: buffer }
}
