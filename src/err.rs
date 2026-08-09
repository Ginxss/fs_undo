use std::{fs::FileType, io};

pub fn io_err_invalid_input<T>(msg: &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidInput, msg))
}

pub fn io_err_invalid_filetype<T>(filetype: FileType) -> io::Result<T> {
    io_err_invalid_input(&format!("Invalid filetype {:?}", filetype))
}
