use std::{
    fs::{self, File},
    io::{self, Write},
    iter,
    os::unix,
    path::{Path, PathBuf},
};

use crate::{err::io_err_invalid_filetype, undo::Undo};

/// Enum representing reversible filesystem operations.
/// Each variant stores all information needed to undo itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsOp {
    /// Wraps `File::create_new` and `write_all` (Fails if the file already exists).
    /// Undo deletes `path`.
    CreateFile { path: PathBuf, data: Vec<u8> },

    /// Wraps `fs::write` (Overwrites existing file).
    /// If `path` will be overwritten, stores the original contents in `previous_data` on execution.
    /// Undo overwrites `path` with the original contents if available, and deletes `path` otherwise.
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
        previous_data: Option<Vec<u8>>,
    },

    /// Wraps `fs::copy` (Overwrites existing file).
    /// Fails if `to == from`.
    /// If `to` will be overwritten, stores the original contents in `previous_data` on execution.
    /// Undo overwrites `to` with the original contents if available, and deletes `to` otherwise.
    CopyFile {
        from: PathBuf,
        to: PathBuf,
        previous_data: Option<Vec<u8>>,
    },

    /// Wraps `fs::remove_file`.
    /// Stores the contents of `path` in `data` on execution.
    /// Undo creates a file at `path` with `data`.
    RemoveFile {
        path: PathBuf,
        data: Option<Vec<u8>>,
    },

    /// Wraps `unix::fs::symlink` (Fails if `path` already exists).
    /// Fails if `target` does not exist on execution. TODO: make this optional
    /// Undo deletes `path` (Only the symlink is deleted).
    CreateSymlink { path: PathBuf, target: PathBuf },

    /// Wraps `fs::remove_file`.
    /// Stores the link target in `target` on execution.
    /// Undo creates a symlink at `path` with target `target`
    RemoveSymlink {
        path: PathBuf,
        target: Option<PathBuf>,
    },

    /// Wraps `fs::create_dir` (Does not create missing parent directories).
    /// Undo deletes `path`, expecting it to be an empty dir.
    CreateDir { path: PathBuf },

    /// Wraps `fs::create_dir_all` (Creates missing parent directories).
    /// Stores the first existing parent in `existing_base` on execution.
    /// Undo deletes all created dirs from `path` up to `existing_base`, expecting `path` to be an empty dir.
    CreateDirAll {
        path: PathBuf,
        existing_base: Option<PathBuf>,
    },

    /// Wraps `fs::remove_dir`.
    /// Undo creates a dir at `path`.
    RemoveEmptyDir { path: PathBuf },

    /// Does not wrap a single operation. Recursively walks the dir at `path` and stores delete operations in `ops`.
    /// Undo reverts all operations in order.
    ///
    /// Fails if `path` is a symlink instead of traversing it.
    ///
    /// Keep in mind that every operation holds its own state in memory, including the full contents
    /// of deleted files, so this can get very expensive.
    RemoveDirAll { path: PathBuf, ops: Vec<FsOp> },

    /// Wraps `fs::rename`.
    /// Undo renames `to` to `from`.
    Rename { from: PathBuf, to: PathBuf },
}

impl FsOp {
    pub fn create_file(path: impl AsRef<Path>, data: Vec<u8>) -> FsOp {
        FsOp::CreateFile {
            path: path.as_ref().to_path_buf(),
            data,
        }
    }

    pub fn write_file(path: impl AsRef<Path>, data: Vec<u8>) -> FsOp {
        FsOp::WriteFile {
            path: path.as_ref().to_path_buf(),
            data,
            previous_data: None,
        }
    }

    pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> FsOp {
        FsOp::CopyFile {
            from: from.as_ref().to_path_buf(),
            to: to.as_ref().to_path_buf(),
            previous_data: None,
        }
    }

    pub fn remove_file(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveFile {
            path: path.as_ref().to_path_buf(),
            data: None,
        }
    }

