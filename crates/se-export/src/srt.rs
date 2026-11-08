//! SRT (`.srt`) subtitles: a pure writer and a tolerant reader (prompt 23).
//!
//! Written files are UTF-8 with a byte-order mark (players that guess encodings read accents
//! right) and CRLF line endings, cues numbered from 1, times `HH:MM:SS,mmm` rounded to the nearest
//! millisecond (an end never before its start plus 1 ms). The reader accepts a byte-order mark,
//! LF/CRLF/CR line endings, `.` or `,` before milliseconds, missing or wrong cue numbers, cue
//! settings after the timing (`X1:…`), extra blank lines, and drops markup (`<i>`, `<font …>`,
//! `{\an8}`).

use se_app::ports::SubtitleCue;
use se_domain::Flicks;

/// Flicks per millisecond.
const FLICKS_PER_MILLISECOND: i64 = Flicks::PER_SECOND / 1000;

/// Why an SRT text could not be read, with the 1-based line number.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {message}")]
pub struct SrtError {
    /// Line of the problem.
    pub line: usize,
    /// What is wrong and what was expected.
    pub message: String,
}

/// `cues` as an SRT file.
#[must_use]
pub fn write_srt(cues: &[SubtitleCue]) -> String {
    let mut text = String::from('\u{feff}');
    for (index, cue) in cues.iter().enumerate() {
        let start = milliseconds(cue.start);
        let end = milliseconds(cue.end).max(start + 1);
        let body = cue.text.replace("\r\n", "\n").replace('\r', "\n");
        let body = body.trim_matches('\n').replace('\n', "\r\n");
        for line in [
            (index + 1).to_string(),
            format!("{} --> {}", clock(start), clock(end)),
            body,
            String::new(),
        ] {
            text.push_str(&line);
            text.push_str("\r\n");
        }
    }
    text
}

/// The cues of an SRT text, in file order.
///
/// # Errors
/// [`SrtError`] when a block has no valid timing line or no cue is found.
pub fn parse_srt(text: &str) -> Result<Vec<SubtitleCue>, SrtError> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cues = Vec::new();
    let mut lines = normalized.lines().enumerate().peekable();
    while let Some((number, line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let timing_line = if line.contains("-->") {
            (number, line)
        } else {
            // A cue number (or anything) before the timing line.
            match lines.next() {
                Some((next_number, next)) if next.contains("-->") => (next_number, next.trim()),
                _ => {
                    return Err(SrtError {
                        line: number + 1,
                        message: format!(
                            "expected a cue timing such as '00:00:01,000 --> 00:00:02,500' \
                             after '{line}'"
                        ),
                    });
                }
            }
        };
        let (start, end) = timing(timing_line.1).ok_or_else(|| SrtError {
            line: timing_line.0 + 1,
            message: format!(
                "'{}' is not a cue timing; expected 'HH:MM:SS,mmm --> HH:MM:SS,mmm' with the \
                 end after the start",
                timing_line.1
            ),
        })?;
        let mut body = Vec::new();
        while let Some((_, next)) = lines.peek() {
            if next.trim().is_empty() {
                break;
            }
            body.push(strip_markup(next.trim()));
            lines.next();
        }
        cues.push(SubtitleCue {
            start,
            end,
            text: body.join("\n").trim().to_owned(),
        });
    }
    if cues.is_empty() {
        return Err(SrtError {
            line: 1,
            message: "no subtitle found; a cue is a number, a timing line \
                      '00:00:01,000 --> 00:00:02,500' and its text"
                .to_owned(),
        });
    }
    Ok(cues)
}

/// A time in whole milliseconds, nearest (halves away from zero), never negative.
fn milliseconds(time: Flicks) -> i64 {
    let value = time.get().max(0);
    (value + FLICKS_PER_MILLISECOND / 2) / FLICKS_PER_MILLISECOND
}

fn clock(milliseconds: i64) -> String {
    let hours = milliseconds / 3_600_000;
    let minutes = milliseconds / 60_000 % 60;
    let seconds = milliseconds / 1000 % 60;
    format!(
        "{hours:02}:{minutes:02}:{seconds:02},{:03}",
        milliseconds % 1000
    )
}

/// The start and end of a timing line; `None` unless both read and the end follows the start.
fn timing(line: &str) -> Option<(Flicks, Flicks)> {
    let (start, rest) = line.split_once("-->")?;
    let end = rest.split_whitespace().next()?;
    let start = parse_clock(start.trim())?;
    let end = parse_clock(end)?;
    (end > start).then_some((start, end))
}

