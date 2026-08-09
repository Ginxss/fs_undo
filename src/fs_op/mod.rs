mod dir;
mod file;
mod rename;
mod symlink;

use std::{
    io::{self},
    path::{Path, PathBuf},
};

use crate::undo::Undo;

/// Enum representing reversible filesystem operations.
/// Each variant stores all information needed to undo itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsOp {
    /// Wraps `File::create_new` and `write_all` (Fails if the file already exists).
    /// Undo deletes `path`.
    CreateFile { path: PathBuf, data: Vec<u8> },

    /// Wraps `fs::write` (Overwrites existing file).
    /// If `path` will be overwritten, stores the original contents in `prev_data` on execution.
    /// Undo overwrites `path` with the original contents if available, and deletes `path` otherwise.
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
        prev_data: Option<Vec<u8>>,
    },

    /// Wraps `fs::copy` (Overwrites existing file).
    /// Fails if `to == from`.
    /// If `to` will be overwritten, stores the original contents in `previous_data` on execution.
    /// Undo overwrites `to` with the original contents if available, and deletes `to` otherwise.
    CopyFile {
        from: PathBuf,
        to: PathBuf,
        prev_data: Option<Vec<u8>>,
    },

    /// Wraps `fs::remove_file`.
    /// Stores the contents of `path` in `data` on execution.
    /// Undo creates a file at `path` with `data`.
    RemoveFile {
        path: PathBuf,
        data: Option<Vec<u8>>,
    },

    /// Wraps `unix::fs::symlink` (Fails if `path` already exists).
    /// Optionally fails if `target` does not exist on execution (set `fail_on_missing_target`).
    /// Undo deletes `path` (Only the symlink is deleted).
    CreateSymlink {
        path: PathBuf,
        target: PathBuf,
        fail_on_missing_target: bool,
    },

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
            prev_data: None,
        }
    }

    pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> FsOp {
        FsOp::CopyFile {
            from: from.as_ref().to_path_buf(),
            to: to.as_ref().to_path_buf(),
            prev_data: None,
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
            fail_on_missing_target: false,
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

    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> FsOp {
        FsOp::Rename {
            from: from.as_ref().to_path_buf(),
            to: to.as_ref().to_path_buf(),
        }
    }

    // TODO: logging configurable with log level?
    /// Executes the operation, storing any data needed to reverse it.
    /// For example, before a file can be deleted, its contents need to be read, which can fail as well.
    pub fn execute(&mut self) -> io::Result<()> {
        match self {
            FsOp::CreateFile { path, data } => file::create::execute(path, data),

            FsOp::WriteFile {
                path,
                data,
                prev_data,
            } => {
                *prev_data = file::write_copy::write(path, data)?;
                Ok(())
            }

            FsOp::CopyFile {
                from,
                to,
                prev_data,
            } => {
                *prev_data = file::write_copy::copy(from, to)?;
                Ok(())
            }

            FsOp::RemoveFile { path, data } => {
                data.replace(file::remove::execute(path)?);
                Ok(())
            }

            FsOp::CreateSymlink {
                path,
                target,
                fail_on_missing_target,
            } => symlink::create::execute(path, target, *fail_on_missing_target),

            FsOp::RemoveSymlink { path, target } => {
                target.replace(symlink::remove::execute(path)?);
                Ok(())
            }

            FsOp::CreateDir { path } => dir::create::execute(path),

            FsOp::CreateDirAll {
                path,
                existing_base,
            } => {
                *existing_base = dir::create_all::execute(path)?;
                Ok(())
            }

            FsOp::RemoveEmptyDir { path } => dir::remove_empty::execute(path),

            FsOp::RemoveDirAll { path, ops } => {
                *ops = dir::remove_all::execute(path)?;
                Ok(())
            }

            FsOp::Rename { from, to } => rename::execute(from, to),
        }
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
            FsOp::CreateFile { path, data: _ } => file::create::undo(path),

            FsOp::WriteFile {
                path: written_path,
                data: _,
                prev_data,
            }
            | FsOp::CopyFile {
                from: _,
                to: written_path,
                prev_data,
            } => file::write_copy::undo(written_path, prev_data),

            FsOp::RemoveFile { path, data } => file::remove::undo(path, data),

            FsOp::CreateSymlink {
                path,
                target,
                fail_on_missing_target: _,
            } => symlink::create::undo(path, target),

            FsOp::RemoveSymlink { path, target } => symlink::remove::undo(path, target),

            FsOp::CreateDir { path } => dir::create::undo(path),

            FsOp::CreateDirAll {
                path,
                existing_base,
            } => dir::create_all::undo(path, existing_base),

            FsOp::RemoveEmptyDir { path } => dir::remove_empty::undo(path),

            FsOp::RemoveDirAll { path, ops } => dir::remove_all::undo(path, ops),

            FsOp::Rename { from, to } => rename::undo(from, to),
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
