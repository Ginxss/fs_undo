use std::{fs, io, os::unix, path::Path};

pub fn execute(path: &Path, target: &Path) -> io::Result<()> {
    println!("Creating link: {} => {}", path.display(), target.display());

    // TODO
    if !target.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Link target {} does not exist", target.display()),
        ));
    }

    unix::fs::symlink(target, path)
}

pub fn undo(path: &Path, target: &Path) -> io::Result<()> {
    println!(
        "Removing created link: {} => {}",
        path.display(),
        target.display()
    );

    fs::remove_file(path)
}
