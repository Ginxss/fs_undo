pub mod fs_op;
pub mod undo;

mod err;

#[cfg(test)]
mod test_util;

use std::{io, path::Path};

use crate::{err::io_err_invalid_filetype, fs_op::FsOp};

pub fn delete(path: impl AsRef<Path>) -> io::Result<FsOp> {
    let path = path.as_ref();

    let mut op = if path.is_file() {
        FsOp::remove_file(path)
    } else if path.is_symlink() {
        FsOp::remove_symlink(path)
    } else if path.is_dir() {
        FsOp::remove_dir_all(path)
    } else {
        return io_err_invalid_filetype(path.metadata()?.file_type());
    };

    op.execute().map(|()| op)
}
