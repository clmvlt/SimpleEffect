//! AI settings, kept as `library.db` settings: reading with defaults, validating and saving.

use std::path::Path;

use se_protocol::{AiSettings, EffortLevel, ProviderCapabilities, ProviderId};

use crate::{AppError, ports::AppLibraryRepository};

/// Keys of the AI settings in `library.db`.
pub mod keys {
    /// Id of the provider running tasks.
    pub const PROVIDER: &str = "ai.provider";
    /// Absolute path of the CLI chosen by the user (empty: search it).
    pub const EXECUTABLE_PATH: &str = "ai.executablePath";
    /// Model of new tasks (empty: the CLI decides).
    pub const DEFAULT_MODEL: &str = "ai.defaultModel";
    /// Effort of new tasks (empty: the CLI decides).
    pub const DEFAULT_EFFORT: &str = "ai.defaultEffort";
    /// Tasks running at once.
    pub const MAX_CONCURRENT_TASKS: &str = "ai.maxConcurrentTasks";
}

/// Longest model name accepted.
const MAX_MODEL_LENGTH: usize = 100;

/// The stored settings, with defaults for values never set or unreadable; `default_provider` when
/// none was chosen.
pub(super) async fn load(
    library: &dyn AppLibraryRepository,
    default_provider: &ProviderId,
) -> Result<AiSettings, AppError> {
    let text = |value: Option<String>| value.filter(|value| !value.is_empty());
    let provider = text(library.setting(keys::PROVIDER).await?)
        .map_or_else(|| default_provider.clone(), ProviderId);
    let max_concurrent_tasks = library
        .setting(keys::MAX_CONCURRENT_TASKS)
        .await?
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|count| AiSettings::MAX_CONCURRENT_TASKS_RANGE.contains(count))
        .unwrap_or(AiSettings::DEFAULT_MAX_CONCURRENT_TASKS);
    Ok(AiSettings {
        provider,
        executable_path: text(library.setting(keys::EXECUTABLE_PATH).await?),
        default_model: text(library.setting(keys::DEFAULT_MODEL).await?),
        default_effort: text(library.setting(keys::DEFAULT_EFFORT).await?)
            .and_then(|value| parse_effort(&value)),
        max_concurrent_tasks,
    })
}

/// Saves validated settings.
pub(super) async fn save(
    library: &dyn AppLibraryRepository,
    settings: &AiSettings,
) -> Result<(), AppError> {
    let effort = settings.default_effort.map(effort_name).unwrap_or_default();
    library
        .set_setting(keys::PROVIDER, settings.provider.as_str())
        .await?;
    library
        .set_setting(
            keys::EXECUTABLE_PATH,
            settings.executable_path.as_deref().unwrap_or_default(),
        )
        .await?;
    library
        .set_setting(
            keys::DEFAULT_MODEL,
            settings.default_model.as_deref().unwrap_or_default(),
        )
        .await?;
    library.set_setting(keys::DEFAULT_EFFORT, effort).await?;
    library
        .set_setting(
            keys::MAX_CONCURRENT_TASKS,
            &settings.max_concurrent_tasks.to_string(),
        )
        .await?;
    Ok(())
}

/// `settings` cleaned (blank texts become `None`) and checked against what the provider supports.
pub(super) fn validated(
    settings: AiSettings,
    capabilities: &ProviderCapabilities,
) -> Result<AiSettings, AppError> {
    let range = AiSettings::MAX_CONCURRENT_TASKS_RANGE;
    if !range.contains(&settings.max_concurrent_tasks) {
        return Err(AppError::InvalidSetting {
            name: "maxConcurrentTasks",
            reason: format!("choose from {} to {} tasks", range.start(), range.end()),
        });
    }
    let executable_path = blank_to_none(settings.executable_path);
    if executable_path
        .as_deref()
        .is_some_and(|path| !Path::new(path).is_absolute())
    {
        return Err(AppError::InvalidSetting {
            name: "executablePath",
            reason: "give the full path of the CLI executable (or leave it empty to search it)"
                .to_owned(),
        });
    }
    let default_model = blank_to_none(settings.default_model);
    if default_model
        .as_deref()
        .is_some_and(|model| model.len() > MAX_MODEL_LENGTH || model.contains(char::is_whitespace))
    {
        return Err(AppError::InvalidSetting {
            name: "defaultModel",
            reason: "pick one of the listed models, or a model name without spaces".to_owned(),
        });
    }
    if let Some(effort) = settings.default_effort
        && !capabilities.effort_levels.contains(&effort)
    {
        return Err(AppError::InvalidSetting {
            name: "defaultEffort",
            reason: format!(
                "the provider does not support the {} effort level",
                effort_name(effort)
            ),
        });
    }
    Ok(AiSettings {
        executable_path,
        default_model,
        ..settings
    })
}

fn blank_to_none(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

/// The wire name of an effort level (`xhigh`).
const fn effort_name(effort: EffortLevel) -> &'static str {
    match effort {
        EffortLevel::Low => "low",
        EffortLevel::Medium => "medium",
        EffortLevel::High => "high",
        EffortLevel::XHigh => "xhigh",
        EffortLevel::Max => "max",
    }
}

fn parse_effort(text: &str) -> Option<EffortLevel> {
    [
        EffortLevel::Low,
        EffortLevel::Medium,
        EffortLevel::High,
        EffortLevel::XHigh,
        EffortLevel::Max,
    ]
    .into_iter()
    .find(|effort| effort_name(*effort) == text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::InMemoryAppLibrary;

    fn capabilities() -> ProviderCapabilities {
        ProviderCapabilities {
            resume_sessions: true,
            image_input: false,
            models: Vec::new(),
            effort_levels: vec![EffortLevel::Low, EffortLevel::High],
        }
    }

    fn settings() -> AiSettings {
        AiSettings {
            provider: ProviderId("fake".to_owned()),
            executable_path: Some("  ".to_owned()),
            default_model: Some(" opus ".to_owned()),
            default_effort: Some(EffortLevel::High),
            max_concurrent_tasks: 4,
        }
    }

    #[tokio::test]
    async fn settings_round_trip_with_defaults() {
        let library = InMemoryAppLibrary::default();
        let provider = ProviderId("fake".to_owned());
        let defaults = load(&library, &provider).await.unwrap();
        assert_eq!(defaults.max_concurrent_tasks, 3);
        assert_eq!(defaults.default_effort, None);
        let saved = validated(settings(), &capabilities()).unwrap();
        save(&library, &saved).await.unwrap();
        let loaded = load(&library, &provider).await.unwrap();
        assert_eq!(loaded, saved);
        assert_eq!(loaded.executable_path, None);
        assert_eq!(loaded.default_model.as_deref(), Some("opus"));
    }

    #[test]
    fn invalid_settings_are_refused_with_their_name() {
        let refused = |settings: AiSettings| match validated(settings, &capabilities()) {
            Err(AppError::InvalidSetting { name, .. }) => name,
            other => panic!("expected an invalid setting, got {other:?}"),
        };
        let mut too_many = settings();
        too_many.max_concurrent_tasks = 9;
        assert_eq!(refused(too_many), "maxConcurrentTasks");
        let mut relative = settings();
        relative.executable_path = Some("bin/claude".to_owned());
        assert_eq!(refused(relative), "executablePath");
        let mut unsupported = settings();
        unsupported.default_effort = Some(EffortLevel::Max);
        assert_eq!(refused(unsupported), "defaultEffort");
        let mut spaced = settings();
        spaced.default_model = Some("big model".to_owned());
        assert_eq!(refused(spaced), "defaultModel");
    }
}
