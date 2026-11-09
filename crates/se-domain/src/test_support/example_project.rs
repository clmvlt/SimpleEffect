//! [`example_project`]: a valid document using every item kind, built from deterministic ids.

#![allow(
    clippy::expect_used,
    reason = "the example is built from constant, known-valid values; a failure is a bug here"
)]

use std::collections::BTreeMap;

use super::{item, millis, range_ms, track};
use crate::{
    Animatable, Asset, AssetAnalysis, AssetKind, AudioStreamInfo, AudioSummary, BrandKit,
    CURRENT_SCHEMA_VERSION, CaptionItem, CaptionWord, Color, ContentFingerprint, Decodability,
    DerivedFile, DerivedMedia, DerivedStatus, DesignPixels, Easing, EasingPreset, FadeShape,
    Format, FormatPreset, FrameRate, ImageFit, ImageItem, Item, ItemId, ItemKind, Keyframe,
    LinkGroupId, Marker, MarkerColor, MarkerId, MediaClip, MediaInfo, MotionTemplate, Origin,
    Project, ProjectId, ProjectSettings, Reframe, SceneSummary, Sequence, SequenceId,
    ShapeGeometry, ShapeItem, StreamDecodability, Stroke, TaskId, TemplateChoice, TemplateItem,
    TemplateParam, TemplateParamSpec, TemplateParamValue,
    TextItem, TextStyle, Track, TrackKind, TranscriptSummary, TranscriptionSettings, Transform2D,
    Transition, TransitionAlignment, TransitionId, TransitionKind, Vec2, VideoStreamInfo,
};

/// Ids of [`example_project`] that tests refer to: its three assets and its template.
pub mod example_ids {
    use crate::{AssetId, CaptionStyleId, TemplateId};

    /// `interview.mp4`, 60 s, 1080p 29.97 with stereo sound.
    pub const VIDEO_ASSET: AssetId = AssetId::from_test_index(1);
    /// `music.wav`, 120 s.
    pub const AUDIO_ASSET: AssetId = AssetId::from_test_index(2);
    /// `logo.png`, 512 × 512.
    pub const IMAGE_ASSET: AssetId = AssetId::from_test_index(3);
    /// The project's only motion template.
    pub const TEMPLATE: TemplateId = TemplateId::from_test_index(1);
    /// The project's only caption style (the `minimal` preset), used by its caption.
    pub const CAPTION_STYLE: CaptionStyleId = CaptionStyleId::from_test_index(1);
}

/// A valid project (`validate_project` finds nothing) with every item kind, keyframes, a link
/// group, an AI-created item, a marker, a template, a caption style and a brand kit.
///
/// Sequence `Main edit`, 1080p at 29.97 fps. Tracks, bottom to top: V1 (two interview clips, the
/// second at 2× speed, filling the frame around a moving focus), V2 (a keyframed text and the logo), V3 (a template, a shape and a
/// caption), A1 (the interview's sound, linked to the first clip), A2 (music with volume
/// automation).
///
/// # Panics
/// Never: every value is a valid constant.
#[must_use]
pub fn example_project() -> Project {
    let format = Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_29_97)
        .expect("named presets have dimensions");
    Project {
        id: ProjectId::from_test_index(1),
        name: "Example".to_owned(),
        schema_version: CURRENT_SCHEMA_VERSION,
        revision: 42,
        settings: ProjectSettings {
            default_format: format,
            transcription: TranscriptionSettings::default(),
        },
        assets: example_assets(),
        sequences: vec![Sequence {
            id: SequenceId::from_test_index(1),
            name: "Main edit".to_owned(),
            format,
            tracks: example_tracks(),
            markers: vec![Marker {
                id: MarkerId::from_test_index(1),
                time: millis(3000),
                label: "Intro ends".to_owned(),
                color: MarkerColor::Yellow,
            }],
        }],
        templates: vec![lower_third_template()],
        brand_kit: Some(BrandKit {
            colors: vec![
                Color::opaque(0xff, 0xcc, 0x00),
                Color::opaque(0x1a, 0x1a, 0x1a),
            ],
            fonts: vec!["Inter".to_owned()],
        }),
        caption_styles: vec![
            crate::caption_style_from_preset("minimal", example_ids::CAPTION_STYLE)
                .expect("the minimal preset exists"),
        ],
    }
}

