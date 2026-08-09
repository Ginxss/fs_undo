use std::{fs, io, os::unix, path::Path};

pub fn execute(path: &Path, target: &Path, fail_on_missing_target: bool) -> io::Result<()> {
    println!("Creating link: {} => {}", path.display(), target.display());

    if fail_on_missing_target && !target.exists() {
        let msg = format!("Link target {} does not exist", target.display());
        return Err(io::Error::new(io::ErrorKind::NotFound, msg));
    }

    unix::fs::symlink(target, path)
}

pub fn undo(path: &Path, target: &Path) -> io::Result<()> {
    println!(
        "Removing created link: {} => {}",
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
        execute(&path, &target, false).unwrap();

        // assert
        assert_symlink_exists_and_len(&path, target_data.len());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_symlink_fail_on_missing_target() {
        // arrange
        let base = init_test_path("test_create_symlink_fail_on_missing_target");
        let path = base.join("created_symlink");
        let target = base.join("target.txt");

        // act
        let res = execute(&path, &target, true);

        // assert
        assert!(res.is_err());

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
        execute(&path, &target, false).unwrap();
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
        execute(&path, &target, false).unwrap();
        let second_create_res = execute(&path, &target, false);

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
        execute(&path, &target, false).unwrap();
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
