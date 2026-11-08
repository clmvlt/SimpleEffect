//! Where Claude Code is usually installed, per operating system (golden rule 10). Apps started
//! from the desktop do not inherit the PATH of the user's shell (macOS gives them
//! `/usr/bin:/bin:/usr/sbin:/sbin`), so these folders are searched after the PATH.

use std::path::{Path, PathBuf};

/// Names looked up on the PATH, in order. On Windows the real executable comes before script
/// wrappers found through PATHEXT: a `.cmd` may add options of its own (a user's wrapper adding
/// `--allow-dangerously-skip-permissions` breaks `--restricted`) and passes arguments through
/// cmd.exe. npm installs only have `claude.cmd`, found by the second name.
#[cfg(windows)]
pub(super) const PATH_NAMES: &[&str] = &["claude.exe", "claude"];

/// Names looked up on the PATH, in order.
#[cfg(not(windows))]
pub(super) const PATH_NAMES: &[&str] = &["claude"];

/// Well-known executables of Claude Code under `home`, most likely first.
#[cfg(windows)]
pub(super) fn install_locations(home: &Path) -> Vec<PathBuf> {
    vec![
        // Native installer.
        home.join(".local").join("bin").join("claude.exe"),
        // npm global install.
        home.join("AppData")
            .join("Roaming")
            .join("npm")
            .join("claude.cmd"),
    ]
}

/// Well-known executables of Claude Code under `home`, most likely first.
#[cfg(not(windows))]
pub(super) fn install_locations(home: &Path) -> Vec<PathBuf> {
    vec![
        // Native installer.
        home.join(".local").join("bin").join("claude"),
        // Older local installs.
        home.join(".claude").join("local").join("claude"),
        // Homebrew (Apple silicon, then Intel) and system-wide npm.
        PathBuf::from("/opt/homebrew/bin/claude"),
        PathBuf::from("/usr/local/bin/claude"),
        // npm with a user prefix.
        home.join(".npm-global").join("bin").join("claude"),
    ]
}