fn example_assets() -> Vec<Asset> {
    let asset = |id, name: &str, kind, media_info| Asset {
        id,
        original_path: format!("/home/demo/media/{name}"),
        relative_hint: Some(format!("media/{name}")),
        kind,
        decodability: Decodability {
            video: if kind == AssetKind::Audio {
                StreamDecodability::NoStream
            } else {
                StreamDecodability::Decodable
            },
            audio: if kind == AssetKind::Image {
                StreamDecodability::NoStream
            } else {
                StreamDecodability::Decodable
            },
        },
        media_info: Some(media_info),
        fingerprint: Some(fingerprint(name)),
        derived: DerivedMedia::default(),
        analysis: AssetAnalysis::default(),
        missing: false,
    };
    let stereo = |codec: &str| AudioStreamInfo {
        codec: codec.to_owned(),
        profile: (codec == "aac").then(|| "LC".to_owned()),
        channels: 2,
        sample_rate: 48_000,
    };
    let mut video = asset(
        example_ids::VIDEO_ASSET,
        "interview.mp4",
        AssetKind::Video,
        MediaInfo {
            duration: Some(millis(60_000)),
            container: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
            video_stream: Some(picture("h264", 1920, 1080, Some(FrameRate::FPS_29_97))),
            audio_streams: vec![stereo("aac")],
        },
    );
    video.derived.proxy = Some(DerivedFile {
        relative_path: format!(
            "cache/media/{}/proxy-540.mp4",
            fingerprint("interview.mp4").cache_key()
        ),
        status: DerivedStatus::Ready,
    });
    video.analysis = AssetAnalysis {
        audio: Some(AudioSummary {
            integrated_lufs: Some(-16.5),
            loudness_range_lu: Some(5.5),
            true_peak_dbtp: Some(-1.0),
            silence_count: 3,
            silence_duration: millis(2_250),
        }),
        scenes: Some(SceneSummary { change_count: 4 }),
        transcript: Some(TranscriptSummary {
            language: "fr".to_owned(),
            model: "large-v3-turbo".to_owned(),
            word_count: 1_234,
            transcribed_at: 1_789_000_000_000,
        }),
    };
    let music = asset(
        example_ids::AUDIO_ASSET,
        "music.wav",
        AssetKind::Audio,
        MediaInfo {
            duration: Some(millis(120_000)),
            container: "wav".to_owned(),
            video_stream: None,
            audio_streams: vec![stereo("pcm_s24le")],
        },
    );
    let logo = asset(
        example_ids::IMAGE_ASSET,
        "logo.png",
        AssetKind::Image,
        MediaInfo {
            duration: None,
            container: "png_pipe".to_owned(),
            video_stream: Some(picture("png", 512, 512, None)),
            audio_streams: Vec::new(),
        },
    );
    vec![video, music, logo]
}

/// A deterministic fingerprint per file name (the sample hash only has to be valid hex).
fn fingerprint(name: &str) -> ContentFingerprint {
    let size = u64::try_from(name.len()).unwrap_or(0) * 1_000_000;
    let hash = format!("{:016x}", size + 0xabc);
    ContentFingerprint::new(size, 1_700_000_000_000, hash).expect("a 16-digit hex hash")
}

fn picture(codec: &str, width: u32, height: u32, frame_rate: Option<FrameRate>) -> VideoStreamInfo {
    VideoStreamInfo {
        codec: codec.to_owned(),
        profile: frame_rate.map(|_| "High".to_owned()),
        level: frame_rate.map(|_| 40),
        codec_tag: frame_rate.map(|_| "avc1".to_owned()),
        pixel_format: Some(
            if frame_rate.is_some() {
                "yuv420p"
            } else {
                "rgba"
            }
            .to_owned(),
        ),
        bit_depth: Some(8),
        chroma_subsampling: frame_rate.map(|_| "4:2:0".to_owned()),
        width,
        height,
        frame_rate,
        rotation: 0,
    }
}

fn example_tracks() -> Vec<Track> {
    vec![
        Track {
            transitions: vec![interview_dissolve()],
            ..track(1, TrackKind::Video, "V1", interview_clips())
        },
        track(2, TrackKind::Video, "V2", vec![title_text(), logo_image()]),
        track(
            3,
            TrackKind::Video,
            "V3",
            vec![lower_third(), backdrop_shape(), caption()],
        ),
        track(4, TrackKind::Audio, "A1", vec![interview_sound()]),
        track(5, TrackKind::Audio, "A2", vec![music()]),
    ]
}

fn interview_clips() -> Vec<Item> {
    let mut first = item(
        1,
        range_ms(0, 10_000),
        ItemKind::MediaClip(interview_clip()),
    );
    first.link_group = Some(LinkGroupId::from_test_index(1));
    let mut fast = interview_clip();
    fast.source_start = millis(20_000);
    fast.speed = 2.0;
    fast.reframe = Reframe::Fill {
        focus: Animatable::Keyframed {
            keyframes: vec![
                Keyframe {
                    time: millis(0),
                    value: Vec2::new(0.3, 0.5),
                    easing: Easing::Preset {
                        name: EasingPreset::EaseInOut,
                    },
                },
                Keyframe {
                    time: millis(400),
                    value: Vec2::new(0.7, 0.4),
                    easing: Easing::Linear,
                },
            ],
        },
    };
    let mut second = item(2, range_ms(10_000, 4000), ItemKind::MediaClip(fast));
    second.origin = Origin::AiTask {
        task_id: TaskId::from_test_index(1),
    };
    vec![first, second]
}

