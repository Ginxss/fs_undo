use std::{
    fs, io, iter,
    path::{Path, PathBuf},
};

use log::info;

/// Returns the first existing parent.
pub fn execute(path: &Path) -> io::Result<Option<PathBuf>> {
    info!(
        "Creating dir including missing parent dirs: {}",
        path.display()
    );

    let existing_base = iter::successors(path.parent(), |parent| parent.parent())
        .find(|p| p.exists())
        .map(|p| p.to_path_buf());

    fs::create_dir_all(path)?;
    Ok(existing_base)
}

pub fn undo(path: &Path, existing_base: &Option<PathBuf>) -> io::Result<()> {
    info!(
        "Removing created dir including created parent dirs: {}",
        path.display()
    );

    iter::successors(Some(path), |p| p.parent())
        .take_while(|p| existing_base.as_ref().is_none_or(|b| p != b))
        .try_for_each(fs::remove_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::{cleanup_test_path, init_test_path};

    #[test]
    fn test_create_dir_all() {
        // arrange
        let base = init_test_path("test_create_dir_all");
        let path = base.join("parent2/parent1/created_dir");

        // act
        execute(&path).unwrap();

        // assert
        assert!(path.exists());
        assert!(path.is_dir());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_all_undo() {
        // arrange
        let base = init_test_path("test_create_dir_all_undo");
        let first_missing_parent = "parent2";
        let path = base.join(first_missing_parent).join("parent1/created_dir");

        // act
        let existing_base = execute(&path).unwrap();
        undo(&path, &existing_base).unwrap();

        // assert
        assert!(base.exists());
        assert!(!base.join(first_missing_parent).exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_all_twice() {
        // arrange
        let base = init_test_path("test_create_dir_all_twice");
        let path = base.join("parent2/parent1/created_dir");

        // act
        execute(&path).unwrap();
        execute(&path).unwrap();

        // assert
        assert!(path.exists());
        assert!(path.is_dir());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_create_dir_all_undo_twice() {
        // arrange
        let base = init_test_path("test_create_dir_all_undo_twice");
        let path = base.join("parent2/parent1/created_dir");

        // act
        let existing_base = execute(&path).unwrap();
        undo(&path, &existing_base).unwrap();
        let second_undo_res = undo(&path, &existing_base);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_undo_before_create_dir_all() {
        // arrange
        let base = init_test_path("test_undo_before_create_dir_all");
        let path = base.join("parent2/parent1/dir");

        // act
        let undo_res = undo(&path, &None);

        // assert
        assert!(undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }
}
