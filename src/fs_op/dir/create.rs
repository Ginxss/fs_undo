use std::{fs, io, path::Path};

pub fn execute(path: &Path) -> io::Result<()> {
    println!("Creating dir: {}", path.display());
    fs::create_dir(path)
}

pub fn undo(path: &Path) -> io::Result<()> {
    println!("Removing created dir: {}", path.display());
    fs::remove_dir(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{cleanup_test_path, init_test_path};

    #[test]
    fn test_create_dir() {
        // arrange
        let base = init_test_path("test_create_dir");
        let path = base.join("created_dir");

        // act
        execute(&path).unwrap();

        // assert
        assert!(path.exists());
        assert!(path.is_dir());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_no_parent() {
        // arrange
        let base = init_test_path("test_create_dir_no_parent");
        let path = base.join("parent/created_dir");

        // act
        let res = execute(&path);

        // assert
        assert!(res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_undo() {
        // arrange
        let base = init_test_path("test_create_dir_undo");
        let path = base.join("created_dir");

        // act
        execute(&path).unwrap();
        undo(&path).unwrap();

        // assert
        assert!(!path.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_twice() {
        // arrange
        let base = init_test_path("test_create_dir_twice");
        let path = base.join("created_dir");

        // act
        execute(&path).unwrap();
        let second_create_res = execute(&path);

        // assert
        assert!(second_create_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_undo_twice() {
        // arrange
        let base = init_test_path("test_create_dir_undo_twice");
        let path = base.join("created_dir");

        // act
        execute(&path).unwrap();
        undo(&path).unwrap();
        let second_undo_res = undo(&path);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_undo_before_create_dir() {
        // arrange
        let base = init_test_path("test_undo_before_create_dir");
        let path = base.join("dir");

        // act
        let undo_res = undo(&path);

        // assert
        assert!(undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
