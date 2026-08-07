use std::{fs, io, path::Path};

/// Returns overwritten data.
pub fn write(path: &Path, data: &Vec<u8>) -> io::Result<Option<Vec<u8>>> {
    println!("Writing file: {}", path.display());

    let prev_data = path.exists().then(|| fs::read(path)).transpose()?;
    fs::write(path, data)?;
    Ok(prev_data)
}

/// Fails if `to === from`.
/// Returns overwritten data.
pub fn copy(from: &Path, to: &Path) -> io::Result<Option<Vec<u8>>> {
    println!("Copying file: {} => {}", from.display(), to.display());

    if to == from {
        // TODO: better?
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Copy target {} is the same as the source", to.display()),
        ));
    }

    let prev_data = to.exists().then(|| fs::read(to)).transpose()?;
    fs::copy(from, to)?;
    Ok(prev_data)
}

pub fn undo(written_path: &Path, prev_data: &Option<Vec<u8>>) -> io::Result<()> {
    match prev_data {
        Some(prev_data) => {
            println!("Restoring overwritten file: {}", written_path.display());
            fs::write(written_path, prev_data)
        }
        None => {
            println!("Removing created file: {}", written_path.display());
            fs::remove_file(written_path)
        }
    }
}
