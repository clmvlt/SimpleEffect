//! Reading ffmpeg's `showinfo` lines, printed for every frame the `select='gt(scene,T)'` filter
//! kept: their `pts_time` are the shot changes.

use se_domain::Flicks;

/// Source times of the frames `showinfo` printed (lines like
/// `[Parsed_showinfo_3 @ 0x…] n:   0 pts:  61440 pts_time:4.8 duration: …`), in order, without
/// duplicates and without the very first frame (the start of the media is not a change).
#[must_use]
pub fn parse_scene_changes(stderr: &str) -> Vec<Flicks> {
    let mut changes: Vec<Flicks> = Vec::new();
    for line in stderr
        .lines()
        .filter(|line| line.contains("Parsed_showinfo"))
    {
        let Some((_, rest)) = line.split_once("pts_time:") else {
            continue;
        };
        let Some(seconds) = rest
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<f64>().ok())
        else {
            continue;
        };
        let Ok(time) = Flicks::from_seconds_f64(seconds) else {
            continue;
        };
        if time > Flicks::ZERO && changes.last().is_none_or(|last| *last < time) {
            changes.push(time);
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_recorded_output_of_a_scene_detection() {
        let stderr = include_str!("../../tests/fixtures/showinfo/scenes.txt");
        assert_eq!(
            parse_scene_changes(stderr),
            [Flicks::from_millis(2_000), Flicks::from_millis(4_000)]
        );
    }

    #[test]
    fn ignores_other_lines_and_the_first_frame() {
        let stderr = "Input #0, lavfi\n[Parsed_showinfo_2 @ 0x1] n:   0 pts:      0 pts_time:0 duration:1\n[Parsed_showinfo_2 @ 0x1] config in time_base: 1/25\n[Parsed_showinfo_2 @ 0x1] n:   1 pts:     75 pts_time:3 duration:1\n";
        assert_eq!(parse_scene_changes(stderr), [Flicks::from_millis(3_000)]);
    }
}
