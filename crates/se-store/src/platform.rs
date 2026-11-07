//! Operating-system specifics of persistence (golden rule 10: OS-specific code lives here).

use std::{io, path::Path};

/// Makes a rename inside `directory` durable (Unix needs the directory itself flushed).
#[cfg(unix)]
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    std::fs::File::open(directory)?.sync_all()
}

/// Makes a rename inside `directory` durable. NTFS journals renames itself, and Windows cannot
/// open a directory as a file, so there is nothing to do.
#[cfg(not(unix))]
pub fn sync_directory(_directory: &Path) -> io::Result<()> {
    Ok(())
}

/// Moves `path` to the trash (Recycle Bin, macOS Trash, freedesktop trash).
///
/// On macOS the `NSFileManager` method is used: the default (Finder, driven by an `osascript`) asks
/// the user for an automation permission.
pub fn move_to_trash(path: &Path) -> Result<(), trash::Error> {
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        let mut context = trash::TrashContext::default();
        context.set_delete_method(DeleteMethod::NsFileManager);
        context.delete(path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        trash::delete(path)
    }
}