/// A centred one-second dissolve on the cut between the two interview clips of V1, at 10 s.
fn interview_dissolve() -> Transition {
    Transition {
        id: TransitionId::from_test_index(1),
        kind: TransitionKind::CrossDissolve,
        left_item: ItemId::from_test_index(1),
        right_item: ItemId::from_test_index(2),
        duration: millis(1000),
        alignment: TransitionAlignment::Center,
    }
}

fn interview_clip() -> MediaClip {
    MediaClip {
        source_start: millis(5000),
        fade_in: millis(500),
        ..MediaClip::new(example_ids::VIDEO_ASSET)
    }
}

fn interview_sound() -> Item {
    let mut sound = item(
        8,
        range_ms(0, 10_000),
        ItemKind::MediaClip(interview_clip()),
    );
    sound.link_group = Some(LinkGroupId::from_test_index(1));
    sound
}

fn title_text() -> Item {
    let transform = Transform2D {
        opacity: keyframed(
            &[(0, 0.0), (500, 1.0)],
            Easing::Preset {
                name: EasingPreset::EaseOut,
            },
        ),
        position: Animatable::constant(Vec2::new(0.5, 0.8)),
        ..Transform2D::default()
    };
    let text = TextItem {
        content: "Bonjour".to_owned(),
        style: TextStyle {
            font_size: Animatable::constant(DesignPixels(72.0)),
            stroke: Stroke {
                enabled: true,
                ..Stroke::default()
            },
            max_width: Some(DesignPixels(900.0)),
            ..TextStyle::default()
        },
        transform,
        reveal: Animatable::constant(1.0),
    };
    item(3, range_ms(1000, 3000), ItemKind::Text(text))
}

fn logo_image() -> Item {
    let image = ImageItem {
        asset_id: example_ids::IMAGE_ASSET,
        fit: ImageFit::Natural,
        transform: Transform2D {
            position: Animatable::constant(Vec2::new(0.9, 0.1)),
            scale: Animatable::constant(Vec2::splat(0.25)),
            ..Transform2D::default()
        },
    };
    item(4, range_ms(5000, 2000), ItemKind::Image(image))
}

/// Source of the example's lower third (`meta`, `params`, `render`).
pub const LOWER_THIRD_SOURCE: &str = r"export const meta = { name: 'Lower third', description: 'A name and a role sliding in', defaultDuration: 5 };

export const params = {
  name: { type: 'string', title: 'Nom', default: 'Jane Doe' },
  role: { type: 'string', title: 'Titre', default: 'Director', multiline: true },
  accent: { type: 'string', format: 'color', title: 'Accent', default: '#ffcc00' },
  slide: { type: 'number', title: 'Glissement', default: 120, minimum: 0, maximum: 400 },
  showLine: { type: 'boolean', title: 'Ligne', default: true },
  align: { type: 'string', title: 'Alignement', enum: ['left', 'right'], enumLabels: ['Gauche', 'Droite'], default: 'left' },
  font: { type: 'string', format: 'font', title: 'Police', default: 'Inter' },
  logo: { type: 'string', format: 'asset', title: 'Logo' },
};

export function render(ctx, time, p) {
  const enter = ctx.ease.outCubic(ctx.clamp(time / 0.6, 0, 1));
  const exit = ctx.ease.inCubic(ctx.clamp((time - ctx.duration + 0.5) / 0.5, 0, 1));
  ctx.globalAlpha = enter * (1 - exit);
  const x = p.align === 'left' ? 120 - p.slide * (1 - enter) : ctx.width - 120 + p.slide * (1 - enter);
  ctx.fillStyle = p.accent;
  if (p.showLine) ctx.fillRect(p.align === 'left' ? x - 24 : x + 12, 760, 12, 150);
  ctx.textAlign = p.align;
  ctx.fillStyle = '#ffffff';
  ctx.font = `800 72px ${p.font}`;
  ctx.fillText(p.name, x, 830);
  ctx.font = `500 44px ${p.font}`;
  ctx.fillText(p.role, x, 895);
}
";

