use std::{fs, io, path::Path};

pub fn execute(path: &Path) -> io::Result<()> {
    println!("Removing empty dir: {}", path.display());
    fs::remove_dir(path)
}

pub fn undo(path: &Path) -> io::Result<()> {
    println!("Recreating empty dir: {}", path.display());
    fs::create_dir(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{cleanup_test_path, init_test_path};

    #[test]
    fn test_remove_empty_dir() {
        // arrange
        let base = init_test_path("test_remove_empty_dir");
        let path = base.join("dir_to_remove");
        fs::create_dir(&path).unwrap();

        // act
        execute(&path).unwrap();

        // assert
        assert!(!path.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_empty_dir_undo() {
        // arrange
        let base = init_test_path("test_remove_empty_dir_undo");
        let path = base.join("dir_to_remove");
        fs::create_dir(&path).unwrap();

        // act
        execute(&path).unwrap();
        undo(&path).unwrap();

        // assert
        assert!(path.exists());
        assert!(path.is_dir());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_empty_dir_twice() {
        // arrange
        let base = init_test_path("test_remove_empty_dir_twice");
        let path = base.join("dir_to_remove");
        fs::create_dir(&path).unwrap();

        // act
        execute(&path).unwrap();
        let second_remove_res = execute(&path);

        // assert
        assert!(second_remove_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_empty_dir_undo_twice() {
        // arrange
        let base = init_test_path("test_remove_empty_dir_undo_twice");
        let path = base.join("dir_to_remove");
        fs::create_dir(&path).unwrap();

        // act
        execute(&path).unwrap();
        undo(&path).unwrap();
        let second_undo_res = undo(&path);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
