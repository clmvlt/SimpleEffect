//! Checking what users give a library entry: name, description, tags.

use se_domain::{MAX_TEMPLATE_DESCRIPTION_CHARS, MAX_TEMPLATE_NAME_CHARS};
use se_protocol::LibraryTemplateInfo;

use crate::AppError;

/// Most tags an entry keeps.
pub const MAX_TAGS: usize = 16;
/// Longest tag, in characters.
pub const MAX_TAG_CHARS: usize = 32;

/// `info` with a trimmed name and description and clean tags (trimmed, lower case, without
/// duplicates or empty ones), or why it cannot be saved.
///
/// # Errors
/// [`AppError::InvalidLibraryEntry`] naming each faulty field.
pub fn checked_info(info: &LibraryTemplateInfo) -> Result<LibraryTemplateInfo, AppError> {
    let name = info.name.trim().to_owned();
    let description = info.description.trim().to_owned();
    let mut tags: Vec<String> = Vec::new();
    for tag in &info.tags {
        let tag = tag.trim().to_lowercase();
        if !tag.is_empty() && !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    let mut problems = Vec::new();
    let name_chars = name.chars().count();
    if !(1..=MAX_TEMPLATE_NAME_CHARS).contains(&name_chars) {
        problems.push(format!(
            "the name must hold 1 to {MAX_TEMPLATE_NAME_CHARS} characters"
        ));
    }
    if description.chars().count() > MAX_TEMPLATE_DESCRIPTION_CHARS {
        problems.push(format!(
            "the description must hold at most {MAX_TEMPLATE_DESCRIPTION_CHARS} characters"
        ));
    }
    if tags.len() > MAX_TAGS {
        problems.push(format!("keep at most {MAX_TAGS} tags"));
    }
    if let Some(long) = tags.iter().find(|tag| tag.chars().count() > MAX_TAG_CHARS) {
        problems.push(format!(
            "tag '{long}' is longer than {MAX_TAG_CHARS} characters"
        ));
    }
    if problems.is_empty() {
        Ok(LibraryTemplateInfo {
            name,
            description,
            tags,
            category: info.category,
        })
    } else {
        Err(AppError::InvalidLibraryEntry {
            reason: format!("the library entry cannot be saved: {}", problems.join("; ")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(name: &str, tags: &[&str]) -> LibraryTemplateInfo {
        LibraryTemplateInfo {
            name: name.to_owned(),
            description: "  Un titre  ".to_owned(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            category: None,
        }
    }

    #[test]
    fn names_and_tags_are_cleaned() {
        let checked =
            checked_info(&info("  Titre  ", &["Shorts", " shorts", "", "Intro"])).unwrap();
        assert_eq!(checked.name, "Titre");
        assert_eq!(checked.description, "Un titre");
        assert_eq!(checked.tags, ["shorts", "intro"]);
    }

    #[test]
    fn empty_names_and_long_tags_are_refused() {
        let error = checked_info(&info(" ", &[&"x".repeat(40)])).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("name must hold"), "{message}");
        assert!(message.contains("longer than 32"), "{message}");
    }
}
