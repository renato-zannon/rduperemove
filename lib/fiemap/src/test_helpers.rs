use std::rt::rtio::{mod, RtioFileStream};
use std::io::TempDir;
use std::os;
use native::io::file;

pub fn create_tempfile() -> (TempDir, file::FileDesc) {
    let tempdir = TempDir::new_in(& os::getcwd(), "fiemap")
        .ok()
        .expect("Couldn't create temp dir");

    let mut file = file::open(
        & tempdir.path().join("foo").to_c_str(),
        rtio::Open,
        rtio::ReadWrite,
    ).ok().expect("Couldn't create test file");

    for _ in range(0u, 100) {
        file.write(b"foo bar baz").ok().expect("Couldn't write to test file");
    }

    (tempdir, file)
}
