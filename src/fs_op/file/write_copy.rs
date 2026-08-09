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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{
        assert_exists_and_len, cleanup_test_path, init_test_path, random_bytes,
    };

    #[test]
    fn test_write_new_file() {
        // arrange
        let base = init_test_path("test_write_new_file");
        let path = base.join("written_file.txt");
        let data = random_bytes();

        // act
        write(&path, &data).unwrap();

        // assert
        assert_exists_and_len(&path, data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_write_new_file_undo() {
        // arrange
        let base = init_test_path("test_write_new_file_undo");
        let path = base.join("written_file.txt");
        let data = random_bytes();

        // act
        let overwritten_data = write(&path, &data).unwrap();
        undo(&path, &overwritten_data).unwrap();

        // assert
        assert!(!path.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_overwrite_file() {
        // arrange
        let base = init_test_path("test_overwrite_file");
        let path = base.join("written_file.txt");
        let new_data = random_bytes();
        let prev_data = random_bytes();
        fs::write(&path, &prev_data).unwrap();

        // act
        write(&path, &new_data).unwrap();

        // assert
        assert_exists_and_len(&path, new_data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_overwrite_file_undo() {
        // arrange
        let base = init_test_path("test_overwrite_file_undo");
        let path = base.join("written_file.txt");
        let new_data = random_bytes();
        let prev_data = random_bytes();
        fs::write(&path, &prev_data).unwrap();

        // act
        let overwritten_data = write(&path, &new_data).unwrap();
        undo(&path, &overwritten_data).unwrap();

        // assert
        assert_exists_and_len(&path, prev_data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_write_file_twice() {
        // arrange
        let base = init_test_path("test_write_file_twice");
        let path = base.join("written_file.txt");
        let data1 = random_bytes();
        let data2 = random_bytes();

        // act
        write(&path, &data1).unwrap();
        write(&path, &data2).unwrap();

        // assert
        assert_exists_and_len(&path, data2.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_write_new_file_undo_twice() {
        // arrange
        let base = init_test_path("test_write_new_file_undo_twice");
        let path = base.join("written_file.txt");
        let data = random_bytes();

        // act
        let overwritten_data = write(&path, &data).unwrap();
        undo(&path, &overwritten_data).unwrap();
        let second_undo_res = undo(&path, &overwritten_data);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_copy_new_file() {
        // arrange
        let base = init_test_path("test_copy_new_file");
        let from = base.join("from.txt");
        let to = base.join("to.txt");
        let data = random_bytes();
        fs::write(&from, &data).unwrap();

        // act
        copy(&from, &to).unwrap();

        // assert
        assert_exists_and_len(&from, data.len());
        assert_exists_and_len(&to, data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_copy_new_file_undo() {
        // arrange
        let base = init_test_path("test_copy_new_file_undo");
        let from = base.join("from.txt");
        let to = base.join("to.txt");
        let data = random_bytes();
        fs::write(&from, &data).unwrap();

        // act
        let overwritten_data = copy(&from, &to).unwrap();
        undo(&to, &overwritten_data).unwrap();

        // assert
        assert_exists_and_len(&from, data.len());
        assert!(!to.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_copy_over_file() {
        // arrange
        let base = init_test_path("test_copy_over_file");
        let from = base.join("from.txt");
        let to = base.join("to.txt");
        let from_data = random_bytes();
        fs::write(&from, &from_data).unwrap();
        let prev_to_data = random_bytes();
        fs::write(&to, &prev_to_data).unwrap();

        // act
        copy(&from, &to).unwrap();

        // assert
        assert_exists_and_len(&from, from_data.len());
        assert_exists_and_len(&to, from_data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_copy_over_file_undo() {
        // arrange
        let base = init_test_path("test_copy_over_file_undo");
        let from = base.join("from.txt");
        let to = base.join("to.txt");
        let from_data = random_bytes();
        fs::write(&from, &from_data).unwrap();
        let prev_to_data = random_bytes();
        fs::write(&to, &prev_to_data).unwrap();

        // act
        let overwritten_data = copy(&from, &to).unwrap();
        undo(&to, &overwritten_data).unwrap();

        // assert
        assert_exists_and_len(&from, from_data.len());
        assert_exists_and_len(&to, prev_to_data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_copy_file_twice() {
        // arrange
        let base = init_test_path("test_copy_file_twice");
        let from = base.join("from.txt");
        let to = base.join("to.txt");
        let data = random_bytes();
        fs::write(&from, &data).unwrap();

        // act
        copy(&from, &to).unwrap();
        copy(&from, &to).unwrap();

        // assert
        assert_exists_and_len(&from, data.len());
        assert_exists_and_len(&to, data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_copy_new_file_undo_twice() {
        // arrange
        let base = init_test_path("test_copy_new_file_undo_twice");
        let from = base.join("from.txt");
        let to = base.join("to.txt");
        let data = random_bytes();
        fs::write(&from, &data).unwrap();

        // act
        let overwritten_data = copy(&from, &to).unwrap();
        undo(&to, &overwritten_data).unwrap();
        let second_undo_res = undo(&to, &overwritten_data);

        // assert
        assert_exists_and_len(&from, data.len());
        assert!(!to.exists());
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_undo_before_write_or_copy() {
        // arrange
        let base = init_test_path("test_undo_before_write_or_copy");
        let path = base.join("file.txt");

        // act
        let undo_res = undo(&path, &None);

        // assert
        assert!(undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
