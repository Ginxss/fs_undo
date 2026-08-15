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
/// Each variant holds all information needed to undo itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsOp {
    /// Wraps [`File::create_new(path)`](std::fs::File::create_new) and [`write_all`](std::io::Write::write_all).
    ///
    /// # Undo
    ///
    /// Deletes `path` using [`fs::remove_file(path)`](std::fs::remove_file).
    CreateFile { path: PathBuf, data: Vec<u8> },

    /// Wraps [`fs::write(path, data)`](std::fs::write).
    ///
    /// # Before execution
    ///
    /// If `path` exists, reads the original contents using [`fs::read(path)`](std::fs::read) and stores them in `prev_data`.
    ///
    /// # Undo
    ///
    /// If `path` was overwritten:
    /// Overwrites `path` with `prev_data` using [`fs::write(path, prev_data)`](std::fs::write).
    ///
    /// Otherwise:
    /// Deletes `path` using [`fs::remove_file(path)`](std::fs::remove_file).
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
        /// Holds the data of the overwritten file, if available.
        prev_data: Option<Vec<u8>>,
    },

    /// Wraps [`fs::copy(from, to)`](std::fs::copy).
    ///
    /// # Before execution
    ///
    /// If `to` exists, reads the original contents using [`fs::read(to)`](std::fs::read) and stores them in `prev_data`.
    ///
    /// # Undo
    ///
    /// If `to` was overwritten:
    /// Overwrites `to` with `prev_data` using [`fs::write(to, prev_data)`](std::fs::write).
    ///
    /// Otherwise:
    /// Deletes `to` using [`fs::remove_file(to)`](std::fs::remove_file).
    CopyFile {
        from: PathBuf,
        to: PathBuf,
        /// Holds the data of the overwritten file, if available.
        prev_data: Option<Vec<u8>>,
    },

    /// Wraps [`fs::remove_file(path)`](std::fs::remove_file).
    ///
    /// # Before execution
    ///
    /// Reads the contents of `path` using [`fs::read(path)`](std::fs::read) and stores them in `data`.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] of kind [`InvalidInput`](io::ErrorKind::InvalidInput) if `path` is a symlink instead of traversing it.
    ///
    /// # Undo
    ///
    /// Creates a file at `path` with `data` using [`File::create_new(path)`](std::fs::File::create_new) and [`write_all`](std::io::Write::write_all).
    RemoveFile {
        path: PathBuf,
        data: Option<Vec<u8>>,
    },

    /// Wraps [`unix::fs::symlink(target, path)`](std::os::unix::fs::symlink).
    ///
    /// # Undo
    ///
    /// Deletes `path` using [`fs::remove_file(path)`](std::fs::remove_file) (Only the symlink is deleted).
    CreateSymlink { path: PathBuf, target: PathBuf },

    /// Wraps [`fs::remove_file(path)`](std::fs::remove_file).
    ///
    /// # Before execution
    ///
    /// Reads the link target using [`fs::read_link(path)`](std::fs::read_link) and stores it in `target`.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] of kind [`InvalidInput`](io::ErrorKind::InvalidInput) if `path` is not a symlink.
    ///
    /// # Undo
    ///
    /// Creates a symlink at `path` with `target` using [`unix::fs::symlink(target, path)`](std::os::unix::fs::symlink).
    RemoveSymlink {
        path: PathBuf,
        target: Option<PathBuf>,
    },

    /// Wraps [`fs::create_dir(path)`](std::fs::create_dir).
    ///
    /// # Undo
    ///
    /// Deletes `path` using [`fs::remove_dir(path)`](std::fs::remove_dir).
    CreateDir { path: PathBuf },

    /// Wraps [`fs::create_dir_all(path)`](std::fs::create_dir_all).
    ///
    /// # Before execution
    ///
    /// Stores the first existing parent in `existing_base`.
    /// Starting with `path`, calls [`parent()`](Path::parent) and [`exists()`](Path::exists) repeatedly
    /// until an existing dir or root is found.
    ///
    /// # Undo
    ///
    /// Deletes all dirs from `path` up to `existing_base` using [`fs::remove_dir`](std::fs::remove_dir) on each.
    CreateDirAll {
        path: PathBuf,
        /// The first existing parent folder at the time of execution, searching upwards from `path`.
        existing_base: Option<PathBuf>,
    },

    /// Wraps [`fs::remove_dir(path)`](std::fs::remove_dir).
    ///
    /// # Undo
    ///
    /// Creates an empty dir at `path` using [`fs::create_dir(path)`](std::fs::create_dir).
    RemoveEmptyDir { path: PathBuf },

    /// Does not wrap a single operation.
    /// Recursively walks the dir at `path`, deletes each entry and stores the delete operations in `ops`.
    ///
    /// Tries to make [`execute()`](FsOp::execute) as atomic as possible:
    /// If there is an error while removing an entry, tries to undo all previous operations before returning the error.
    /// Only returns `Ok(())` if all delete operations were successful.
    ///
    /// Keep in mind that each operation holds its own state in memory, including the full contents
    /// of deleted files, so this can get very expensive.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] of kind [`InvalidInput`](io::ErrorKind::InvalidInput) if:
    /// - `path` is a symlink (does not traverse it)
    /// - `path` is not a dir
    ///
    /// # Undo
    ///
    /// Reverts all operations in reverse order of execution using `ops.undo()`.
    RemoveDirAll {
        path: PathBuf,
        /// All delete operations for the recursive deletion of this directory in order.
        ops: Vec<FsOp>,
    },

    /// Wraps [`fs::rename(from, to)`](std::fs::rename).
    ///
    /// # Undo
    ///
    /// Renames `to` to `from` using [`fs::rename(to, from)`](std::fs::rename).
    Rename { from: PathBuf, to: PathBuf },
}

