//! Rules of a sequence itself: `INVALID_FORMAT` and `NEGATIVE_TIME` for markers.

use super::{Path, Report, ViolationCode, report::seconds};
use crate::{Flicks, Format, Sequence};

/// Largest frame width or height accepted, in pixels.
const MAX_DIMENSION: u32 = 16_384;

pub(super) fn check(sequence: &Sequence, path: &Path, report: &mut Report) {
    check_format(&sequence.format, &path.field("format"), report);
    let markers = path.field("markers");
    for marker in &sequence.markers {
        if marker.time.is_negative() {
            report.push(
                ViolationCode::NegativeTime,
                &markers.key(marker.id).field("time"),
                format!(
                    "marker is at {}, before the start of the sequence; move it to 0s or later",
                    seconds(marker.time)
                ),
            );
        }
    }
}

fn check_format(format: &Format, path: &Path, report: &mut Report) {
    for (name, value) in [("width", format.width), ("height", format.height)] {
        if !(2..=MAX_DIMENSION).contains(&value) || value % 2 != 0 {
            report.push(
                ViolationCode::InvalidFormat,
                &path.field(name),
                format!(
                    "{name} must be an even number of pixels between 2 and {MAX_DIMENSION} \
                     (got {value}); video encoders need even dimensions"
                ),
            );
        }
    }
    let sample_rate = i64::from(format.sample_rate);
    if sample_rate == 0 || Flicks::PER_SECOND % sample_rate != 0 {
        report.push(
            ViolationCode::InvalidFormat,
            &path.field("sampleRate"),
            format!(
                "sample rate {sample_rate} Hz is not supported: one sample must last a whole \
                 number of flicks; use 44100, 48000 or 96000"
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ViolationCode,
        test_support::{example_project, millis},
        validate_project,
    };

    #[test]
    fn reports_unusable_dimensions_and_sample_rates() {
        let mut project = example_project();
        let format = &mut project.sequences[0].format;
        format.width = 1081;
        format.height = 0;
        format.sample_rate = 44_000;
        let violations = validate_project(&project);
        let fields: Vec<_> = violations
            .iter()
            .map(|violation| {
                assert_eq!(violation.code, ViolationCode::InvalidFormat);
                violation.path.rsplit('.').next().unwrap_or_default()
            })
            .collect();
        assert_eq!(fields, ["width", "height", "sampleRate"]);
    }

    #[test]
    fn reports_markers_before_zero() {
        let mut project = example_project();
        project.sequences[0].markers[0].time = millis(-500);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, ViolationCode::NegativeTime);
        assert!(
            violations[0].message.contains("-0.500s"),
            "{}",
            violations[0].message
        );
    }
}
