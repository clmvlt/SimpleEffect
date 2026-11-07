//! Turning ffprobe's output into the domain's [`MediaInfo`] and [`AssetKind`] (pure).

use se_domain::{AssetKind, AudioStreamInfo, Flicks, FrameRate, MediaInfo, VideoStreamInfo};

use super::ffprobe_output::{ProbeOutput, Stream};

/// Containers ffprobe reads that are not media we can edit (text shown as "ANSI art").
const REFUSED_CONTAINERS: [&str; 1] = ["tty"];

/// Largest relative difference between the average and base frame rates for the base rate to be
/// taken as the nominal rate (`2997/100` reported for 29.97 fps).
const FRAME_RATE_TOLERANCE: f64 = 0.002;

/// What a probe revealed, or why the file is not usable media.
///
/// # Errors
/// A reason (for [`se_app::ports::MediaError::Unsupported`]) when the JSON is not ffprobe output
/// or the file has no picture or sound.
pub fn parse_probe_output(json: &str) -> Result<(AssetKind, MediaInfo), String> {
    let output: ProbeOutput = serde_json::from_str(json)
        .map_err(|error| format!("unexpected ffprobe output: {error}"))?;
    let container = output
        .format
        .as_ref()
        .and_then(|format| format.format_name.clone())
        .unwrap_or_default();
    if REFUSED_CONTAINERS.contains(&container.as_str()) {
        return Err(format!("'{container}' files are not media"));
    }
    let picture = output
        .streams
        .iter()
        .find(|stream| is_type(stream, "video") && !stream.is_attached_picture());
    let audio_streams: Vec<AudioStreamInfo> = output
        .streams
        .iter()
        .filter(|stream| is_type(stream, "audio"))
        .filter_map(audio_stream)
        .collect();
    let still = is_still_image(&container);
    let kind = match (picture, audio_streams.is_empty()) {
        (Some(_), _) if still => AssetKind::Image,
        (Some(_), _) => AssetKind::Video,
        (None, false) => AssetKind::Audio,
        (None, true) => return Err("it has no picture or sound stream".to_owned()),
    };
    let video_stream = match picture {
        Some(stream) => Some(video_stream(stream)?),
        None => None,
    };
    let duration = if kind == AssetKind::Image {
        None
    } else {
        file_duration(&output)
    };
    Ok((
        kind,
        MediaInfo {
            duration,
            container,
            video_stream,
            audio_streams,
        },
    ))
}

fn is_type(stream: &Stream, codec_type: &str) -> bool {
    stream.codec_type.as_deref() == Some(codec_type)
}

/// Still pictures come through image demuxers (`png_pipe`, `jpeg_pipe`, `image2`…). GIFs are
/// treated as stills (their first frame).
fn is_still_image(container: &str) -> bool {
    container.ends_with("_pipe") || container == "image2" || container == "gif"
}

fn video_stream(stream: &Stream) -> Result<VideoStreamInfo, String> {
    let codec = stream.codec_name.clone().unwrap_or_default();
    if codec == "ansi" {
        return Err("text files are not media".to_owned());
    }
    let (Some(width), Some(height)) = (stream.width, stream.height) else {
        return Err("its picture stream has no size".to_owned());
    };
    let pixel_format = stream.pix_fmt.clone();
    Ok(VideoStreamInfo {
        profile: stream.profile.clone(),
        level: stream
            .level
            .filter(|level| *level >= 0)
            .and_then(|level| u16::try_from(level).ok()),
        codec_tag: codec_tag(stream.codec_tag_string.as_deref()),
        bit_depth: bit_depth(
            stream.bits_per_raw_sample.as_deref(),
            pixel_format.as_deref(),
        ),
        chroma_subsampling: pixel_format.as_deref().and_then(chroma_subsampling),
        pixel_format,
        width,
        height,
        frame_rate: frame_rate(stream),
        rotation: rotation(stream),
        codec,
    })
}

fn audio_stream(stream: &Stream) -> Option<AudioStreamInfo> {
    Some(AudioStreamInfo {
        codec: stream.codec_name.clone()?,
        profile: stream.profile.clone(),
        channels: stream.channels.filter(|channels| *channels > 0)?,
        sample_rate: stream.sample_rate.as_deref()?.parse().ok()?,
    })
}