/// `HH:MM:SS,mmm` (also `.` and fewer millisecond digits; hours may have more digits) in flicks.
fn parse_clock(text: &str) -> Option<Flicks> {
    let (clock, fraction) = text.split_once([',', '.']).unwrap_or((text, "0"));
    let parts: Vec<&str> = clock.split(':').collect();
    let [hours, minutes, seconds] = parts.as_slice() else {
        return None;
    };
    let number = |part: &str| -> Option<i64> {
        (!part.is_empty() && part.chars().all(|character| character.is_ascii_digit()))
            .then(|| part.parse().ok())
            .flatten()
    };
    let (hours, minutes, seconds) = (number(hours)?, number(minutes)?, number(seconds)?);
    if minutes >= 60 || seconds >= 60 || fraction.len() > 3 {
        return None;
    }
    let millis = number(fraction)? * 10_i64.pow(3 - u32::try_from(fraction.len()).ok()?);
    let total = ((hours * 60 + minutes) * 60 + seconds) * 1000 + millis;
    Flicks::new(total.checked_mul(FLICKS_PER_MILLISECOND)?).ok()
}

/// `line` without HTML-like tags (`<i>`, `</font>`) and ASS overrides (`{\an8}`).
fn strip_markup(line: &str) -> String {
    let mut text = String::with_capacity(line.len());
    let mut depth: Option<char> = None;
    for character in line.chars() {
        match (depth, character) {
            (None, '<') => depth = Some('>'),
            (None, '{') if line.contains("{\\") => depth = Some('}'),
            (Some(closing), current) if current == closing => depth = None,
            (Some(_), _) => {}
            (None, current) => text.push(current),
        }
    }
    text.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn millis(value: i64) -> Flicks {
        Flicks::new(value * FLICKS_PER_MILLISECOND).unwrap()
    }

    fn cue(start: i64, end: i64, text: &str) -> SubtitleCue {
        SubtitleCue {
            start: millis(start),
            end: millis(end),
            text: text.to_owned(),
        }
    }

    #[test]
    fn writes_numbered_cues_with_a_bom_and_crlf() {
        let text = write_srt(&[
            cue(1000, 2500, "Bonjour à tous"),
            cue(3_723_456, 3_725_000, "deux\nlignes"),
        ]);
        assert_eq!(
            text,
            "\u{feff}1\r\n00:00:01,000 --> 00:00:02,500\r\nBonjour à tous\r\n\r\n\
             2\r\n01:02:03,456 --> 01:02:05,000\r\ndeux\r\nlignes\r\n\r\n"
        );
    }

    #[test]
    fn written_files_read_back_identically() {
        let cues = vec![cue(0, 900, "Salut"), cue(950, 4000, "« Ça va ? »\nOui !")];
        assert_eq!(parse_srt(&write_srt(&cues)).unwrap(), cues);
    }

    #[test]
    fn reads_loose_files() {
        let text = "\n\n7\n00:00:01.5 --> 00:00:02,250 X1:10 X2:20\n<i>Bonjour</i> {\\an8}à tous\n\n\n\
                    00:00:03,000 --> 00:00:04,000\nsans numéro\n";
        assert_eq!(
            parse_srt(text).unwrap(),
            [
                cue(1500, 2250, "Bonjour à tous"),
                cue(3000, 4000, "sans numéro")
            ]
        );
    }

    #[test]
    fn rounds_to_the_millisecond_and_never_writes_an_empty_cue() {
        let text = write_srt(&[SubtitleCue {
            start: Flicks::new(FLICKS_PER_MILLISECOND * 2 - 1).unwrap(),
            end: Flicks::new(FLICKS_PER_MILLISECOND * 2).unwrap(),
            text: "x".to_owned(),
        }]);
        assert!(text.contains("00:00:00,002 --> 00:00:00,003"), "{text}");
    }

    #[test]
    fn errors_name_the_line() {
        let error = parse_srt("1\n00:00:01,000 -> 00:00:02,000\nTexte\n").unwrap_err();
        assert_eq!(error.line, 1);
        let error = parse_srt("1\n00:00:05,000 --> 00:00:02,000\nTexte\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("end after the start"));
        assert!(parse_srt("\n\n").is_err());
    }
}
