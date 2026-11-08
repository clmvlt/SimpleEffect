//! Snapshots of every tool definition (name, description, input schema, annotations): the
//! contract with agents, documented in `docs/ai-tools.md`. A changed snapshot is a changed
//! contract: update the catalogue, then accept it (`INSTA_UPDATE=always cargo test -p se-mcp`).

#![allow(clippy::expect_used, clippy::unwrap_used)]

use se_mcp::tools::catalogue;

#[test]
fn tool_definitions_are_stable() {
    for tool in catalogue() {
        let definition = serde_json::to_value(&tool.definition).unwrap();
        insta::assert_json_snapshot!(tool.name().to_owned(), definition);
    }
}

#[test]
fn the_catalogue_is_documented() {
    let catalogue_doc = include_str!("../../../docs/ai-tools.md");
    for tool in catalogue() {
        assert!(
            catalogue_doc.contains(&format!("### `{}`", tool.name())),
            "docs/ai-tools.md has no section for {}",
            tool.name()
        );
    }
}
