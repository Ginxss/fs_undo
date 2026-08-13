use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use log::info;

use crate::err::io_err_invalid_input;

/// Returns the contents of the removed file.
/// Fails if `path` is a symlink. Use `RemoveSymlink` instead.
pub fn execute(path: &Path) -> io::Result<Vec<u8>> {
    info!("Removing file: {}", path.display());

    if path.is_symlink() {
        let msg = &format!("RemoveFile path {} is a symlink", path.display());
        return io_err_invalid_input(msg);
    }

    let data = fs::read(path)?;
    fs::remove_file(path)?;
    Ok(data)
}

pub fn undo(path: &Path, data: &Option<Vec<u8>>) -> io::Result<()> {
    info!("Recreating file: {}", path.display());

    let data = data
        .as_ref()
        .expect("data should have been filled during execute()");

    let mut file = File::create_new(path)?;
    file.write_all(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{
        assert_file_exists_and_len, cleanup_test_path, init_test_path, random_bytes,
    };

    #[test]
    fn test_remove_file() {
        // arrange
        let base = init_test_path("test_remove_file");
        let path = base.join("file_to_remove.txt");
        let data = random_bytes();
        fs::write(&path, &data).unwrap();

        // act
        execute(&path).unwrap();

        // assert
        assert!(!path.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_file_undo() {
        // arrange
        let base = init_test_path("test_remove_file_undo");
        let path = base.join("file_to_remove.txt");
        let data = random_bytes();
        fs::write(&path, &data).unwrap();

        // act
        let overwritten_data = execute(&path).unwrap();
        undo(&path, &Some(overwritten_data)).unwrap();

        // assert
        assert_file_exists_and_len(&path, data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_file_twice() {
        // arrange
        let base = init_test_path("test_remove_file_twice");
        let path = base.join("file_to_remove.txt");
        let data = random_bytes();
        fs::write(&path, &data).unwrap();

        // act
        execute(&path).unwrap();
        let second_remove_res = execute(&path);

        // assert
        assert!(second_remove_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_file_undo_twice() {
        // arrange
        let base = init_test_path("test_remove_file_undo_twice");
        let path = base.join("file_to_remove.txt");
        let data = random_bytes();
        fs::write(&path, &data).unwrap();

        // act
        let overwritten_data = execute(&path).unwrap();
        undo(&path, &Some(overwritten_data.clone())).unwrap();
        let second_undo_res = undo(&path, &Some(overwritten_data));

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
