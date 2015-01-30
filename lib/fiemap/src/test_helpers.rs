use std::rt::rtio::{self, RtioFileStream};
use native::old_io::file::{self, FileDesc};
use std::old_io::TempDir;
use std::os;
use std::rand::{self, Rng};

pub struct TestTempFile<'a> {
    name: &'a str,
    directory: DirectoryType,
    content: ContentType<'a>,
}

impl<'a> TestTempFile<'a> {
    pub fn new<'b>(name: &'b str) -> TestTempFile<'b> {
        TestTempFile {
            name: name,
            directory: CreateTempDirectory,
            content: DefaultContent,
        }
    }

    pub fn directory(mut self, path: Path) -> TestTempFile<'a> {
        self.directory = UseExistingDirectory(path);
        self
    }

    pub fn content(mut self, content: &'a str) -> TestTempFile<'a> {
        self.content = CustomContent(content);
        self
    }

    pub fn create(self) -> TestTempFileResult {
        let (tempdir, dir_path) = match self.directory {
            CreateTempDirectory => {
                let tempdir = TempDir::new_in(& os::getcwd(), "fiemap").unwrap();
                let path = tempdir.path().clone();

                (Some(tempdir), path)
            },

            UseExistingDirectory(path) => (None, path),
        };

        let path = dir_path.join(self.name);

        let mut rtio_file = file::open(
            & path.to_c_str(),
            rtio::Open,
            rtio::ReadWrite,
        ).ok().expect("Couldn't create test file");

        match self.content {
            DefaultContent => {
                let mut rng = rand::task_rng();
                let mut buffer = [0u8, ..(4096 * 4)];

                rng.fill_bytes(&mut buffer[]);

                rtio_file.write(buffer).ok().unwrap()
            },

            CustomContent(content) => {
                rtio_file.write(content.as_bytes()).ok().unwrap()
            }
        }

        TestTempFileResult {
            rtio_file: rtio_file,
            path: path,
            dir_path: dir_path,
            tempdir: tempdir,
        }
    }
}

pub struct TestTempFileResult {
    pub rtio_file: FileDesc,
    pub path: Path,
    pub dir_path: Path,

    tempdir: Option<TempDir>,
}

enum DirectoryType {
    CreateTempDirectory,
    UseExistingDirectory(Path),
}

enum ContentType<'a> {
    DefaultContent,
    CustomContent(&'a str),
}

pub fn create_tempfile() -> (TempDir, file::FileDesc) {
    let result = TestTempFile::new("foobar").create();
    (result.tempdir.unwrap(), result.rtio_file)
}