    pub fn create_symlink(path: impl AsRef<Path>, target: impl AsRef<Path>) -> FsOp {
        FsOp::CreateSymlink {
            path: path.as_ref().to_path_buf(),
            target: target.as_ref().to_path_buf(),
        }
    }

    pub fn remove_symlink(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveSymlink {
            path: path.as_ref().to_path_buf(),
            target: None,
        }
    }

    pub fn create_dir(path: impl AsRef<Path>) -> FsOp {
        FsOp::CreateDir {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn create_dir_all(path: impl AsRef<Path>) -> FsOp {
        FsOp::CreateDirAll {
            path: path.as_ref().to_path_buf(),
            existing_base: None,
        }
    }

    pub fn remove_empty_dir(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveEmptyDir {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn remove_dir_all(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveDirAll {
            path: path.as_ref().to_path_buf(),
            ops: Vec::new(),
        }
    }

    /// Executes the operation, storing any data needed to reverse it.
    /// For example, before a file can be deleted, its contents need to be read, which can fail as well.
    /// TODO: logging can be turned off...
    pub fn execute(&mut self) -> io::Result<()> {
        match self {
            FsOp::CreateFile { path, data } => {
                // TODO: Make prints configurable
                println!("Creating file: {}", path.display());
                let mut file = File::create_new(path)?;
                file.write_all(data)
            }

            FsOp::WriteFile {
                path,
                data,
                previous_data,
            } => {
                println!("Writing file: {}", path.display());

                if path.exists() {
                    let data = fs::read(&path)?;
                    previous_data.replace(data);
                }

                fs::write(path, data)
            }

            FsOp::CopyFile {
                from,
                to,
                previous_data,
            } => {
                println!("Copying file: {} => {}", from.display(), to.display());

                if to == from {
                    // TODO: better?
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Copy target {} is the same as the source", to.display()),
                    ));
                }

                if to.exists() {
                    let data = fs::read(&to)?;
                    previous_data.replace(data);
                }

                fs::copy(from, to).map(|_| ())
            }

            FsOp::RemoveFile { path, data } => {
                println!("Removing file: {}", path.display());
                data.replace(fs::read(&path)?);
                fs::remove_file(path)
            }

            FsOp::CreateSymlink { path, target } => {
                println!("Creating link: {} => {}", path.display(), target.display());

                if !target.exists() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Link target {} does not exist", target.display()),
                    ));
                }

                unix::fs::symlink(target, path)
            }

            FsOp::RemoveSymlink { path, target } => {
                target.replace(fs::read_link(&path)?);

                println!(
                    "Removing symlink: {} => {}",
                    path.display(),
                    target.as_ref().unwrap().display()
                );

                fs::remove_file(path)
            }

            FsOp::CreateDir { path } => {
                println!("Creating dir: {}", path.display());
                fs::create_dir(path)
            }

            FsOp::CreateDirAll {
                path,
                existing_base,
            } => {
                println!("Creating dir including parents: {}", path.display());

                *existing_base = iter::successors(path.parent(), |parent| parent.parent())
                    .find(|p| p.exists())
                    .map(|p| p.to_path_buf());

                fs::create_dir_all(path)
            }

            FsOp::RemoveEmptyDir { path } => {
                println!("Removing empty dir: {}", path.display());
                fs::remove_dir(path)
            }

            FsOp::RemoveDirAll { path, ops } => {
                println!("Removing dir including contents: {}", path.display());

                if path.is_symlink() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("RemoveDirAll input path {} is a symlink", path.display()),
                    ));
                }

                if !path.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("RemoveDirAll input path {} is not a dir", path.display()),
                    ));
                }

                Self::delete_dir_recursive(path, ops)
            }

            FsOp::Rename { from, to } => {
                println!("Renaming: {} => {}", from.display(), to.display());
                fs::rename(from, to)
            }
        }
    }

    fn delete_dir_recursive(path: impl AsRef<Path>, fs_ops: &mut Vec<FsOp>) -> io::Result<()> {
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
                Self::delete_dir_recursive(entry_path, fs_ops)?;
            } else {
                return io_err_invalid_filetype(filetype);
            }
        }

        let mut op = FsOp::remove_empty_dir(path);
        op.execute()?;
        fs_ops.push(op);

        Ok(())
    }
}

