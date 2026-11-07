//! Pure path arithmetic for media: relative hints (relinking media that moved with their project),
//! the cache folder of an asset's derived files and the names of the files the document records
//! (mezzanines, proxies) or se-app caches itself (extracted frames). No file system access.

use std::path::{Component, Path, PathBuf, Prefix};

use se_domain::{ContentFingerprint, Flicks};
use se_protocol::ProxyResolution;

use crate::ports::Container;

/// The cache folder inside a project folder (ADR-0006): everything in it can be regenerated.
const CACHE_DIR: &str = "cache";
/// Where derived files of all assets live inside the cache folder.
const MEDIA_DIR: &str = "media";
/// Where extracted frames live inside a content's cache folder.
const FRAMES_DIR: &str = "frames";

/// The cache folder of a project: `<project>/cache/`.
#[must_use]
pub fn project_cache_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(CACHE_DIR)
}

/// The cache folder of a content: `<project>/cache/media/<fingerprint cache key>/`.
#[must_use]
pub fn media_cache_dir(project_dir: &Path, fingerprint: &ContentFingerprint) -> PathBuf {
    project_cache_dir(project_dir)
        .join(MEDIA_DIR)
        .join(fingerprint.cache_key())
}

/// The document form (relative to the project folder, `/`-separated) of a file in a content's
/// cache folder: `cache/media/<key>/<file_name>`.
#[must_use]
pub fn cached_file_relative_path(fingerprint: &ContentFingerprint, file_name: &str) -> String {
    format!(
        "{CACHE_DIR}/{MEDIA_DIR}/{}/{file_name}",
        fingerprint.cache_key()
    )
}

/// The absolute path of a document relative path (`/`-separated) inside `project_dir`; `None`
/// for paths that would leave it (`..`, roots: a hand-edited document must not point elsewhere).
#[must_use]
pub fn resolve_relative(project_dir: &Path, relative: &str) -> Option<PathBuf> {
    let mut path = project_dir.to_path_buf();
    for part in relative.split('/') {
        if part.is_empty() || part == "." || part == ".." || Path::new(part).has_root() {
            return None;
        }
        path.push(part);
    }
    Some(path)
}

/// File name of a mezzanine: `mezzanine.mp4`, `mezzanine.m4a`, `mezzanine.png`.
#[must_use]
pub fn mezzanine_file_name(container: Container) -> String {
    format!("mezzanine.{}", container.extension())
}

/// File name of a proxy: `proxy-540.mp4`, `proxy-720.mp4`.
#[must_use]
pub fn proxy_file_name(resolution: ProxyResolution) -> String {
    format!("proxy-{}.mp4", resolution.short_side())
}

/// The cached JPEG of a frame: `<content cache>/frames/<time in flicks>-<max width>.jpg`.
#[must_use]
pub fn frame_file(content_cache_dir: &Path, time: Flicks, max_width: u32) -> PathBuf {
    content_cache_dir
        .join(FRAMES_DIR)
        .join(format!("{}-{max_width}.jpg", time.get()))
}

