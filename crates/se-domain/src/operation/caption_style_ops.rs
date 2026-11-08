//! Caption style operations (prompt 23): [`AddCaptionStyle`], [`UpdateCaptionStyle`],
//! [`RemoveCaptionStyle`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Assign, Edit, Effect,
    checks::{ensure_unused, insertion_index, swap_field},
    lookup::all_items,
};
use crate::{
    CaptionAnimation, CaptionBox, CaptionOutline, CaptionShadow, CaptionStyle, CaptionStyleId,
    Color, DesignPixels, DomainError, ItemKind, Project, TextCase, Vec2,
};

/// Index of the caption style `id`.
fn find_caption_style(project: &Project, id: CaptionStyleId) -> Result<usize, DomainError> {
    project
        .caption_styles
        .iter()
        .position(|style| style.id == id)
        .ok_or(DomainError::CaptionStyleNotFound { style: id })
}

/// Adds a caption style. Inverse: [`RemoveCaptionStyle`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddCaptionStyle {
    /// Position in the project's caption styles (`null` appends).
    pub index: Option<usize>,
    /// The style, with an unused id.
    pub style: CaptionStyle,
}

impl Edit for AddCaptionStyle {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let id = self.style.id;
        ensure_unused(
            find_caption_style(project, id).is_ok(),
            "caption style",
            &id,
        )?;
        let index = insertion_index("caption style", self.index, project.caption_styles.len())?;
        project.caption_styles.insert(index, self.style.clone());
        effect.touched.caption_styles.insert(id);
        effect.undo_with(RemoveCaptionStyle { style: id });
        Ok(())
    }
}

/// Changes of a caption style; `null` (or absent) fields are unchanged.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct CaptionStylePatch {
    /// Another origin preset (`{"value": null}` clears it).
    pub preset: Option<Assign<Option<String>>>,
    /// Another font family.
    pub font_family: Option<String>,
    /// Another font weight (1–1000).
    pub font_weight: Option<u16>,
    /// Italic or not.
    pub italic: Option<bool>,
    /// Another font size (> 0).
    pub font_size: Option<DesignPixels>,
    /// Another letter case.
    pub text_case: Option<TextCase>,
    /// Another line height, in `(0, 10]`.
    pub line_height: Option<f64>,
    /// Another colour of words not spoken yet.
    pub text_color: Option<Color>,
    /// Another colour of the active word.
    pub active_color: Option<Color>,
    /// Another colour of emphasised words.
    pub emphasis_color: Option<Color>,
    /// Another outline.
    pub outline: Option<CaptionOutline>,
    /// Another shadow.
    pub shadow: Option<CaptionShadow>,
    /// Another plate.
    pub background: Option<CaptionBox>,
    /// Another position of the block's bottom centre.
    pub position: Option<Vec2>,
    /// Another animation.
    pub animation: Option<CaptionAnimation>,
    /// Whether spoken words keep the active colour.
    pub highlight_spoken: Option<bool>,
    /// Another longest line, in characters.
    pub max_chars_per_line: Option<u32>,
    /// Another number of lines shown at once.
    pub max_lines: Option<u32>,
}

impl CaptionStylePatch {
    /// Applies the patch and returns the one undoing it.
    #[must_use]
    pub fn apply(&self, style: &mut CaptionStyle) -> Self {
        Self {
            preset: self.preset.as_ref().map(|assign| Assign {
                value: std::mem::replace(&mut style.preset, assign.value.clone()),
            }),
            font_family: swap_field(&mut style.font_family, self.font_family.as_ref()),
            font_weight: swap_field(&mut style.font_weight, self.font_weight.as_ref()),
            italic: swap_field(&mut style.italic, self.italic.as_ref()),
            font_size: swap_field(&mut style.font_size, self.font_size.as_ref()),
            text_case: swap_field(&mut style.text_case, self.text_case.as_ref()),
            line_height: swap_field(&mut style.line_height, self.line_height.as_ref()),
            text_color: swap_field(&mut style.text_color, self.text_color.as_ref()),
            active_color: swap_field(&mut style.active_color, self.active_color.as_ref()),
            emphasis_color: swap_field(&mut style.emphasis_color, self.emphasis_color.as_ref()),
            outline: swap_field(&mut style.outline, self.outline.as_ref()),
            shadow: swap_field(&mut style.shadow, self.shadow.as_ref()),
            background: swap_field(&mut style.background, self.background.as_ref()),
            position: swap_field(&mut style.position, self.position.as_ref()),
            animation: swap_field(&mut style.animation, self.animation.as_ref()),
            highlight_spoken: swap_field(
                &mut style.highlight_spoken,
                self.highlight_spoken.as_ref(),
            ),
            max_chars_per_line: swap_field(
                &mut style.max_chars_per_line,
                self.max_chars_per_line.as_ref(),
            ),
            max_lines: swap_field(&mut style.max_lines, self.max_lines.as_ref()),
        }
    }