impl Undo for FsOp {
    type Result = ();
    type Error = io::Error;

    /// `Undo` implementation for `FsOp`.
    ///
    /// Chained filesystem operations always need to be undone in order.
    /// There is no protection against calling `undo()` before `execute()`.
    ///
    /// Undo expects the filesystem state to not have changed for the context of the operation.
    ///
    /// For example:
    /// - Undo on `FsOp::CreateFile` deletes the created file, expecting the path to exist, and failing otherwise.
    /// - Undo on `FsOp::DeleteFile` recreates the deleted file, expecting the path to not exist, and failing otherwise.
    /// - Undo on `FsOp::CreateDir` deletes the created dir, expecting it to be empty.
    fn undo(&self) -> Result<Self::Result, Self::Error> {
        match self {
            FsOp::CreateFile { path, data: _ } => {
                println!("Removing created file: {}", path.display());
                fs::remove_file(path)
            }

            FsOp::WriteFile {
                path: overwritten_path,
                data: _,
                previous_data,
            }
            | FsOp::CopyFile {
                from: _,
                to: overwritten_path,
                previous_data,
            } => match previous_data {
                Some(previous_data) => {
                    println!("Restoring overwritten file: {}", overwritten_path.display());
                    fs::write(overwritten_path, previous_data)
                }
                None => {
                    println!("Removing created file: {}", overwritten_path.display());
                    fs::remove_file(overwritten_path)
                }
            },

            FsOp::RemoveFile { path, data } => {
                println!("Recreating file: {}", path.display());

                let data = data
                    .as_ref()
                    .expect("data should have been filled during execute()");

                let mut file = File::create_new(path)?;
                file.write_all(data)
            }

            FsOp::CreateSymlink { path, target } => {
                println!(
                    "Removing created link: {} => {}",
                    path.display(),
                    target.display()
                );

                fs::remove_file(path)
            }
            FsOp::RemoveSymlink { path, target } => {
                let target = target
                    .as_ref()
                    .expect("Should have been filled during execute()");

                println!(
                    "Recreating link: {} => {}",
                    path.display(),
                    target.display()
                );

                unix::fs::symlink(target, path)
            }

            FsOp::CreateDir { path } => {
                println!("Removing created dir: {}", path.display());
                fs::remove_dir(path)
            }

            FsOp::CreateDirAll {
                path,
                existing_base,
            } => {
                println!(
                    "Removing created dir including created parent dirs: {}",
                    path.display()
                );

                iter::successors(Some(path.as_path()), |p| p.parent())
                    .take_while(|p| existing_base.as_ref().is_none_or(|b| p != b))
                    .try_for_each(fs::remove_dir)?;

                Ok(())
            }

            FsOp::RemoveEmptyDir { path } => {
                println!("Recreating empty dir: {}", path.display());
                fs::create_dir(path)
            }

            FsOp::RemoveDirAll { path, ops } => {
                println!("Recreating dir including contents: {}", path.display());
                ops.undo()
            }

            FsOp::Rename { from, to } => {
                println!("Renaming back: {} => {}", to.display(), from.display());
                fs::rename(to, from)
            }
        }
    }
}

impl Undo for Vec<FsOp> {
    type Result = ();
    type Error = io::Error;

    // TODO: better rust docs...
    /// `Undo` implementation for `Vec<FsOp>`.
    ///
    /// Calls `undo()` for every operation in reverse order.
    ///
    /// Returns with an error as soon as one operation can't be undone.
    ///
    /// Only returns Ok(()) if all operations were successfully undone.
    fn undo(&self) -> Result<Self::Result, Self::Error> {
        for op in self.iter().rev() {
            op.undo()?;
        }

        Ok(())
    }
}
