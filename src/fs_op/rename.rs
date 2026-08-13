use std::{fs, io, path::Path};

use log::info;

pub fn execute(from: &Path, to: &Path) -> io::Result<()> {
    info!("Renaming: {} => {}", from.display(), to.display());
    fs::rename(from, to)
}

pub fn undo(from: &Path, to: &Path) -> io::Result<()> {
    info!("Renaming back: {} => {}", to.display(), from.display());
    fs::rename(to, from)
}
