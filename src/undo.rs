/// Trait for reversing filesystem operations.
/// Is implemented by default on [`FsOp`](crate::fs_op::FsOp) and `Vec<FsOp>`.
pub trait Undo {
    type Result;
    type Error;

    fn undo(&self) -> Result<Self::Result, Self::Error>;
}
