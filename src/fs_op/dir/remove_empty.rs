use std::{fs, io, path::Path};

pub fn execute(path: &Path) -> io::Result<()> {
    println!("Removing empty dir: {}", path.display());
    fs::remove_dir(path)
}

pub fn undo(path: &Path) -> io::Result<()> {
    println!("Recreating empty dir: {}", path.display());
    fs::create_dir(path)
}