impl FsOp {
    /// Creates a [`FsOp::CreateFile`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn create_file(path: impl AsRef<Path>, data: Vec<u8>) -> FsOp {
        FsOp::CreateFile {
            path: path.as_ref().to_path_buf(),
            data,
        }
    }

    /// Creates a [`FsOp::WriteFile`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn write_file(path: impl AsRef<Path>, data: Vec<u8>) -> FsOp {
        FsOp::WriteFile {
            path: path.as_ref().to_path_buf(),
            data,
            prev_data: None,
        }
    }

    /// Creates a [`FsOp::CopyFile`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `from` and `to` into owned [`PathBuf`]s.
    pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> FsOp {
        FsOp::CopyFile {
            from: from.as_ref().to_path_buf(),
            to: to.as_ref().to_path_buf(),
            prev_data: None,
        }
    }

    /// Creates a [`FsOp::RemoveFile`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn remove_file(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveFile {
            path: path.as_ref().to_path_buf(),
            data: None,
        }
    }

    /// Creates a [`FsOp::CreateSymlink`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` and `target` into owned [`PathBuf`]s.
    pub fn create_symlink(path: impl AsRef<Path>, target: impl AsRef<Path>) -> FsOp {
        FsOp::CreateSymlink {
            path: path.as_ref().to_path_buf(),
            target: target.as_ref().to_path_buf(),
        }
    }

    /// Creates a [`FsOp::RemoveSymlink`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn remove_symlink(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveSymlink {
            path: path.as_ref().to_path_buf(),
            target: None,
        }
    }

    /// Creates a [`FsOp::CreateDir`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn create_dir(path: impl AsRef<Path>) -> FsOp {
        FsOp::CreateDir {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Creates a [`FsOp::CreateDirAll`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn create_dir_all(path: impl AsRef<Path>) -> FsOp {
        FsOp::CreateDirAll {
            path: path.as_ref().to_path_buf(),
            existing_base: None,
        }
    }

    /// Creates a [`FsOp::RemoveEmptyDir`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn remove_empty_dir(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveEmptyDir {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Creates a [`FsOp::RemoveDirAll`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `path` into an owned [`PathBuf`].
    pub fn remove_dir_all(path: impl AsRef<Path>) -> FsOp {
        FsOp::RemoveDirAll {
            path: path.as_ref().to_path_buf(),
            ops: Vec::new(),
        }
    }

    /// Creates a [`FsOp::Rename`] variant.
    ///
    /// Does not execute the operation.
    /// Turns `from` and `to` into owned [`PathBuf`]s.
    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> FsOp {
        FsOp::Rename {
            from: from.as_ref().to_path_buf(),
            to: to.as_ref().to_path_buf(),
        }
    }

    /// Executes the operation.
    ///
    /// # Before execution
    ///
    /// Queries and stores any data needed to reverse itself, as described in the individual [`FsOp`] variants.
    /// This can fail as well.
    ///
    /// The [`log`] crate is used to print descriptive text about the operation to be executed with log level [`Info`](log::Level::Info).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use fs_undo::fs_op::FsOp;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let mut op = FsOp::copy_file("a.txt", "b.txt");
    ///     op.execute()
    /// }
    /// ```
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

            FsOp::CreateSymlink { path, target } => symlink::create::execute(path, target),

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

    /// [`Undo`] implementation for [`FsOp`].
    ///
    /// Behaviour is based on and described in the specific [`FsOp`] variants.
    /// Chained filesystem operations that depend on each other always need to be undone in order.
    /// There is no protection against calling `undo()` before [`execute()`](FsOp::execute()).
    ///
    /// Expects the filesystem state to not have changed for the context of the operation since execution. For example:
    /// - Undo on [`FsOp::CreateFile`] deletes the created file, expecting the path to exist, and failing otherwise.
    /// - Undo on [`FsOp::RemoveFile`] recreates the deleted file, expecting the path to not exist, and failing otherwise.
    /// - Undo on [`FsOp::CreateDir`] deletes the created dir, expecting it to be empty.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use fs_undo::{fs_op::FsOp, undo::Undo};
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let mut op = FsOp::copy_file("a.txt", "b.txt");
    ///     op.execute()?;
    ///     op.undo()
    /// }
    /// ```
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

            FsOp::CreateSymlink { path, target } => symlink::create::undo(path, target),

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

    /// [`Undo`] implementation for `Vec<FsOp>`.
    ///
    /// Calls [`FsOp::undo()`] on every operation in reverse order.
    ///
    /// Returns with the error as soon as one operation can't be undone.
    /// Only returns `Ok(())` if all operations were successfully undone.
    fn undo(&self) -> Result<Self::Result, Self::Error> {
        for op in self.iter().rev() {
            op.undo()?;
        }

        Ok(())
    }
}