/// `target` relative to `base`, `/`-separated (`../media/clip.mp4`), or `None` when they share no
/// root (another drive) or a component is not UTF-8. Both paths must be absolute.
#[must_use]
pub fn relative_hint(base: &Path, target: &Path) -> Option<String> {
    let base: Vec<Component<'_>> = base.components().collect();
    let target: Vec<Component<'_>> = target.components().collect();
    if !same_root(base.first()?, target.first()?) {
        return None;
    }
    let common = base
        .iter()
        .zip(&target)
        .skip(1)
        .take_while(|(left, right)| left == right)
        .count()
        + 1;
    let mut parts: Vec<&str> = vec![".."; base.len() - common];
    for component in &target[common..] {
        match component {
            Component::Normal(name) => parts.push(name.to_str()?),
            Component::ParentDir => parts.push(".."),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// The file a relative hint points at from `base`, with `.` and `..` resolved lexically; `None`
/// for hints that are not relative (a hand-edited document must not point anywhere else).
#[must_use]
pub fn resolve_hint(base: &Path, hint: &str) -> Option<PathBuf> {
    let hint = Path::new(hint);
    if hint.has_root() || hint.is_absolute() {
        return None;
    }
    let mut resolved = PathBuf::new();
    for component in base.components().chain(hint.components()) {
        match component {
            Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            other => resolved.push(other.as_os_str()),
        }
    }
    Some(resolved)
}

/// Whether two first components are the same root: `C:` and `\\?\C:` are (canonical project
/// paths are verbatim on Windows, paths from dialogs are not).
fn same_root(left: &Component<'_>, right: &Component<'_>) -> bool {
    match (left, right) {
        (Component::Prefix(left), Component::Prefix(right)) => {
            match (disk(left.kind()), disk(right.kind())) {
                (Some(left), Some(right)) => left.eq_ignore_ascii_case(&right),
                _ => left == right,
            }
        }
        _ => left == right,
    }
}

/// The drive letter of a disk prefix.
fn disk(prefix: Prefix<'_>) -> Option<u8> {
    match prefix {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => Some(letter),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn hints_climb_out_of_the_project_folder() {
        let project = Path::new("/home/me/films/Film.seproj");
        assert_eq!(
            relative_hint(project, Path::new("/home/me/films/rushes/a.mp4")).as_deref(),
            Some("../rushes/a.mp4")
        );
        assert_eq!(
            relative_hint(project, Path::new("/home/me/films/Film.seproj/media/b.wav")).as_deref(),
            Some("media/b.wav")
        );
        assert_eq!(
            relative_hint(project, Path::new("/srv/c.mov")).as_deref(),
            Some("../../../../srv/c.mov")
        );
    }

    #[cfg(windows)]
    #[test]
    fn hints_need_the_same_drive() {
        let project = Path::new(r"\\?\C:\Films\Film.seproj");
        assert_eq!(
            relative_hint(project, Path::new(r"C:\Films\rushes\a.mp4")).as_deref(),
            Some("../rushes/a.mp4")
        );
        assert_eq!(relative_hint(project, Path::new(r"D:\a.mp4")), None);
    }

    #[test]
    fn hints_resolve_from_the_project_folder() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("Film.seproj");
        let resolved = resolve_hint(&project, "../rushes/a.mp4").unwrap();
        assert_eq!(
            resolved,
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("rushes")
                .join("a.mp4")
        );
        assert_eq!(
            relative_hint(&project, &resolved).as_deref(),
            Some("../rushes/a.mp4")
        );
    }

    #[test]
    fn rooted_hints_are_refused() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(resolve_hint(project, "/etc/passwd"), None);
    }

    #[test]
    fn derived_files_are_recorded_relative_to_the_project() {
        let fingerprint = ContentFingerprint::new(10, 20, "00000000000000ff").unwrap();
        let relative =
            cached_file_relative_path(&fingerprint, &mezzanine_file_name(Container::Mp4));
        assert_eq!(relative, "cache/media/00000000000000ff-10-20/mezzanine.mp4");
        let project = Path::new("Film.seproj");
        assert_eq!(
            resolve_relative(project, &relative),
            Some(media_cache_dir(project, &fingerprint).join("mezzanine.mp4"))
        );
        assert_eq!(resolve_relative(project, "../outside.mp4"), None);
        assert_eq!(proxy_file_name(ProxyResolution::P720), "proxy-720.mp4");
        assert_eq!(
            frame_file(project, Flicks::one_second(), 640),
            project.join("frames").join("705600000-640.jpg")
        );
    }

    #[test]
    fn cache_folders_are_named_after_the_content() {
        let fingerprint = ContentFingerprint::new(10, 20, "00000000000000ff").unwrap();
        let dir = media_cache_dir(Path::new("Film.seproj"), &fingerprint);
        assert_eq!(
            dir,
            Path::new("Film.seproj")
                .join("cache")
                .join("media")
                .join("00000000000000ff-10-20")
        );
    }
}
