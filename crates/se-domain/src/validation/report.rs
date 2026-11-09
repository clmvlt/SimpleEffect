//! Shared plumbing of the rules: violation paths, the report collecting them, the id index and
//! time descriptions for messages.

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use super::{Violation, ViolationCode};
use crate::{Asset, AssetId, CaptionStyleId, Flicks, MotionTemplate, Project, TemplateId};

/// A location in the document, written like JS property access with element ids in brackets.
#[derive(Debug, Clone, Default)]
pub(super) struct Path(String);

impl Path {
    /// The path of a field of this location (`a` → `a.name`).
    pub(super) fn field(&self, name: &str) -> Self {
        if self.0.is_empty() {
            Self(name.to_owned())
        } else {
            Self(format!("{}.{name}", self.0))
        }
    }

    /// The path of an element of this collection (`a` → `a[key]`).
    pub(super) fn key(&self, key: impl Display) -> Self {
        Self(format!("{}[{key}]", self.0))
    }
}

/// Collects violations in traversal order, so reports are deterministic.
#[derive(Debug, Default)]
pub(super) struct Report(Vec<Violation>);

impl Report {
    /// Records a violation.
    pub(super) fn push(&mut self, code: ViolationCode, path: &Path, message: String) {
        self.0.push(Violation {
            code,
            path: path.0.clone(),
            message,
        });
    }

    /// The violations recorded so far.
    pub(super) fn into_violations(self) -> Vec<Violation> {
        self.0
    }
}

/// Entities that items refer to, by id.
pub(super) struct ProjectIndex<'a> {
    assets: HashMap<AssetId, &'a Asset>,
    templates: HashMap<TemplateId, &'a MotionTemplate>,
    caption_styles: HashSet<CaptionStyleId>,
}

impl<'a> ProjectIndex<'a> {
    /// Indexes the project. With duplicate ids (reported separately), the first entity wins.
    pub(super) fn new(project: &'a Project) -> Self {
        let mut assets = HashMap::new();
        for asset in &project.assets {
            assets.entry(asset.id).or_insert(asset);
        }
        let mut templates = HashMap::new();
        for template in &project.templates {
            templates.entry(template.id).or_insert(template);
        }
        Self {
            assets,
            templates,
            caption_styles: project
                .caption_styles
                .iter()
                .map(|style| style.id)
                .collect(),
        }
    }

    /// The asset with this id, if the project has one.
    pub(super) fn asset(&self, id: AssetId) -> Option<&'a Asset> {
        self.assets.get(&id).copied()
    }

    /// The duration of the asset with this id, when the asset exists and its duration is known.
    pub(super) fn asset_duration(&self, id: AssetId) -> Option<Flicks> {
        self.asset(id)
            .and_then(|asset| asset.media_info.as_ref())
            .and_then(|info| info.duration)
    }

    /// Whether the project has a caption style with this id.
    pub(super) fn has_caption_style(&self, id: CaptionStyleId) -> bool {
        self.caption_styles.contains(&id)
    }

    /// The template with this id, if the project has one.
    pub(super) fn template(&self, id: TemplateId) -> Option<&'a MotionTemplate> {
        self.templates.get(&id).copied()
    }
}

/// A time for messages, in seconds with millisecond precision (`12.500s`).
pub(super) fn seconds(time: Flicks) -> String {
    format!("{:.3}s", time.to_seconds_f64())
}
