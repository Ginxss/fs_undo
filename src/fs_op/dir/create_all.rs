use std::{
    fs, io, iter,
    path::{Path, PathBuf},
};

/// Returns the first existing parent.
pub fn execute(path: &Path) -> io::Result<Option<PathBuf>> {
    println!("Creating dir including parents: {}", path.display());

    let existing_base = iter::successors(path.parent(), |parent| parent.parent())
        .find(|p| p.exists())
        .map(|p| p.to_path_buf());

    fs::create_dir_all(path)?;
    Ok(existing_base)
}

pub fn undo(path: &Path, existing_base: &Option<PathBuf>) -> io::Result<()> {
    println!(
        "Removing created dir including created parent dirs: {}",
        path.display()
    );

    iter::successors(Some(path), |p| p.parent())
        .take_while(|p| existing_base.as_ref().is_none_or(|b| p != b))
        .try_for_each(fs::remove_dir)?;

    Ok(())
}