/// The container's duration, else the longest stream's.
fn file_duration(output: &ProbeOutput) -> Option<Flicks> {
    let seconds = output
        .format
        .as_ref()
        .and_then(|format| format.duration.as_deref())
        .and_then(|text| text.parse::<f64>().ok())
        .or_else(|| {
            output
                .streams
                .iter()
                .filter_map(|stream| stream.duration.as_deref()?.parse::<f64>().ok())
                .reduce(f64::max)
        })?;
    Flicks::from_seconds_f64(seconds).ok()
}

/// A four-character tag, without the `[0][0][0][0]` placeholder.
fn codec_tag(tag: Option<&str>) -> Option<String> {
    tag.filter(|tag| tag.len() == 4 && tag.chars().all(|c| c.is_ascii_alphanumeric()))
        .map(str::to_owned)
}

/// Bits per component: the stream's own value, else read from the pixel format.
fn bit_depth(raw: Option<&str>, pixel_format: Option<&str>) -> Option<u8> {
    if let Some(bits) = raw
        .and_then(|raw| raw.parse::<u8>().ok())
        .filter(|bits| *bits > 0)
    {
        return Some(bits);
    }
    let format = pixel_format?;
    let depth = if format.contains("16") {
        16
    } else if format.contains("12") {
        12
    } else if format.contains("10") || format.starts_with("p010") {
        10
    } else {
        8
    };
    Some(depth)
}

/// `4:2:0`, `4:2:2` or `4:4:4`, from the pixel format.
fn chroma_subsampling(pixel_format: &str) -> Option<String> {
    let subsampling = if [
        "yuv420", "yuvj420", "yuva420", "nv12", "nv21", "p010", "p016",
    ]
    .iter()
    .any(|prefix| pixel_format.starts_with(prefix))
    {
        "4:2:0"
    } else if [
        "yuv422", "yuvj422", "yuva422", "nv16", "p210", "y210", "uyvy", "yuyv",
    ]
    .iter()
    .any(|prefix| pixel_format.starts_with(prefix))
    {
        "4:2:2"
    } else if [
        "yuv444", "yuvj444", "yuva444", "gbr", "rgb", "bgr", "argb", "abgr", "rgba", "bgra",
    ]
    .iter()
    .any(|prefix| pixel_format.starts_with(prefix))
    {
        "4:4:4"
    } else {
        return None;
    };
    Some(subsampling.to_owned())
}

/// The nominal frame rate: the average rate when representable in flicks, else the base rate
/// when it is close to the average; `None` for variable frame rates.
fn frame_rate(stream: &Stream) -> Option<FrameRate> {
    let average = stream.avg_frame_rate.as_deref().and_then(parse_ratio);
    let base = stream.r_frame_rate.as_deref().and_then(parse_ratio);
    if let Some(rate) = average.and_then(|(num, den)| FrameRate::new(num, den).ok()) {
        return Some(rate);
    }
    let (base_num, base_den) = base?;
    let (avg_num, avg_den) = average?;
    let base_fps = f64::from(base_num) / f64::from(base_den);
    let average_fps = f64::from(avg_num) / f64::from(avg_den);
    if ((average_fps - base_fps) / base_fps).abs() > FRAME_RATE_TOLERANCE {
        return None;
    }
    FrameRate::new(base_num, base_den).ok()
}

/// `30000/1001` → `(30000, 1001)`; `0/0` and malformed text → `None`.
fn parse_ratio(text: &str) -> Option<(u32, u32)> {
    let (numerator, denominator) = text.split_once('/')?;
    let numerator: u32 = numerator.trim().parse().ok()?;
    let denominator: u32 = denominator.trim().parse().ok()?;
    (numerator > 0 && denominator > 0).then_some((numerator, denominator))
}

