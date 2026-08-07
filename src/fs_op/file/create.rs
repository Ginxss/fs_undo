use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

// TODO: visibility
pub fn execute(path: &Path, data: &[u8]) -> io::Result<()> {
    println!("Creating file: {}", path.display());
    let mut file = File::create_new(path)?;
    file.write_all(data)
}

pub fn undo(path: &Path) -> io::Result<()> {
    println!("Removing created file: {}", path.display());
    fs::remove_file(path)
}
