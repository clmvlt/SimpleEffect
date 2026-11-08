//! whisper-cli's arguments for one transcription (pure).

use std::path::Path;

use se_app::ports::TranscriptionRequest;

/// Base name (without `.json`) of the output file in the run's folder.
pub const OUTPUT_NAME: &str = "transcript";

/// The arguments of a run of `request`:
/// - full JSON output (`-oj -ojf`: tokens with probabilities and times) in `work_dir`;
/// - progress lines (`-pp`), the language or `auto`, `threads` threads;
/// - DTW alignment (`-dtw <preset>`) for catalogue models, which needs flash attention off
///   (`-nfa`; about 20 % slower, much better word times);
/// - voice detection (`--vad -vm <model>`) when a voice model is given;
/// - the initial prompt: a few hesitations in the transcript's language when they must be kept
///   (whisper imitates the prompt's style, so it writes « euh » instead of cleaning it away),
///   then the vocabulary;
/// - `-ng` when the GPU must not be used.
#[must_use]
pub fn arguments(request: &TranscriptionRequest, use_gpu: bool, threads: usize) -> Vec<String> {
    let mut args = vec![
        "-m".to_owned(),
        path(&request.model.path),
        "-f".to_owned(),
        path(&request.audio),
        "-l".to_owned(),
        request
            .language
            .clone()
            .unwrap_or_else(|| "auto".to_owned()),
        "-oj".to_owned(),
        "-ojf".to_owned(),
        "-of".to_owned(),
        path(&request.work_dir.join(OUTPUT_NAME)),
        "-pp".to_owned(),
        "-t".to_owned(),
        threads.max(1).to_string(),
    ];
    if let Some(preset) = &request.model.alignment {
        args.extend(["-dtw".to_owned(), preset.clone(), "-nfa".to_owned()]);
    }
    if let Some(voice_model) = &request.voice_model {
        args.extend(["--vad".to_owned(), "-vm".to_owned(), path(voice_model)]);
    }
    if let Some(prompt) = initial_prompt(request) {
        args.extend(["--prompt".to_owned(), prompt]);
    }
    if !use_gpu {
        args.push("-ng".to_owned());
    }
    args
}

/// Hesitations written in whisper's style, per language (whisper continues the prompt's style).
fn hesitations(language: Option<&str>) -> &'static str {
    match language {
        Some("fr") => "Euh, alors, hum… euh, bon, ben voilà.",
        Some("en") => "Um, so, uh… I mean, hmm, you know.",
        _ => "Um, uh, hmm, euh.",
    }
}

/// The initial prompt: hesitations (when kept), then the vocabulary.
fn initial_prompt(request: &TranscriptionRequest) -> Option<String> {
    let hesitations = request
        .keep_hesitations
        .then(|| hesitations(request.language.as_deref()));
    match (hesitations, request.vocabulary.as_deref()) {
        (Some(hesitations), Some(vocabulary)) => Some(format!("{hesitations} {vocabulary}")),
        (Some(hesitations), None) => Some(hesitations.to_owned()),
        (None, Some(vocabulary)) => Some(vocabulary.to_owned()),
        (None, None) => None,
    }
}

fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use se_app::ports::ModelChoice;

    use super::*;

    fn request() -> TranscriptionRequest {
        TranscriptionRequest {
            audio: PathBuf::from("work/speech-16k.wav"),
            work_dir: PathBuf::from("work"),
            model: ModelChoice {
                id: "large-v3-turbo".to_owned(),
                path: PathBuf::from("models/ggml-large-v3-turbo.bin"),
                alignment: Some("large.v3.turbo".to_owned()),
            },
            voice_model: Some(PathBuf::from("models/ggml-silero-v6.2.0.bin")),
            language: Some("fr".to_owned()),
            vocabulary: Some("SimpleEffect, Dimzou".to_owned()),
            keep_hesitations: false,
            use_gpu: true,
        }
    }

    #[test]
    fn asks_for_full_json_alignment_voice_detection_and_the_vocabulary() {
        let args = arguments(&request(), true, 8);
        let line = args.join(" ");
        assert!(line.contains("-l fr -oj -ojf -of"));
        assert!(line.contains("-pp -t 8 -dtw large.v3.turbo -nfa --vad -vm"));
        assert!(
            args.contains(&"SimpleEffect, Dimzou".to_owned()),
            "one argument"
        );
        assert!(!args.contains(&"-ng".to_owned()));
    }

    #[test]
    fn detects_the_language_and_skips_options_it_cannot_use() {
        let mut request = request();
        request.language = None;
        request.model.alignment = None;
        request.voice_model = None;
        request.vocabulary = None;
        let args = arguments(&request, false, 0);
        let line = args.join(" ");
        assert!(line.contains("-l auto"));
        assert!(!line.contains("-dtw") && !line.contains("--vad") && !line.contains("--prompt"));
        assert!(line.ends_with("-t 1 -ng"));
    }

    #[test]
    fn kept_hesitations_open_the_prompt_in_the_transcripts_language() {
        let mut request = request();
        request.keep_hesitations = true;
        let args = arguments(&request, true, 8);
        assert!(
            args.contains(&"Euh, alors, hum… euh, bon, ben voilà. SimpleEffect, Dimzou".to_owned())
        );
        request.vocabulary = None;
        request.language = None;
        let args = arguments(&request, true, 8);
        assert!(args.contains(&"Um, uh, hmm, euh.".to_owned()));
    }
}
