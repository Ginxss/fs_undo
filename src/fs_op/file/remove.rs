use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

/// Returns the contents of the removed file.
pub fn execute(path: &Path) -> io::Result<Vec<u8>> {
    println!("Removing file: {}", path.display());

    let data = fs::read(path)?;
    fs::remove_file(path)?;
    Ok(data)
}

pub fn undo(path: &Path, data: &Option<Vec<u8>>) -> io::Result<()> {
    println!("Recreating file: {}", path.display());

    let data = data
        .as_ref()
        .expect("data should have been filled during execute()");

    let mut file = File::create_new(path)?;
    file.write_all(data)
}
