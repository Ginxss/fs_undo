use std::{fs, io, os::unix, path::Path};

use log::info;

pub fn execute(path: &Path, target: &Path) -> io::Result<()> {
    info!(
        "Creating symlink: {} => {}",
        path.display(),
        target.display()
    );

    unix::fs::symlink(target, path)
}

pub fn undo(path: &Path, target: &Path) -> io::Result<()> {
    info!(
        "Removing created symlink: {} => {}",
        path.display(),
        target.display()
    );

    fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{
        assert_symlink_exists_and_len, cleanup_test_path, init_test_path, random_bytes,
    };

    #[test]
    fn test_create_symlink() {
        // arrange
        let base = init_test_path("test_create_symlink");
        let path = base.join("created_symlink");
        let target = base.join("target.txt");
        let target_data = random_bytes();
        fs::write(&target, &target_data).unwrap();
        let target = target.canonicalize().unwrap();

        // act
        execute(&path, &target).unwrap();

        // assert
        assert_symlink_exists_and_len(&path, target_data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_symlink_undo() {
        // arrange
        let base = init_test_path("test_create_symlink_undo");
        let path = base.join("created_symlink");
        let target = base.join("target.txt");
        let target_data = random_bytes();
        fs::write(&target, &target_data).unwrap();
        let target = target.canonicalize().unwrap();

        // act
        execute(&path, &target).unwrap();
        undo(&path, &target).unwrap();

        // assert
        assert!(!path.exists());
        assert!(target.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_symlink_twice() {
        // arrange
        let base = init_test_path("test_create_symlink_twice");
        let path = base.join("created_symlink");
        let target = base.join("target.txt");
        let target_data = random_bytes();
        fs::write(&target, &target_data).unwrap();
        let target = target.canonicalize().unwrap();

        // act
        execute(&path, &target).unwrap();
        let second_create_res = execute(&path, &target);

        // assert
        assert!(second_create_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_symlink_undo_twice() {
        // arrange
        let base = init_test_path("test_create_symlink_undo_twice");
        let path = base.join("created_symlink");
        let target = base.join("target.txt");
        let target_data = random_bytes();
        fs::write(&target, &target_data).unwrap();
        let target = target.canonicalize().unwrap();

        // act
        execute(&path, &target).unwrap();
        undo(&path, &target).unwrap();
        let second_undo_res = undo(&path, &target);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_undo_before_create_symlink() {
        // arrange
        let base = init_test_path("test_undo_before_create_symlink");
        let path = base.join("created_symlink");
        let target = base.join("target.txt");

        // act
        let undo_res = undo(&path, &target);

        // assert
        assert!(undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
