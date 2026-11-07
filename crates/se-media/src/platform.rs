//! Everything that differs between operating systems (golden rule 10): binary folder names,
//! executable names, how a sidecar process is isolated so its whole tree can be stopped, and
//! which hardware encoders exist.

use process_wrap::tokio::CommandWrap;

use crate::encoders::HardwareEncoder;

/// Hardware encoder families tried on this OS, in order of preference.
pub const HARDWARE_ENCODER_CANDIDATES: &[HardwareEncoder] = if cfg!(target_os = "macos") {
    &[HardwareEncoder::VideoToolbox]
} else if cfg!(windows) {
    &[HardwareEncoder::Nvenc]
} else {
    &[HardwareEncoder::Nvenc, HardwareEncoder::Vaapi]
};

/// The VA-API render device (Linux; the first GPU).
pub const VAAPI_DEVICE: &str = "/dev/dri/renderD128";

/// Folder of this platform's binaries under `resources/bin/` (`docs/architecture.md` §2).
pub const PLATFORM_DIR: &str = if cfg!(all(windows, target_arch = "x86_64")) {
    "win32-x64"
} else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
    "darwin-arm64"
} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
    "darwin-x64"
} else {
    "linux-x64"
};

/// File name of a tool's executable (`ffmpeg.exe` on Windows).
#[must_use]
pub fn executable_name(tool: &str) -> String {
    if cfg!(windows) {
        format!("{tool}.exe")
    } else {
        tool.to_owned()
    }
}

/// Puts the process in a job object (Windows: killing the job kills the tree; no console window
/// opens) or a new process group (Unix: killing the group kills the tree).
#[cfg(windows)]
pub fn isolate(command: &mut CommandWrap) {
    use process_wrap::tokio::{CreationFlags, JobObject, KillOnDrop};
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;
    command
        .wrap(CreationFlags(CREATE_NO_WINDOW))
        .wrap(JobObject)
        .wrap(KillOnDrop);
}

/// Puts the process in a job object (Windows: killing the job kills the tree; no console window
/// opens) or a new process group (Unix: killing the group kills the tree).
#[cfg(not(windows))]
pub fn isolate(command: &mut CommandWrap) {
    use process_wrap::tokio::{KillOnDrop, ProcessGroup};
    command.wrap(ProcessGroup::leader()).wrap(KillOnDrop);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executables_have_the_platform_suffix() {
        let name = executable_name("ffprobe");
        assert_eq!(name == "ffprobe.exe", cfg!(windows));
        assert!(name.starts_with("ffprobe"));
    }
}
