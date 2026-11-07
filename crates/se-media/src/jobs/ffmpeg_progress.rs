//! Reading ffmpeg's `-progress pipe:1` output: `key=value` lines, one block per update.

/// Seconds of output written so far, from an `out_time_us=…` line (`out_time_ms` is also in
/// microseconds, a historical ffmpeg quirk); `None` for other lines or `N/A`.
#[must_use]
pub fn out_time_seconds(line: &str) -> Option<f64> {
    let (key, value) = line.split_once('=')?;
    if key != "out_time_us" && key != "out_time_ms" {
        return None;
    }
    let micros: f64 = value.trim().parse().ok()?;
    (micros >= 0.0).then_some(micros / 1_000_000.0)
}

/// The fraction done after `seconds` of `total` seconds, in `[0, 1]`.
#[must_use]
pub fn fraction(seconds: f64, total: Option<f64>) -> Option<f64> {
    let total = total.filter(|total| *total > 0.0)?;
    Some((seconds / total).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_output_times_in_microseconds() {
        assert_eq!(out_time_seconds("out_time_us=2500000"), Some(2.5));
        assert_eq!(out_time_seconds("out_time_ms=1000000"), Some(1.0));
        assert_eq!(out_time_seconds("out_time_us=N/A"), None);
        assert_eq!(out_time_seconds("frame=12"), None);
        assert_eq!(out_time_seconds("progress=end"), None);
    }

    #[test]
    fn fractions_need_a_duration() {
        assert_eq!(fraction(5.0, Some(10.0)), Some(0.5));
        assert_eq!(fraction(15.0, Some(10.0)), Some(1.0));
        assert_eq!(fraction(5.0, None), None);
    }
}