/// Clockwise display rotation (0, 90, 180, 270): from the display matrix (counter-clockwise
/// degrees), else the legacy `rotate` tag (clockwise).
fn rotation(stream: &Stream) -> u16 {
    let from_matrix = stream
        .side_data_list
        .iter()
        .filter(|data| data.side_data_type.as_deref() == Some("Display Matrix"))
        .find_map(|data| data.rotation)
        .map(|counter_clockwise| -counter_clockwise);
    let from_tag = || {
        stream
            .tags
            .get("rotate")
            .and_then(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .or_else(|| Some(value.to_string()))
            })
            .and_then(|text| text.parse::<f64>().ok())
    };
    let degrees = from_matrix.or_else(from_tag).unwrap_or(0.0);
    let quarter_turns = (degrees / 90.0).round().rem_euclid(4.0);
    // `quarter_turns` is 0, 1, 2 or 3.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let turns = quarter_turns as u16;
    turns * 90
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(json: &str) -> Stream {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn rotation_reads_the_display_matrix_then_the_legacy_tag() {
        let matrix =
            stream(r#"{"side_data_list":[{"side_data_type":"Display Matrix","rotation":-90}]}"#);
        assert_eq!(rotation(&matrix), 90);
        let upside_down =
            stream(r#"{"side_data_list":[{"side_data_type":"Display Matrix","rotation":180}]}"#);
        assert_eq!(rotation(&upside_down), 180);
        let tagged = stream(r#"{"tags":{"rotate":"270"}}"#);
        assert_eq!(rotation(&tagged), 270);
        assert_eq!(rotation(&stream("{}")), 0);
    }

    #[test]
    fn variable_frame_rates_have_no_nominal_rate() {
        let ntsc = stream(r#"{"avg_frame_rate":"30000/1001","r_frame_rate":"30000/1001"}"#);
        assert_eq!(frame_rate(&ntsc), Some(FrameRate::FPS_29_97));
        let rounded = stream(r#"{"avg_frame_rate":"2997/100","r_frame_rate":"30000/1001"}"#);
        assert_eq!(frame_rate(&rounded), Some(FrameRate::FPS_29_97));
        let variable = stream(r#"{"avg_frame_rate":"16380000/546271","r_frame_rate":"60/1"}"#);
        assert_eq!(frame_rate(&variable), None);
        assert_eq!(frame_rate(&stream(r#"{"avg_frame_rate":"0/0"}"#)), None);
    }

    #[test]
    fn pixel_formats_tell_depth_and_subsampling() {
        assert_eq!(bit_depth(None, Some("yuv422p10le")), Some(10));
        assert_eq!(bit_depth(Some("12"), Some("yuv420p")), Some(12));
        assert_eq!(chroma_subsampling("yuvj420p").as_deref(), Some("4:2:0"));
        assert_eq!(chroma_subsampling("yuv422p10le").as_deref(), Some("4:2:2"));
        assert_eq!(chroma_subsampling("rgba").as_deref(), Some("4:4:4"));
        assert_eq!(chroma_subsampling("pal8"), None);
    }

    #[test]
    fn placeholder_tags_and_unknown_levels_are_dropped() {
        assert_eq!(codec_tag(Some("[0][0][0][0]")), None);
        assert_eq!(codec_tag(Some("hvc1")).as_deref(), Some("hvc1"));
    }

    #[test]
    fn text_and_empty_files_are_refused() {
        let tty = r#"{"streams":[{"codec_type":"video","codec_name":"ansi","width":640,"height":400}],"format":{"format_name":"tty"}}"#;
        assert!(parse_probe_output(tty).is_err());
        let empty = r#"{"streams":[],"format":{"format_name":"mp3"}}"#;
        assert!(
            parse_probe_output(empty)
                .unwrap_err()
                .contains("no picture or sound")
        );
        assert!(parse_probe_output("not json").is_err());
    }

    #[test]
    fn cover_art_does_not_make_a_song_a_video() {
        let song = r#"{"streams":[
            {"codec_type":"audio","codec_name":"mp3","sample_rate":"44100","channels":2},
            {"codec_type":"video","codec_name":"mjpeg","width":600,"height":600,"disposition":{"attached_pic":1}}
        ],"format":{"format_name":"mp3","duration":"3.5"}}"#;
        let (kind, info) = parse_probe_output(song).unwrap();
        assert_eq!(kind, AssetKind::Audio);
        assert_eq!(info.video_stream, None);
        assert_eq!(info.duration, Flicks::from_seconds_f64(3.5).ok());
    }
}
