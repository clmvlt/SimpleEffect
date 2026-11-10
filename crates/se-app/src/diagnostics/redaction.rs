//! Masking of personal data in diagnostics (prompt 30): the user's home folder becomes `~`,
//! media and project file names become hashes, secrets and e-mail addresses are removed.
//! Applied to every text of the archive (report and logs), whatever produced it.

use std::path::Path;

/// Extensions whose file names are hashed: media, projects, subtitles, templates, exports.
const MASKED_EXTENSIONS: &[&str] = &[
    "seproj",
    "json",
    "mp4",
    "mov",
    "mkv",
    "webm",
    "avi",
    "m4v",
    "mxf",
    "mts",
    "m2ts",
    "mpg",
    "mpeg",
    "wmv",
    "flv",
    "3gp",
    "mp3",
    "wav",
    "aac",
    "m4a",
    "flac",
    "ogg",
    "opus",
    "aif",
    "aiff",
    "wma",
    "png",
    "jpg",
    "jpeg",
    "gif",
    "webp",
    "heic",
    "tif",
    "tiff",
    "bmp",
    "svg",
    "srt",
    "vtt",
    "setemplate",
    "xml",
    "otio",
    "zip",
    "bin",
];

/// Query parameters and prefixes whose value is a secret.
const SECRET_KEYS: &[&str] = &["t=", "token=", "Bearer ", "bearer "];

/// A run of this many token characters (letters, digits, `+/=_`) without a `-` is taken for a
/// key or token: UUIDs (with dashes) stay readable for debugging.
const SECRET_RUN: usize = 32;

/// Masks personal data in texts.
#[derive(Debug, Clone)]
pub struct Redactor {
    /// Spellings of the home folder (as given, with `/`, with `\`), longest first.
    home_spellings: Vec<String>,
}

impl Redactor {
    /// A redactor for the user whose home folder is `home` (none: nothing is replaced by `~`).
    pub fn new(home: Option<&Path>) -> Self {
        let mut home_spellings = Vec::new();
        if let Some(home) = home.and_then(Path::to_str).filter(|home| home.len() > 1) {
            let trimmed = home.trim_end_matches(['/', '\\']);
            for spelling in [
                trimmed.to_owned(),
                trimmed.replace('\\', "/"),
                trimmed.replace('/', "\\"),
                trimmed.replace('\\', "\\\\"),
            ] {
                if !home_spellings.contains(&spelling) {
                    home_spellings.push(spelling);
                }
            }
        }
        home_spellings.sort_by_key(|spelling| std::cmp::Reverse(spelling.len()));
        Self { home_spellings }
    }

    /// `text` with the home folder, file names, secrets and e-mail addresses masked.
    #[must_use]
    pub fn redact(&self, text: &str) -> String {
        let mut text = text.to_owned();
        for spelling in &self.home_spellings {
            text = replace_ignoring_case(&text, spelling, "~");
        }
        let text = mask_secrets(&text);
        let text = mask_emails(&text);
        mask_file_names(&text)
    }
}

/// `text` with every case-insensitive occurrence of `needle` replaced (Windows paths ignore case).
fn replace_ignoring_case(text: &str, needle: &str, replacement: &str) -> String {
    let lower_text = text.to_lowercase();
    let lower_needle = needle.to_lowercase();
    // Lowercasing may change byte lengths outside ASCII: fall back to exact matching then.
    if lower_text.len() != text.len() || lower_needle.len() != needle.len() {
        return text.replace(needle, replacement);
    }
    let mut result = String::with_capacity(text.len());
    let mut position = 0;
    while let Some(found) = lower_text[position..].find(&lower_needle) {
        let start = position + found;
        result.push_str(&text[position..start]);
        result.push_str(replacement);
        position = start + needle.len();
    }
    result.push_str(&text[position..]);
    result
}

fn is_token_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '+' | '/' | '=' | '_')
}

/// Values after secret keys, and long runs of token characters, become `<redacted>`.
fn mask_secrets(text: &str) -> String {
    let mut text = text.to_owned();
    for key in SECRET_KEYS {
        let mut result = String::with_capacity(text.len());
        let mut rest = text.as_str();
        while let Some(found) = rest.find(key) {
            let boundary = found == 0 || !rest[..found].ends_with(|c: char| c.is_alphanumeric());
            let value_start = found + key.len();
            result.push_str(&rest[..value_start]);
            rest = &rest[value_start..];
            let value_len = rest
                .find(|c: char| !(is_token_char(c) || c == '-' || c == '.'))
                .unwrap_or(rest.len());
            if boundary && value_len > 0 {
                result.push_str("<redacted>");
                rest = &rest[value_len..];
            }
        }
        result.push_str(rest);
        text = result;
    }
    mask_long_runs(&text)
}

fn mask_long_runs(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut run = String::new();
    let flush = |run: &mut String, result: &mut String| {
        let has_digit = run.chars().any(|c| c.is_ascii_digit());
        let has_letter = run.chars().any(|c| c.is_ascii_alphabetic());
        if run.len() >= SECRET_RUN && has_digit && has_letter {
            result.push_str("<redacted>");
        } else {
            result.push_str(run);
        }
        run.clear();
    };
    for character in text.chars() {
        if is_token_char(character) && character != '/' {
            run.push(character);
        } else {
            flush(&mut run, &mut result);
            result.push(character);
        }
    }
    flush(&mut run, &mut result);
    result
}

