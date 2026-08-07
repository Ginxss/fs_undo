use std::{
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

/// Returns the target of the removed link.
// TODO: fail if not symlink?
pub fn execute(path: &Path) -> io::Result<PathBuf> {
    let target = fs::read_link(path)?;

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