    /// Whether the patch changes nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Changes a caption style (every caption using it changes look).
/// Inverse: [`UpdateCaptionStyle`] with the previous values of the changed fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCaptionStyle {
    /// The style changed.
    pub style: CaptionStyleId,
    /// What changes.
    pub patch: CaptionStylePatch,
}

impl Edit for UpdateCaptionStyle {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_caption_style(project, self.style)?;
        let previous = self.patch.apply(&mut project.caption_styles[index]);
        effect.touched.caption_styles.insert(self.style);
        effect.undo_with(Self {
            style: self.style,
            patch: previous,
        });
        Ok(())
    }
}

/// Removes a caption style no caption uses. Inverse: [`AddCaptionStyle`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveCaptionStyle {
    /// The style removed.
    pub style: CaptionStyleId,
}

impl Edit for RemoveCaptionStyle {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_caption_style(project, self.style)?;
        let user = all_items(project).find(
            |item| matches!(&item.kind, ItemKind::Caption(caption) if caption.style_id == self.style),
        );
        if let Some(user) = user {
            return Err(DomainError::CaptionStyleInUse {
                style: self.style,
                item: user.id,
            });
        }
        let style = project.caption_styles.remove(index);
        effect.touched.caption_styles.insert(self.style);
        effect.undo_with(AddCaptionStyle {
            index: Some(index),
            style,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DomainErrorCode,
        operation::testing::{apply_err, apply_ok, example},
        test_support::example_ids,
    };

    fn new_style(index: u64) -> CaptionStyle {
        crate::caption_style_from_preset("box", CaptionStyleId::from_test_index(index)).unwrap()
    }

    #[test]
    fn adds_and_removes_an_unused_style() {
        let applied = apply_ok(
            &example(),
            AddCaptionStyle {
                index: Some(0),
                style: new_style(50),
            },
        );
        assert_eq!(applied.project.caption_styles[0].id, new_style(50).id);
        assert!(applied.touched.caption_styles.contains(&new_style(50).id));
        let removed = apply_ok(
            &applied.project,
            RemoveCaptionStyle {
                style: new_style(50).id,
            },
        );
        assert_eq!(removed.project.caption_styles, example().caption_styles);
    }

    #[test]
    fn updates_only_the_given_fields() {
        let style = example_ids::CAPTION_STYLE;
        let patch = CaptionStylePatch {
            font_size: Some(DesignPixels(90.0)),
            animation: Some(CaptionAnimation::RevealPerWord),
            ..CaptionStylePatch::default()
        };
        let applied = apply_ok(&example(), UpdateCaptionStyle { style, patch });
        let updated = &applied.project.caption_styles[0];
        assert_eq!(updated.font_size, DesignPixels(90.0));
        assert_eq!(updated.animation, CaptionAnimation::RevealPerWord);
        assert_eq!(updated.font_family, example().caption_styles[0].font_family);
    }

    #[test]
    fn refuses_used_unknown_or_duplicate_styles() {
        let used = example_ids::CAPTION_STYLE;
        let error = apply_err(&example(), RemoveCaptionStyle { style: used });
        assert_eq!(error.code(), DomainErrorCode::CaptionStyleInUse);
        let unknown = CaptionStyleId::from_test_index(77);
        let error = apply_err(
            &example(),
            UpdateCaptionStyle {
                style: unknown,
                patch: CaptionStylePatch::default(),
            },
        );
        assert_eq!(error.code(), DomainErrorCode::CaptionStyleNotFound);
        let mut duplicate = new_style(1);
        duplicate.id = used;
        let error = apply_err(
            &example(),
            AddCaptionStyle {
                index: None,
                style: duplicate,
            },
        );
        assert_eq!(error.code(), DomainErrorCode::IdAlreadyUsed);
    }

    #[test]
    fn an_invalid_style_is_refused_by_validation() {
        let patch = CaptionStylePatch {
            max_lines: Some(12),
            ..CaptionStylePatch::default()
        };
        let error = apply_err(
            &example(),
            UpdateCaptionStyle {
                style: example_ids::CAPTION_STYLE,
                patch,
            },
        );
        assert_eq!(error.code(), DomainErrorCode::InvalidDocument);
    }
}
