//! OS specifics of transcription: executable names, process isolation, GPU detection, the GPU
//! pack each platform offers and how a missing system library shows.

use std::path::Path;

use process_wrap::tokio::CommandWrap;
use se_protocol::TranscriptionBackend;

/// Folder of this platform under `resources/bin/` (same keys as `pnpm fetch-binaries`).
pub const PLATFORM_DIR: &str = if cfg!(all(windows, target_arch = "x86_64")) {
    "win32-x64"
} else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
    "darwin-arm64"
} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
    "darwin-x64"
} else {
    "linux-x64"
};

/// `whisper-cli` with this platform's executable suffix.
#[must_use]
pub fn whisper_executable() -> &'static str {
    if cfg!(windows) {
        "whisper-cli.exe"
    } else {
        "whisper-cli"
    }
}

/// Puts the process in a job object (Windows: killing the job kills the tree; no console window)
/// or a new process group (Unix).
#[cfg(windows)]
pub fn isolate(command: &mut CommandWrap) {
    use process_wrap::tokio::{CreationFlags, JobObject, KillOnDrop};
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;
    command
        .wrap(CreationFlags(CREATE_NO_WINDOW))
        .wrap(JobObject)
        .wrap(KillOnDrop);
}

/// Puts the process in a job object (Windows: killing the job kills the tree; no console window)
/// or a new process group (Unix).
#[cfg(not(windows))]
pub fn isolate(command: &mut CommandWrap) {
    use process_wrap::tokio::{KillOnDrop, ProcessGroup};
    command.wrap(ProcessGroup::leader()).wrap(KillOnDrop);
}

/// Whether an exit code or a log says a system library is missing (the program never ran).
#[must_use]
pub fn missing_library(code: Option<i32>, log: &str) -> bool {
    // STATUS_DLL_NOT_FOUND (0xC0000135) and STATUS_INVALID_IMAGE_FORMAT (0xC000007B).
    const WINDOWS_CODES: [u32; 2] = [0xC000_0135, 0xC000_007B];
    let windows = code.is_some_and(|code| WINDOWS_CODES.contains(&code.cast_unsigned()));
    windows || log.contains("error while loading shared libraries")
}

/// What to install when a system library is missing.
pub const MISSING_LIBRARY_ADVICE: &str = if cfg!(windows) {
    "a system library is missing: install the « Microsoft Visual C++ Redistributable 2015–2022 \
     (x64) » from Microsoft, then try again"
} else {
    "a system library is missing (see the log): install your distribution's C++ runtime, then \
     try again"
};

/// The GPU pack of a platform: the official CUDA build of whisper.cpp on Windows x64.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuPack {
    /// Shown to the user.
    pub name: &'static str,
    /// Pinned release asset.
    pub url: &'static str,
    /// Its SHA-256 (GitHub's asset digest).
    pub sha256: &'static str,
    /// Its size.
    pub size: u64,
    /// Folder of the installed pack under `<app data>/runtimes/`.
    pub dir_name: &'static str,
    /// Folder of the files inside the archive.
    pub archive_dir: &'static str,
    /// The files kept (the program, its libraries, ggml's CPU and CUDA backends, CUDA's runtime).
    pub files: &'static [&'static str],
}

/// The pack of this platform, if any.
#[must_use]
pub fn gpu_pack() -> Option<GpuPack> {
    cfg!(all(windows, target_arch = "x86_64")).then_some(WINDOWS_CUDA)
}

const WINDOWS_CUDA: GpuPack = GpuPack {
    name: "CUDA 12.4",
    url: "https://github.com/ggml-org/whisper.cpp/releases/download/b5454/whisper-bin-win-cuda-12.4.0-x64.zip",
    sha256: "afef0b881c500958921c3f5523b50e59ee2ec9b6f5cbd25b324c51ed308a957a",
    size: 684_913_404,
    dir_name: "whisper-cuda-12.4",
    archive_dir: "Release",
    files: &[
        "whisper-cli.exe",
        "whisper.dll",
        "ggml.dll",
        "ggml-base.dll",
        "ggml-cpu-alderlake.dll",
        "ggml-cpu-cannonlake.dll",
        "ggml-cpu-cascadelake.dll",
        "ggml-cpu-haswell.dll",
        "ggml-cpu-icelake.dll",
        "ggml-cpu-sandybridge.dll",
        "ggml-cpu-skylakex.dll",
        "ggml-cpu-sse42.dll",
        "ggml-cpu-x64.dll",
        "ggml-cuda.dll",
        "cublas64_12.dll",
        "cublasLt64_12.dll",
        "cudart64_12.dll",
    ],
};

/// Where the bundled whisper.cpp computes on this platform when the GPU is allowed: Metal on
/// Apple Silicon (built with it), the processor elsewhere (CUDA needs the GPU pack).
#[must_use]
pub fn bundled_backend() -> TranscriptionBackend {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        TranscriptionBackend::Metal
    } else {
        TranscriptionBackend::Cpu
    }
}

/// The program and arguments that name this machine's GPU, if the platform has one to ask.
#[must_use]
pub fn gpu_query() -> Option<(&'static str, &'static [&'static str])> {
    if cfg!(target_os = "macos") {
        cfg!(target_arch = "aarch64")
            .then_some(("sysctl", ["-n", "machdep.cpu.brand_string"].as_slice()))
    } else {
        Some((
            "nvidia-smi",
            ["--query-gpu=name", "--format=csv,noheader"].as_slice(),
        ))
    }
}

/// The archive tool unpacking the GPU pack (Windows 10+ ships bsdtar, which reads zip files).
#[must_use]
pub fn archive_tool() -> std::path::PathBuf {
    std::env::var_os("SystemRoot")
        .map_or_else(|| Path::new("C:\\Windows").to_path_buf(), Into::into)
        .join("System32")
        .join("tar.exe")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_missing_libraries() {
        assert!(missing_library(Some(-1_073_741_515), ""));
        assert!(missing_library(
            Some(1),
            "whisper-cli: error while loading shared libraries: x"
        ));
        assert!(!missing_library(Some(1), "failed to open model"));
        let extension = Path::new(whisper_executable()).extension();
        assert_eq!(
            extension.is_some_and(|extension| extension == "exe"),
            cfg!(windows)
        );
    }

    #[test]
    fn the_windows_pack_keeps_the_program_and_its_cuda_libraries() {
        let files = WINDOWS_CUDA.files;
        for name in [
            "whisper-cli.exe",
            "ggml-cuda.dll",
            "cublasLt64_12.dll",
            "cudart64_12.dll",
        ] {
            assert!(files.contains(&name), "{name}");
        }
        assert_eq!(
            gpu_pack().is_some(),
            cfg!(all(windows, target_arch = "x86_64"))
        );
    }
}
