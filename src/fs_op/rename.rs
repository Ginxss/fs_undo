use std::{fs, io, path::Path};

pub fn execute(from: &Path, to: &Path) -> io::Result<()> {
    println!("Renaming: {} => {}", from.display(), to.display());
    fs::rename(from, to)
}

pub fn undo(from: &Path, to: &Path) -> io::Result<()> {
    println!("Renaming back: {} => {}", to.display(), from.display());
    fs::rename(to, from)
}
