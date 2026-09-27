//! Read-only file identity checks for destinations whose parents may not exist.

use std::{
    io,
    path::{Component, Path, PathBuf},
};

/// Compare file identity after resolving prospective destination components.
/// Existing symlinks are followed before a subsequent `..` is processed.
pub fn paths_alias(first: &Path, second: &Path) -> io::Result<bool> {
    let first = resolve_destination(first)?;
    let second = resolve_destination(second)?;
    if first == second {
        return Ok(true);
    }
    match same_file::is_same_file(first, second) {
        Ok(same) => Ok(same),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn resolve_destination(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut resolved = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => resolved.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(name) => {
                resolved.push(name);
                match resolved.canonicalize() {
                    Ok(existing) => resolved = existing,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        // Missing directories may be created by publication. A
                        // dangling symlink is not a missing directory: do not
                        // guess where an unresolved link would lead.
                        match resolved.symlink_metadata() {
                            Ok(_) => return Err(error),
                            Err(missing) if missing.kind() == io::ErrorKind::NotFound => {}
                            Err(other) => return Err(other),
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(resolved)
}
