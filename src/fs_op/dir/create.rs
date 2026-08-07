use std::{fs, io, path::Path};

pub fn execute(path: &Path) -> io::Result<()> {
    println!("Creating dir: {}", path.display());
    fs::create_dir(path)
}

pub fn undo(path: &Path) -> io::Result<()> {
    println!("Removing created dir: {}", path.display());
    fs::remove_dir(path)
}
