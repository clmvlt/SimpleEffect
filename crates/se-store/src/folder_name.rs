//! Folder names derived from project names, valid on every supported file system.

use crate::layout::PROJECT_EXTENSION;

/// Characters Windows refuses in file names (also covers `/`, the Unix separator).
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Device names Windows reserves, whatever the extension.
const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Longest folder stem kept, in characters (paths stay well under Windows' limits).
const MAX_STEM_CHARS: usize = 120;

/// Stem used when nothing usable is left of the name.
const FALLBACK_STEM: &str = "Project";

/// `<stem>.seproj` for `project_name`, `<stem> <number>.seproj` from `number` 2 on.
///
/// Forbidden and control characters become `-`; surrounding spaces and trailing dots are
/// dropped (Windows strips them); reserved device names get a `_` suffix.
pub fn project_folder_name(project_name: &str, number: u32) -> String {
    let stem = folder_stem(project_name);
    if number <= 1 {
        format!("{stem}.{PROJECT_EXTENSION}")
    } else {
        format!("{stem} {number}.{PROJECT_EXTENSION}")
    }
}

fn folder_stem(project_name: &str) -> String {
    let replaced: String = project_name
        .chars()
        .map(|character| {
            if character.is_control() || FORBIDDEN.contains(&character) {
                '-'
            } else {
                character
            }
        })
        .take(MAX_STEM_CHARS)
        .collect();
    let trimmed = replaced.trim().trim_end_matches(['.', ' ']);
    if trimmed.is_empty() {
        return FALLBACK_STEM.to_owned();
    }
    if RESERVED
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(trimmed))
    {
        return format!("{trimmed}_");
    }
    trimmed.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_ordinary_names_and_numbers_copies() {
        assert_eq!(project_folder_name("Mon film", 1), "Mon film.seproj");
        assert_eq!(project_folder_name("Mon film", 2), "Mon film 2.seproj");
        assert_eq!(project_folder_name("Été à Paris", 1), "Été à Paris.seproj");
    }

    #[test]
    fn replaces_forbidden_characters_and_trims() {
        assert_eq!(project_folder_name(" a/b:c?. ", 1), "a-b-c-.seproj");
        assert_eq!(project_folder_name("tab\there", 1), "tab-here.seproj");
        assert_eq!(project_folder_name("Fin...", 1), "Fin.seproj");
    }

    #[test]
    fn avoids_empty_and_reserved_names() {
        assert_eq!(project_folder_name("   ", 1), "Project.seproj");
        assert_eq!(project_folder_name("con", 1), "con_.seproj");
    }

    #[test]
    fn limits_the_length() {
        let long = "x".repeat(500);
        assert_eq!(
            project_folder_name(&long, 1).len(),
            MAX_STEM_CHARS + ".seproj".len()
        );
    }
}
