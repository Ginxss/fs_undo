use std::{fs, io, path::Path};

use crate::{err::io_err_invalid_filetype, fs_op::FsOp, undo::Undo};

/// Tries to be as atomic as possible:
/// If there is an error while removing, tries to undo all previous operations before returning the error.
/// Only returns Ok(()) if all delete operations were successful.
///
/// Returns all delete operations in the order they were executed.
pub fn execute(path: &Path) -> io::Result<Vec<FsOp>> {
    println!("Removing dir including contents: {}", path.display());

    if path.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("RemoveDirAll input path {} is a symlink", path.display()),
        ));
    }

    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("RemoveDirAll input path {} is not a dir", path.display()),
        ));
    }

    let mut ops = Vec::new();
    match remove_dir_recursive(path, &mut ops) {
        Ok(()) => Ok(ops),
        Err(err) => {
            ops.undo()?;
            Err(err)
        }
    }
}

fn remove_dir_recursive(path: impl AsRef<Path>, fs_ops: &mut Vec<FsOp>) -> io::Result<()> {
    for entry in fs::read_dir(&path)? {
        let entry = entry?;

        let filetype = entry.file_type()?;
        let entry_path = entry.path();

        if filetype.is_file() {
            let mut op = FsOp::remove_file(&entry_path);
            op.execute()?;
            fs_ops.push(op);
        } else if filetype.is_symlink() {
            let mut op = FsOp::remove_symlink(&entry_path);
            op.execute()?;
            fs_ops.push(op);
        } else if filetype.is_dir() {
            remove_dir_recursive(entry_path, fs_ops)?;
        } else {
            return io_err_invalid_filetype(filetype);
        }
    }

    let mut op = FsOp::remove_empty_dir(path);
    op.execute()?;
    fs_ops.push(op);

    Ok(())
}

pub fn undo(path: &Path, ops: &Vec<FsOp>) -> io::Result<()> {
    println!("Recreating dir including contents: {}", path.display());
    ops.undo()
}
