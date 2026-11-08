//! The smart-cut settings, kept as `library.db` settings (`smartCuts.*`), and their checks.

use se_domain::Flicks;
use se_protocol::{FillerList, SilenceCutParams, SmartCutSettings};

use super::fillers::{DEFAULT_FILLER_MAX_DURATION, default_filler_lists};
use crate::{AppError, ports::AppLibraryRepository};

/// Keys of the smart-cut settings in `library.db`.
pub mod keys {
    /// The filler lists: one line per language, `<code>\t<word>|<word>…`.
    pub const FILLERS: &str = "smartCuts.fillers";
    /// Longest filler occurrence (flicks).
    pub const FILLER_MAX_DURATION: &str = "smartCuts.fillerMaxDuration";
    /// Silence threshold (dBFS).
    pub const SILENCE_THRESHOLD: &str = "smartCuts.silenceThresholdDbfs";
    /// Shortest silence cut (flicks).
    pub const MIN_SILENCE: &str = "smartCuts.minSilence";
    /// Silence kept around cuts (flicks).
    pub const PADDING: &str = "smartCuts.padding";
}

/// Accepted silence thresholds (dBFS).
pub const THRESHOLD_RANGE: std::ops::RangeInclusive<f64> = -80.0..=-10.0;
/// Most lists, words per list and characters per word.
const MAX_LISTS: usize = 20;
const MAX_WORDS: usize = 200;
const MAX_WORD_CHARS: usize = 40;

/// The defaults: the French and English lists, 1.5 s, speech silences.
#[must_use]
pub fn default_settings() -> SmartCutSettings {
    SmartCutSettings {
        fillers: default_filler_lists(),
        filler_max_duration: DEFAULT_FILLER_MAX_DURATION,
        silence: SilenceCutParams::default(),
    }
}

/// The stored settings (defaults for values never set or unreadable).
///
/// # Errors
/// [`AppError::Storage`] when `library.db` cannot be read.
pub async fn load(library: &dyn AppLibraryRepository) -> Result<SmartCutSettings, AppError> {
    let defaults = default_settings();
    let fillers = library
        .setting(keys::FILLERS)
        .await?
        .map(|text| decoded_lists(&text))
        .unwrap_or(defaults.fillers);
    let flicks = |value: Option<String>, default: Flicks| {
        value
            .and_then(|text| text.parse::<i64>().ok())
            .and_then(|value| Flicks::new(value).ok())
            .unwrap_or(default)
    };
    let loaded = SmartCutSettings {
        fillers,
        filler_max_duration: flicks(
            library.setting(keys::FILLER_MAX_DURATION).await?,
            defaults.filler_max_duration,
        ),
        silence: SilenceCutParams {
            threshold_dbfs: library
                .setting(keys::SILENCE_THRESHOLD)
                .await?
                .and_then(|text| text.parse::<f64>().ok())
                .unwrap_or(defaults.silence.threshold_dbfs),
            min_silence: flicks(
                library.setting(keys::MIN_SILENCE).await?,
                defaults.silence.min_silence,
            ),
            padding: flicks(
                library.setting(keys::PADDING).await?,
                defaults.silence.padding,
            ),
        },
    };
    Ok(check(&loaded).map_or_else(|_| default_settings(), |()| loaded))
}

/// Saves `settings` after checking them; returns them as stored (lists trimmed).
///
/// # Errors
/// [`AppError::InvalidSetting`] naming the faulty value; [`AppError::Storage`].
pub async fn save(
    library: &dyn AppLibraryRepository,
    settings: &SmartCutSettings,
) -> Result<SmartCutSettings, AppError> {
    let cleaned = SmartCutSettings {
        fillers: settings.fillers.iter().map(cleaned_list).collect(),
        ..settings.clone()
    };
    check(&cleaned)?;
    let entries = [
        (keys::FILLERS, encoded_lists(&cleaned.fillers)),
        (
            keys::FILLER_MAX_DURATION,
            cleaned.filler_max_duration.get().to_string(),
        ),
        (
            keys::SILENCE_THRESHOLD,
            cleaned.silence.threshold_dbfs.to_string(),
        ),
        (
            keys::MIN_SILENCE,
            cleaned.silence.min_silence.get().to_string(),
        ),
        (keys::PADDING, cleaned.silence.padding.get().to_string()),
    ];
    for (key, value) in entries {
        library.set_setting(key, &value).await?;
    }
    Ok(cleaned)
}

