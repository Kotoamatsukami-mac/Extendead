use std::io;
use std::path::{Component, Path, PathBuf};

use crate::errors::AppError;

pub struct HomeTrashBoundary {
    pub home_canonical: PathBuf,
    pub trash_canonical: PathBuf,
}

pub fn canonical_home_and_trash() -> Result<HomeTrashBoundary, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::ValidationError("Cannot resolve home directory".to_string()))?;
    let home_canonical = std::fs::canonicalize(&home).map_err(|e| {
        AppError::ValidationError(format!("Cannot canonicalize home directory: {e}"))
    })?;
    let trash_canonical = home_canonical.join(".Trash");
    Ok(HomeTrashBoundary {
        home_canonical,
        trash_canonical,
    })
}

pub fn canonicalize_path_for_boundary(path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path must be absolute",
        ));
    }

    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "parent traversal is not allowed",
            ));
        }
    }

    if path.exists() {
        return std::fs::canonicalize(path);
    }

    let mut unresolved_components = Vec::new();
    let mut cursor = path;

    while !cursor.exists() {
        let Some(file_name) = cursor.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot resolve path root",
            ));
        };
        unresolved_components.push(file_name.to_os_string());
        let Some(parent) = cursor.parent() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path has no parent",
            ));
        };
        cursor = parent;
    }

    let mut canonical = std::fs::canonicalize(cursor)?;
    for component in unresolved_components.into_iter().rev() {
        canonical.push(component);
    }
    Ok(canonical)
}

pub fn canonicalize_existing_path_for_boundary(path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path must be absolute",
        ));
    }

    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "parent traversal is not allowed",
            ));
        }
    }

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "path does not exist",
        ));
    }

    std::fs::canonicalize(path)
}

pub fn destination_is_home_trash(destination_path: &str) -> Result<bool, AppError> {
    let boundary = canonical_home_and_trash()?;
    let destination = canonicalize_path_for_boundary(Path::new(destination_path)).map_err(|e| {
        AppError::ValidationError(format!(
            "Destination path '{destination_path}' cannot be resolved safely: {e}"
        ))
    })?;

    Ok(destination.starts_with(&boundary.trash_canonical)
        && destination.starts_with(&boundary.home_canonical))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_trash_destination_via_canonical_boundary() {
        if let Some(home) = dirs::home_dir() {
            let destination = format!("{}/.Trash/sample-file.txt", home.display());
            assert!(destination_is_home_trash(&destination).unwrap_or(false));
        }
    }

    #[test]
    fn does_not_treat_substring_match_as_trash() {
        if let Some(home) = dirs::home_dir() {
            let destination = format!("{}/Documents/not.Trash/sample-file.txt", home.display());
            assert!(!destination_is_home_trash(&destination).unwrap_or(false));
        }
    }
}
