use std::{fs, io, path::Path};

use crate::{
    err::{io_err_invalid_filetype, io_err_invalid_input},
    fs_op::FsOp,
    undo::Undo,
};

/// Tries to be as atomic as possible:
/// If there is an error while removing, tries to undo all previous operations before returning the error.
/// Only returns Ok(()) if all delete operations were successful.
///
/// Returns all delete operations in the order they were executed.
pub fn execute(path: &Path) -> io::Result<Vec<FsOp>> {
    println!("Removing dir including contents: {}", path.display());

    if path.is_symlink() {
        let msg = &format!("RemoveDirAll input path {} is a symlink", path.display());
        return io_err_invalid_input(msg);
    }

    if !path.is_dir() {
        let msg = &format!("RemoveDirAll input path {} is not a dir", path.display());
        return io_err_invalid_input(msg);
    }

    let mut ops = Vec::new();
    match remove_dir_recursive(path, &mut ops) {
        Ok(()) => Ok(ops),
        Err(err) => {
            ops.undo()?;
            Err(err)
        }
    }
}

fn remove_dir_recursive(path: impl AsRef<Path>, fs_ops: &mut Vec<FsOp>) -> io::Result<()> {
    for entry in fs::read_dir(&path)? {
        let entry = entry?;

        let filetype = entry.file_type()?;
        let entry_path = entry.path();

        if filetype.is_file() {
            let mut op = FsOp::remove_file(&entry_path);
            op.execute()?;
            fs_ops.push(op);
        } else if filetype.is_symlink() {
            let mut op = FsOp::remove_symlink(&entry_path);
            op.execute()?;
            fs_ops.push(op);
        } else if filetype.is_dir() {
            remove_dir_recursive(entry_path, fs_ops)?;
        } else {
            return io_err_invalid_filetype(filetype);
        }
    }

    let mut op = FsOp::remove_empty_dir(path);
    op.execute()?;
    fs_ops.push(op);

    Ok(())
}

pub fn undo(path: &Path, ops: &Vec<FsOp>) -> io::Result<()> {
    println!("Recreating dir including contents: {}", path.display());
    ops.undo()
}

#[cfg(test)]
mod tests {
    use std::os::unix;

    use rand::{
        RngExt,
        distr::{Alphanumeric, SampleString},
    };

    use super::*;

    use crate::test_util::{cleanup_test_path, init_test_path, random_bytes};

    #[test]
    fn test_remove_dir_all() {
        // arrange
        let base = init_test_path("test_remove_dir_all");
        let path = base.join("dir_to_remove");
        create_random_dir(&path, 0);

        // act
        execute(&path).unwrap();

        // assert
        assert!(!path.exists());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_dir_all_undo() {
        // arrange
        let base = init_test_path("test_remove_dir_all_undo");
        let path = base.join("dir_to_remove");
        create_random_dir(&path, 0);
        let dir_size_before = get_total_dir_size(&path).unwrap();

        // act
        let ops = execute(&path).unwrap();
        undo(&path, &ops).unwrap();
        let dir_size_after = get_total_dir_size(&path).unwrap();

        // assert
        assert!(path.exists());
        assert_eq!(dir_size_before, dir_size_after);

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_dir_all_twice() {
        // arrange
        let base = init_test_path("test_remove_dir_all_twice");
        let path = base.join("dir_to_remove");
        create_random_dir(&path, 0);

        // act
        execute(&path).unwrap();
        let second_create_res = execute(&path);

        // assert
        assert!(second_create_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    #[test]
    fn test_remove_dir_all_undo_twice() {
        // arrange
        let base = init_test_path("test_remove_dir_all_undo_twice");
        let path = base.join("dir_to_remove");
        create_random_dir(&path, 0);

        // act
        let ops = execute(&path).unwrap();
        undo(&path, &ops).unwrap();
        let second_undo_res = undo(&path, &ops);

        // assert
        assert!(second_undo_res.is_err());

        // cleanup
        cleanup_test_path(base);
    }

    fn create_random_dir(path: &Path, depth: i32) {
        if depth > 10 {
            return;
        }

        fs::create_dir(path).unwrap();

        let mut rng = rand::rng();
        let num_elements = rng.random_range(0..=10);

        for _ in 0..num_elements {
            let name = Alphanumeric.sample_string(&mut rng, 10);
            let path = path.join(&name);

            match rng.random_range(0..3) {
                0 => fs::write(&path, random_bytes()).unwrap(),
                1 => {
                    let target_name = Alphanumeric.sample_string(&mut rng, 10);
                    unix::fs::symlink(&target_name, &path).unwrap()
                }
                2 => create_random_dir(&path, depth + 1),
                _ => (),
            };
        }
    }

    fn get_total_dir_size(path: &Path) -> io::Result<u64> {
        let mut total = 0;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            // does not follow symlinks
            let metadata = fs::symlink_metadata(&entry_path)?;

            total += if metadata.is_dir() {
                get_total_dir_size(&entry_path)?
            } else {
                metadata.len()
            };
        }

        Ok(total)
    }
}