/// The stored form of `lists`.
fn encoded_lists(lists: &[FillerList]) -> String {
    lists
        .iter()
        .map(|list| format!("{}\t{}", list.language, list.words.join("|")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The lists of a stored text (lines without a tab are skipped).
fn decoded_lists(text: &str) -> Vec<FillerList> {
    text.lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(language, words)| FillerList {
            language: language.to_owned(),
            words: words
                .split('|')
                .filter(|word| !word.is_empty())
                .map(str::to_owned)
                .collect(),
        })
        .collect()
}

/// `list` with its language lowercased and its words trimmed, empty and repeated ones dropped.
fn cleaned_list(list: &FillerList) -> FillerList {
    let mut words: Vec<String> = Vec::new();
    for word in &list.words {
        let word = word.split_whitespace().collect::<Vec<_>>().join(" ");
        if !word.is_empty() && !words.contains(&word) {
            words.push(word);
        }
    }
    FillerList {
        language: list.language.trim().to_lowercase(),
        words,
    }
}

/// Refuses values outside their ranges.
///
/// # Errors
/// [`AppError::InvalidSetting`].
pub fn check(settings: &SmartCutSettings) -> Result<(), AppError> {
    let invalid = |name, reason: &str| AppError::InvalidSetting {
        name,
        reason: reason.to_owned(),
    };
    if !THRESHOLD_RANGE.contains(&settings.silence.threshold_dbfs) {
        return Err(invalid(
            "thresholdDbfs",
            "choose a threshold from -80 to -10 dBFS",
        ));
    }
    if !(Flicks::from_millis(100)..=Flicks::from_millis(10_000))
        .contains(&settings.silence.min_silence)
    {
        return Err(invalid(
            "minSilence",
            "choose a minimum silence from 0.1 s to 10 s",
        ));
    }
    if !(Flicks::ZERO..=Flicks::from_millis(1000)).contains(&settings.silence.padding) {
        return Err(invalid("padding", "choose a padding from 0 to 1 s"));
    }
    if !(Flicks::from_millis(200)..=Flicks::from_millis(5000))
        .contains(&settings.filler_max_duration)
    {
        return Err(invalid(
            "fillerMaxDuration",
            "choose a longest filler from 0.2 s to 5 s",
        ));
    }
    if settings.fillers.len() > MAX_LISTS {
        return Err(invalid("fillers", "keep at most 20 languages"));
    }
    for list in &settings.fillers {
        let language_ok = (2..=8).contains(&list.language.len())
            && list
                .language
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c == '-');
        if !language_ok {
            return Err(invalid(
                "fillers",
                "name each list with a language code such as fr or en",
            ));
        }
        if list.words.len() > MAX_WORDS
            || list
                .words
                .iter()
                .any(|word| word.chars().count() > MAX_WORD_CHARS)
        {
            return Err(invalid(
                "fillers",
                "keep at most 200 fillers per language, each at most 40 characters",
            ));
        }
        if list
            .words
            .iter()
            .any(|word| word.contains('|') || word.chars().any(char::is_control))
        {
            return Err(invalid(
                "fillers",
                "fillers cannot contain « | » or line breaks",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_pass_their_own_checks() {
        assert!(check(&default_settings()).is_ok());
    }

    #[test]
    fn out_of_range_values_are_refused_with_their_name() {
        let mut settings = default_settings();
        settings.silence.threshold_dbfs = -5.0;
        assert!(matches!(
            check(&settings),
            Err(AppError::InvalidSetting {
                name: "thresholdDbfs",
                ..
            })
        ));
        let mut settings = default_settings();
        settings.fillers.push(FillerList {
            language: "français".to_owned(),
            words: Vec::new(),
        });
        assert!(matches!(
            check(&settings),
            Err(AppError::InvalidSetting {
                name: "fillers",
                ..
            })
        ));
    }

    #[test]
    fn lists_survive_their_stored_form() {
        let lists = default_filler_lists();
        assert_eq!(decoded_lists(&encoded_lists(&lists)), lists);
    }

    #[test]
    fn lists_are_cleaned() {
        let list = cleaned_list(&FillerList {
            language: " FR ".to_owned(),
            words: vec![
                " euh ".to_owned(),
                "euh".to_owned(),
                String::new(),
                "tu  vois".to_owned(),
            ],
        });
        assert_eq!(list.language, "fr");
        assert_eq!(list.words, ["euh", "tu vois"]);
    }
}
