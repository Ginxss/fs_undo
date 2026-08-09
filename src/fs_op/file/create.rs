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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{
        assert_exists_and_len, cleanup_test_path, init_test_path, random_bytes,
    };

    #[test]
    fn test_create_file() {
        // arrange
        let base = init_test_path("test_create_file");
        let path = base.join("created_file.txt");
        let data = random_bytes();

        // act
        execute(&path, &data).unwrap();

        // assert
        assert_exists_and_len(&path, data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_file_undo() {
        // arrange
        let base = init_test_path("test_create_file_undo");
        let path = base.join("created_file.txt");
        let data = random_bytes();

        // act
        execute(&path, &data).unwrap();
        undo(&path).unwrap();

        // assert
        assert!(!path.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_file_twice() {
        // arrange
        let base = init_test_path("test_create_file_twice");
        let path = base.join("created_file.txt");
        let data = random_bytes();

        // act
        execute(&path, &data).unwrap();
        let second_create_res = execute(&path, &data);

        // assert
        assert!(second_create_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_file_undo_twice() {
        // arrange
        let base = init_test_path("test_create_file_undo_twice");
        let path = base.join("created_file.txt");
        let data = random_bytes();

        // act
        execute(&path, &data).unwrap();
        undo(&path).unwrap();
        let second_undo_res = undo(&path);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_undo_before_create() {
        // arrange
        let base = init_test_path("test_undo_before_create");
        let path = base.join("file.txt");

        // act
        let undo_res = undo(&path);

        // assert
        assert!(undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