fn lower_third_template() -> MotionTemplate {
    let param = |name: &str, label: &str, spec| TemplateParam {
        name: name.to_owned(),
        label: label.to_owned(),
        description: None,
        spec,
    };
    let choice = |value: &str, label: &str| TemplateChoice {
        value: value.to_owned(),
        label: label.to_owned(),
    };
    MotionTemplate {
        id: example_ids::TEMPLATE,
        name: "Lower third".to_owned(),
        description: "A name and a role sliding in".to_owned(),
        version: 1,
        source: LOWER_THIRD_SOURCE.to_owned(),
        params: vec![
            param(
                "name",
                "Nom",
                TemplateParamSpec::Text {
                    default: "Jane Doe".to_owned(),
                    multiline: false,
                    max_length: crate::DEFAULT_TEXT_MAX_LENGTH,
                },
            ),
            param(
                "role",
                "Titre",
                TemplateParamSpec::Text {
                    default: "Director".to_owned(),
                    multiline: true,
                    max_length: crate::DEFAULT_TEXT_MAX_LENGTH,
                },
            ),
            param(
                "accent",
                "Accent",
                TemplateParamSpec::Color {
                    default: Color::opaque(0xff, 0xcc, 0x00),
                },
            ),
            param(
                "slide",
                "Glissement",
                TemplateParamSpec::Number {
                    default: 120.0,
                    minimum: Some(0.0),
                    maximum: Some(400.0),
                    step: None,
                },
            ),
            param("showLine", "Ligne", TemplateParamSpec::Boolean { default: true }),
            param(
                "align",
                "Alignement",
                TemplateParamSpec::Enum {
                    default: "left".to_owned(),
                    options: vec![choice("left", "Gauche"), choice("right", "Droite")],
                },
            ),
            param(
                "font",
                "Police",
                TemplateParamSpec::Font {
                    default: "Inter".to_owned(),
                },
            ),
            param("logo", "Logo", TemplateParamSpec::Asset),
        ],
        default_duration: millis(5000),
        origin: Origin::AiTask {
            task_id: TaskId::from_test_index(1),
        },
    }
}

fn lower_third() -> Item {
    let params = BTreeMap::from([
        (
            "name".to_owned(),
            TemplateParamValue::Text {
                value: "Ada Lovelace".to_owned(),
            },
        ),
        (
            "accent".to_owned(),
            TemplateParamValue::Color {
                value: Animatable::constant(Color::opaque(0x33, 0x99, 0xff)),
            },
        ),
        (
            "slide".to_owned(),
            TemplateParamValue::Number {
                value: Animatable::Keyframed {
                    keyframes: vec![
                        Keyframe {
                            time: millis(0),
                            value: 200.0,
                            easing: Easing::Preset {
                                name: EasingPreset::EaseOut,
                            },
                        },
                        Keyframe {
                            time: millis(500),
                            value: 0.0,
                            easing: Easing::Linear,
                        },
                    ],
                },
            },
        ),
        (
            "showLine".to_owned(),
            TemplateParamValue::Boolean { value: false },
        ),
    ]);
    let template = TemplateItem {
        template_id: example_ids::TEMPLATE,
        params,
        transform: Transform2D {
            opacity: Animatable::constant(0.9),
            ..Transform2D::default()
        },
    };
    item(5, range_ms(2000, 3000), ItemKind::MotionTemplate(template))
}

fn backdrop_shape() -> Item {
    let shape = ShapeItem {
        geometry: ShapeGeometry::Rectangle {
            width: DesignPixels(600.0),
            height: DesignPixels(120.0),
            corner_radius: DesignPixels(16.0),
        },
        fill: Animatable::constant(Color {
            alpha: 0xcc,
            ..Color::BLACK
        }),
        stroke: Stroke::default(),
        cover_frame: false,
        transform: Transform2D::default(),
    };
    item(6, range_ms(6000, 2000), ItemKind::Shape(shape))
}

fn caption() -> Item {
    let caption = CaptionItem {
        words: vec![
            CaptionWord::new("Merci", millis(0), millis(400)),
            CaptionWord::new("d'être", millis(450), millis(900)),
            CaptionWord::new("là", millis(950), millis(1500)),
        ],
        style_id: example_ids::CAPTION_STYLE,
    };
    item(7, range_ms(8000, 2000), ItemKind::Caption(caption))
}

fn music() -> Item {
    let ducking = Easing::CubicBezier {
        x1: 0.42,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };
    let clip = MediaClip {
        volume_db: keyframed(&[(0, -6.0), (2000, -20.0)], ducking),
        fade_out: millis(1000),
        fade_out_shape: FadeShape::ConstantPower,
        ..MediaClip::new(example_ids::AUDIO_ASSET)
    };
    item(9, range_ms(0, 14_000), ItemKind::MediaClip(clip))
}

/// A keyframed number: `(time in ms, value)` pairs, all with `easing`.
fn keyframed(points: &[(i64, f64)], easing: Easing) -> Animatable<f64> {
    Animatable::Keyframed {
        keyframes: points
            .iter()
            .map(|&(time_ms, value)| Keyframe {
                time: millis(time_ms),
                value,
                easing,
            })
            .collect(),
    }
}
