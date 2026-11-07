//! Reading what `claude --version` and `claude auth status --json` print. Only the fields below
//! are read: the e-mail address and organisation the CLI also prints are never parsed, logged
//! nor stored.

use se_app::ports::AgentError;
use se_protocol::AuthStatus;

/// The version in `claude --version` output (`2.1.317 (Claude Code)` → `2.1.317`).
pub(super) fn parse_version(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .find(|word| {
            word.chars()
                .next()
                .is_some_and(|first| first.is_ascii_digit())
                && word
                    .chars()
                    .all(|character| character.is_ascii_digit() || character == '.')
        })
        .map(str::to_owned)
}

/// The login state in `claude auth status --json` output: `loggedIn`, and the account kind from
/// `subscriptionType` (`max`, `pro`…) or else `authMethod` (`claude.ai`, `api_key`…).
///
/// # Errors
/// [`AgentError::Failed`] when the output is not the expected JSON.
pub(super) fn parse_auth_status(output: &str) -> Result<AuthStatus, AgentError> {
    let json: serde_json::Value =
        serde_json::from_str(output.trim()).map_err(|error| AgentError::Failed {
            message: format!(
                "`claude auth status --json` printed something unexpected ({error}): update \
                 Claude Code or report it"
            ),
        })?;
    let logged_in = json["loggedIn"]
        .as_bool()
        .ok_or_else(|| AgentError::Failed {
            message:
                "`claude auth status --json` did not say whether it is logged in: update Claude \
                  Code or report it"
                    .to_owned(),
        })?;
    if !logged_in {
        return Ok(AuthStatus::LoggedOut);
    }
    let text = |field: &str| {
        json[field]
            .as_str()
            .filter(|value| !value.is_empty() && *value != "none")
            .map(str::to_owned)
    };
    Ok(AuthStatus::LoggedIn {
        account_kind: text("subscriptionType").or_else(|| text("authMethod")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_read_from_the_first_numeric_word() {
        assert_eq!(
            parse_version("2.1.317 (Claude Code)\n").as_deref(),
            Some("2.1.317")
        );
        assert_eq!(parse_version("claude 3.0.1").as_deref(), Some("3.0.1"));
        assert_eq!(parse_version("Claude Code"), None);
    }

    #[test]
    fn logged_out_and_logged_in_answers_are_read() {
        let logged_out = r#"{"loggedIn":false,"authMethod":"none","apiProvider":"firstParty"}"#;
        assert_eq!(
            parse_auth_status(logged_out).unwrap(),
            AuthStatus::LoggedOut
        );
        let subscriber = r#"{"loggedIn":true,"authMethod":"claude.ai","subscriptionType":"max","email":"x@y.z","orgName":"Org"}"#;
        assert_eq!(
            parse_auth_status(subscriber).unwrap(),
            AuthStatus::LoggedIn {
                account_kind: Some("max".to_owned())
            }
        );
        let key = r#"{"loggedIn":true,"authMethod":"api_key","subscriptionType":null}"#;
        assert_eq!(
            parse_auth_status(key).unwrap(),
            AuthStatus::LoggedIn {
                account_kind: Some("api_key".to_owned())
            }
        );
    }

    #[test]
    fn unexpected_answers_are_errors() {
        assert!(parse_auth_status("Logged in").is_err());
        assert!(parse_auth_status(r#"{"status":"ok"}"#).is_err());
    }
}
