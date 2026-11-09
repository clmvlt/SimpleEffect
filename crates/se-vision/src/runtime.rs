//! Finding and loading the ONNX Runtime library (ADR-0037): nothing is linked at build time; the
//! library of `pnpm fetch-binaries` is loaded the first time faces are detected. The runtime is
//! process-wide, so it is loaded once and its environment kept for every session, with ONNX
//! Runtime's telemetry turned off (golden rule 1).

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

use ort::environment::Environment;

use crate::platform::{LIBRARY_FILE, PLATFORM_DIR, unsupported_system};

/// Where to look for the ONNX Runtime library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLocator {
    exe_dir: Option<PathBuf>,
}

impl RuntimeLocator {
    /// A locator searching `<exe_dir>/onnxruntime/` (packaged: `resources/bin/`) and then
    /// `<ancestor>/resources/bin/<platform>/onnxruntime/` of `exe_dir` (development).
    #[must_use]
    pub const fn new(exe_dir: Option<PathBuf>) -> Self {
        Self { exe_dir }
    }

    /// A locator for the running executable.
    #[must_use]
    pub fn for_current_exe() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        Self::new(exe_dir)
    }

    /// The candidate library files, in search order.
    #[must_use]
    pub fn candidates(&self) -> Vec<PathBuf> {
        let Some(dir) = &self.exe_dir else {
            return Vec::new();
        };
        let mut candidates = vec![dir.join("onnxruntime").join(LIBRARY_FILE)];
        for ancestor in dir.ancestors() {
            candidates.push(
                ancestor
                    .join("resources")
                    .join("bin")
                    .join(PLATFORM_DIR)
                    .join("onnxruntime")
                    .join(LIBRARY_FILE),
            );
        }
        candidates
    }

    /// The first candidate that exists.
    ///
    /// # Errors
    /// Why there is no library: an unsupported system, or the places searched.
    pub fn locate(&self) -> Result<PathBuf, String> {
        if let Some(reason) = unsupported_system() {
            return Err(reason.to_owned());
        }
        let candidates = self.candidates();
        candidates
            .iter()
            .find(|path| path.is_file())
            .cloned()
            .ok_or_else(|| {
                let searched: Vec<String> = candidates
                    .iter()
                    .take(3)
                    .map(|path| path.display().to_string())
                    .collect();
                format!(
                    "the ONNX Runtime library ({LIBRARY_FILE}) was not found (searched {}); \
                     reinstall SimpleEffect, or run `pnpm fetch-binaries` in a development checkout",
                    searched.join(", ")
                )
            })
    }
}

/// The loaded runtime: its environment (kept alive for every session) and what to show.
#[derive(Debug, Clone)]
pub struct LoadedRuntime {
    /// The environment sessions are created in (telemetry off).
    pub environment: Environment,
    /// `ONNX Runtime 1.31.0`, or `ONNX Runtime (build 402979a)` (see `describe`).
    pub description: String,
    /// The library file loaded.
    pub path: PathBuf,
}

/// The process-wide runtime, once loaded.
static RUNTIME: Mutex<Option<Arc<LoadedRuntime>>> = Mutex::new(None);

/// Loads the runtime found by `locator` (once per process; a failure is retried next time).
///
/// # Errors
/// Why the library could not be found or loaded.
pub fn load(locator: &RuntimeLocator) -> Result<Arc<LoadedRuntime>, String> {
    let mut slot = RUNTIME.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(runtime) = slot.as_ref() {
        return Ok(Arc::clone(runtime));
    }
    let path = locator.locate()?;
    let environment = ort::init_from(&path)
        .map_err(|error| {
            format!(
                "the ONNX Runtime library {} could not be loaded: {error}",
                path.display()
            )
        })?
        .with_name("simpleeffect")
        .with_telemetry(false)
        .build()
        .map_err(|error| format!("ONNX Runtime could not start: {error}"))?;
    let runtime = Arc::new(LoadedRuntime {
        environment,
        description: describe(ort::info()),
        path,
    });
    tracing::info!(runtime = %runtime.description, path = %runtime.path.display(), "ONNX Runtime loaded");
    *slot = Some(Arc::clone(&runtime));
    Ok(runtime)
}

/// What to show of the build information: `ONNX Runtime 1.31.0` for a release branch
/// (`git-branch=rel-1.31.0, …`), else `ONNX Runtime (build 402979a)` — Microsoft's official
/// builds say `git-branch=HEAD`, and ort exposes no other version text.
fn describe(info: &str) -> String {
    let field = |prefix: &str| {
        info.split([',', ' '])
            .find_map(|part| part.trim().strip_prefix(prefix).map(str::to_owned))
    };
    if let Some(version) = field("git-branch=rel-") {
        return format!("ONNX Runtime {version}");
    }
    field("git-commit-id=").map_or_else(
        || "ONNX Runtime".to_owned(),
        |commit| format!("ONNX Runtime (build {commit})"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_packaged_folder_comes_first_then_development_resources() {
        let exe_dir = PathBuf::from("repo").join("target").join("debug");
        let candidates = RuntimeLocator::new(Some(exe_dir.clone())).candidates();
        assert_eq!(
            candidates[0],
            exe_dir.join("onnxruntime").join(LIBRARY_FILE)
        );
        let development = PathBuf::from("repo")
            .join("resources")
            .join("bin")
            .join(PLATFORM_DIR)
            .join("onnxruntime")
            .join(LIBRARY_FILE);
        assert!(candidates.contains(&development));
        assert_eq!(
            RuntimeLocator::new(None).candidates(),
            Vec::<PathBuf>::new()
        );
    }

    #[test]
    fn a_missing_library_names_where_it_was_searched() {
        let missing = std::env::temp_dir().join("se-vision-no-runtime-here");
        let error = RuntimeLocator::new(Some(missing)).locate().unwrap_err();
        if unsupported_system().is_none() {
            assert!(error.contains("se-vision-no-runtime-here"), "{error}");
            assert!(error.contains("pnpm fetch-binaries"), "{error}");
        }
    }

    #[test]
    fn the_version_comes_from_the_build_information() {
        assert_eq!(
            describe(
                "ORT Build Info: git-branch=rel-1.31.0, git-commit-id=abc, build type=Release"
            ),
            "ONNX Runtime 1.31.0"
        );
        assert_eq!(describe("something else"), "ONNX Runtime");
    }

    #[test]
    fn official_builds_without_a_release_branch_name_their_commit() {
        // The text of Microsoft's 1.31.0 Windows build.
        assert_eq!(
            describe(
                "ORT Build Info: git-branch=HEAD, git-commit-id=402979a, fp8-kv-cache=1, build type=RelWithDebInfo"
            ),
            "ONNX Runtime (build 402979a)"
        );
    }
}
