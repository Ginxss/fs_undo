pub trait Undo {
    type Result;
    type Error;

    fn undo(&self) -> Result<Self::Result, Self::Error>;
}
