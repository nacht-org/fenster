use std::{fs, io, path::Path};

/// Create parents of the path if they dont exist
pub fn create_parent_all(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(path)?;
        }
    }
    Ok(())
}
