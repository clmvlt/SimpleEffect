//! Operating-system specifics of the vision adapter: the platform folder, the name of the ONNX
//! Runtime library, the execution providers tried, and the systems without a runtime build.

use ort::ep::ExecutionProviderDispatch;

/// Platform folder of `resources/bin/<platform>/` (`docs/architecture.md` §2).
pub const PLATFORM_DIR: &str = if cfg!(all(windows, target_arch = "x86_64")) {
    "win32-x64"
} else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
    "darwin-arm64"
} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
    "darwin-x64"
} else {
    "linux-x64"
};

/// File name of the ONNX Runtime library in the `onnxruntime/` folder (`pnpm fetch-binaries`).
pub const LIBRARY_FILE: &str = if cfg!(windows) {
    "onnxruntime.dll"
} else if cfg!(target_os = "macos") {
    "libonnxruntime.dylib"
} else {
    "libonnxruntime.so"
};

/// Why this system has no ONNX Runtime build, when it has none.
#[must_use]
pub const fn unsupported_system() -> Option<&'static str> {
    if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some(
            "Microsoft publishes no ONNX Runtime build for Intel Macs, so faces cannot be \
             detected on this Mac; reframe the clips by hand (« Recadrer dans le viewer »)",
        )
    } else {
        None
    }
}

/// The execution providers tried before the CPU, in order: CoreML on macOS (part of
/// Microsoft's macOS builds), none elsewhere (the model takes a few milliseconds on a CPU).
#[must_use]
pub fn execution_providers() -> Vec<ExecutionProviderDispatch> {
    #[cfg(target_os = "macos")]
    {
        vec![ort::ep::CoreML::default().build()]
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

/// The name of where inference runs, shown in the settings.
#[must_use]
pub const fn accelerator() -> &'static str {
    if cfg!(target_os = "macos") {
        "CoreML"
    } else {
        "CPU"
    }
}
