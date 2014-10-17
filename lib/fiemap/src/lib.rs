#![feature(phase)]

extern crate libc;
extern crate native;

#[phase(plugin, link)] extern crate ioctl;

use native::io::file::FileDesc;
use bindings::FiemapRequest;

#[allow(non_camel_case_types)]
pub mod bindings;

#[cfg(test)]
mod test_helpers;

pub fn compare(file1: &FileDesc, file2: &FileDesc) -> ComparisonResult {
    let mut request1 = FiemapRequest::new(file1.fd()).unwrap();
    let mut request2 = FiemapRequest::new(file2.fd()).unwrap();

    let extents1 = request1.extents();
    let extents2 = request2.extents();

    let (inits_match, lasts_match) = {
        let (init1, last1) = if extents1.len() == 1 {
            (extents1.slice_to(1), None)
        } else {
            let last_index = extents1.len() - 1;
            (extents1.slice_to(last_index), Some(extents1[last_index]))
        };

        let (init2, last2) = if extents2.len() == 1 {
            (extents2.slice_to(1), None)
        } else {
            let last_index = extents2.len() - 1;
            (extents2.slice_to(last_index), Some(extents2[last_index]))
        };

        (init1 == init2, last1 == last2)
    };

    if inits_match && (lasts_match || extents1.len() == 1 && extents2.len() == 1)  {
        AlreadyDeduped
    } else if inits_match {
        PartiallyDeduped
    } else {
        NotDeduped
    }
}

#[deriving(Show, PartialEq, Eq)]
pub enum ComparisonResult {
    AlreadyDeduped,
    PartiallyDeduped,
    NotDeduped,
}

#[cfg(test)]
mod tests {
    use test_helpers::TestTempFile;
    use std::io::Command;
    use native::io::file::{mod, FileDesc};
    use std::rt::rtio;
    use super::{compare, AlreadyDeduped, NotDeduped};

    #[test]
    fn test_detects_different_files_with_same_content() {
        let content = "foo bar baz".repeat(1_000);

        let result1 = TestTempFile::new("file1").content(content.as_slice()).create();

        let result2 = TestTempFile::new("file2")
            .directory(result1.dir_path.clone())
            .content(content.as_slice())
            .create();

        sync();

        let result = compare(&result1.rtio_file, &result2.rtio_file);
        assert_eq!(result, NotDeduped);
    }

    #[test]
    fn test_detects_reflinked_file() {
        let tempfile_result = TestTempFile::new("original").create();
        let reflinked_path  = tempfile_result.dir_path.join("reflinked");

        reflink(&tempfile_result.path, &reflinked_path);
        sync();

        let reflinked_file = open_for_inspection(& reflinked_path);

        let result = compare(&tempfile_result.rtio_file, &reflinked_file);
        assert_eq!(result, AlreadyDeduped);
    }

    fn reflink(source: &Path, destination: &Path) {
        Command::new("cp")
            .arg("--reflink=always")
            .arg(source)
            .arg(destination)
            .spawn()
            .unwrap();
    }

    fn sync() {
        Command::new("sync").spawn().unwrap();
    }

    fn open_for_inspection(path: &Path) -> FileDesc {
        file::open(
            & path.to_c_str(),
            rtio::Open,
            rtio::ReadWrite,
        ).ok().expect("Couldn't open reflinked file")
    }
}
