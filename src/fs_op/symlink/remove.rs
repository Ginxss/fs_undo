use std::{
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

use crate::err::io_err_invalid_input;

/// Returns the target of the removed link.
/// Fails if `path` is not a symlink.
pub fn execute(path: &Path) -> io::Result<PathBuf> {
    let target = fs::read_link(path)?;

    if !path.is_symlink() {
        let msg = &format!("RemoveSymlink path {} is not a symlink", path.display());
        return io_err_invalid_input(msg);
    }

    println!(
        "Removing symlink: {} => {}",
        path.display(),
        target.display()
    );

    fs::remove_file(path)?;
    Ok(target)
}

pub fn undo(path: &Path, target: &Option<PathBuf>) -> io::Result<()> {
    let target = target
        .as_ref()
        .expect("Should have been filled during execute()");

    println!(
        "Recreating link: {} => {}",
        path.display(),
        target.display()
    );

    unix::fs::symlink(target, path)
}