/// `name@domain.tld` becomes `<email>`.
fn mask_emails(text: &str) -> String {
    let is_local = |c: char| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+');
    let is_domain = |c: char| c.is_alphanumeric() || matches!(c, '.' | '-');
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        let local_start = rest[..at].rfind(|c: char| !is_local(c)).map_or(0, |index| {
            index + rest[index..].chars().next().map_or(1, char::len_utf8)
        });
        let domain_len = rest[at + 1..]
            .find(|c: char| !is_domain(c))
            .unwrap_or(rest.len() - at - 1);
        let domain = rest[at + 1..at + 1 + domain_len].trim_end_matches('.');
        if local_start < at && domain.contains('.') {
            result.push_str(&rest[..local_start]);
            result.push_str("<email>");
            rest = &rest[at + 1 + domain.len()..];
        } else {
            result.push_str(&rest[..=at]);
            rest = &rest[at + 1..];
        }
    }
    result.push_str(rest);
    result
}

/// Characters ending a file name when walking back from its extension.
fn ends_name(character: char) -> bool {
    matches!(
        character,
        '/' | '\\' | '"' | '\'' | '`' | '\n' | '\t' | '=' | '(' | '[' | '<' | '>'
    )
}

/// Every `name.ext` with a masked extension becomes `<hash>.ext`. The name runs back to the
/// previous path separator or quote; without one on its line, only its last word is taken.
fn mask_file_names(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut copied = 0;
    let lower = text.to_ascii_lowercase();
    let mut search = 0;
    while let Some(found) = lower[search..].find('.') {
        let dot = search + found;
        search = dot + 1;
        let Some(extension) = masked_extension_at(&lower, dot) else {
            continue;
        };
        let name_start = name_start(text, copied, dot);
        if name_start >= dot {
            continue;
        }
        result.push_str(&text[copied..name_start]);
        result.push_str(&short_hash(&text[name_start..dot]));
        copied = dot;
        search = dot + 1 + extension.len();
    }
    result.push_str(&text[copied..]);
    result
}

/// The masked extension right after the dot at `dot`, when it ends there.
fn masked_extension_at(lower: &str, dot: usize) -> Option<&'static str> {
    let after = &lower[dot + 1..];
    MASKED_EXTENSIONS.iter().copied().find(|extension| {
        after.starts_with(extension)
            && !after[extension.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn name_start(text: &str, floor: usize, dot: usize) -> usize {
    let before = &text[floor..dot];
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    let line = &before[line_start..];
    let start = match line.rfind(ends_name) {
        Some(index) => index + line[index..].chars().next().map_or(1, char::len_utf8),
        None => line.rfind(char::is_whitespace).map_or(0, |index| {
            index + line[index..].chars().next().map_or(1, char::len_utf8)
        }),
    };
    floor + line_start + start
}

/// A short, stable, non-reversible-at-a-glance name: FNV-1a 64, 12 hex digits.
fn short_hash(name: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{:012x}", hash >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redactor() -> Redactor {
        Redactor::new(Some(Path::new("C:\\Users\\Marie Dupont")))
    }

    #[test]
    fn the_home_folder_becomes_a_tilde_in_every_spelling() {
        let text = "C:\\Users\\Marie Dupont\\Videos and c:/users/marie dupont/Documents \
                    and C:\\\\Users\\\\Marie Dupont\\\\x";
        let redacted = redactor().redact(text);
        assert!(!redacted.to_lowercase().contains("marie"), "{redacted}");
        assert!(redacted.starts_with("~\\Videos"), "{redacted}");
    }

    #[test]
    fn media_and_project_names_are_hashed_with_their_extension_kept() {
        let text = "imported \"D:\\Rushes\\Mariage Julie.MP4\" into ~/Films/Client Acme.seproj";
        let redacted = redactor().redact(text);
        assert!(!redacted.contains("Julie"), "{redacted}");
        assert!(!redacted.contains("Acme"), "{redacted}");
        assert!(redacted.contains(".MP4"), "{redacted}");
        assert!(redacted.contains(".seproj"), "{redacted}");
        assert!(
            redacted.starts_with("imported \"D:\\Rushes\\"),
            "{redacted}"
        );
    }

    #[test]
    fn a_bare_file_name_loses_only_its_last_word() {
        let redacted = redactor().redact("probing interview.wav failed");
        assert!(redacted.starts_with("probing "), "{redacted}");
        assert!(!redacted.contains("interview"), "{redacted}");
        assert!(redacted.ends_with(".wav failed"), "{redacted}");
    }

    #[test]
    fn names_hash_the_same_way_every_time() {
        let first = redactor().redact("/a/clip.mov");
        assert_eq!(first, redactor().redact("/a/clip.mov"));
        assert_ne!(first, redactor().redact("/a/other.mov"));
    }

    #[test]
    fn secrets_and_emails_are_removed_but_ids_stay() {
        let token = "a".repeat(20) + &"7".repeat(30);
        let text = format!(
            "ws://127.0.0.1:5000/rpc?t={token} Bearer abc.def-1 key {token} \
             by marie.dupont@example.fr item 0190f3a2-7c1b-7000-8000-00000000abcd"
        );
        let redacted = redactor().redact(&text);
        assert!(!redacted.contains(&token), "{redacted}");
        assert!(redacted.contains("t=<redacted>"), "{redacted}");
        assert!(redacted.contains("Bearer <redacted>"), "{redacted}");
        assert!(redacted.contains("<email>"), "{redacted}");
        assert!(!redacted.contains("dupont"), "{redacted}");
        assert!(
            redacted.contains("0190f3a2-7c1b-7000-8000-00000000abcd"),
            "{redacted}"
        );
    }

    #[test]
    fn ordinary_text_is_left_alone() {
        let text = "INFO se_core: slow command kind=\"importMedia\" elapsed_ms=312.5 v1.2.3";
        assert_eq!(redactor().redact(text), text);
    }
}
