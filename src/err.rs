use std::{fs::FileType, io};

pub fn io_err_invalid_filetype<T>(filetype: FileType) -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("Invalid filetype {:?}", filetype),
    ))
}
