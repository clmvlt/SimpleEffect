//! Rules of the static style values of texts and shapes: `VALUE_OUT_OF_RANGE` for an empty font
//! family, weights outside 1–1000, line heights, wrapping widths, paddings, corner radii and shape
//! extents that are not positive (animatable values are checked by `animation_rules`).

use super::{Path, Report, ViolationCode};
use crate::{DesignPixels, ShapeGeometry, ShapeItem, TextItem, TextStyle};

/// Largest line height accepted, as a multiple of the font size.
const MAX_LINE_HEIGHT: f64 = 10.0;

/// Checks the static style of a text (paths relative to `kind_path`).
pub(super) fn check_text(text: &TextItem, kind_path: &Path, report: &mut Report) {
    let style = &text.style;
    let path = kind_path.field("style");
    if style.font_family.trim().is_empty() {
        report.push(
            ViolationCode::ValueOutOfRange,
            &path.field("fontFamily"),
            format!(
                "the font family is empty; name a font of the catalogue, e.g. '{}'",
                TextItem::DEFAULT_FONT_FAMILY
            ),
        );
    }
    if !TextStyle::WEIGHT_RANGE.contains(&style.font_weight) {
        report.push(
            ViolationCode::ValueOutOfRange,
            &path.field("fontWeight"),
            format!(
                "font weight must be between 1 and 1000 (400 regular, 700 bold), got {}",
                style.font_weight
            ),
        );
    }
    if !(style.line_height > 0.0 && style.line_height <= MAX_LINE_HEIGHT) {
        report.push(
            ViolationCode::ValueOutOfRange,
            &path.field("lineHeight"),
            format!(
                "line height is a multiple of the font size above 0 and at most \
                 {MAX_LINE_HEIGHT}, got {}",
                style.line_height
            ),
        );
    }
    if let Some(width) = style.max_width {
        positive(width, "maxWidth", &path.field("maxWidth"), report);
    }
    let background = path.field("background");
    let padding = style.background.padding;
    for (axis, value) in [("x", padding.x), ("y", padding.y)] {
        non_negative(
            DesignPixels(value),
            "background padding",
            &background.field("padding").field(axis),
            report,
        );
    }
    non_negative(
        style.background.corner_radius,
        "background corner radius",
        &background.field("cornerRadius"),
        report,
    );
}

/// Checks the geometry of a shape (paths relative to `kind_path`).
pub(super) fn check_shape(shape: &ShapeItem, kind_path: &Path, report: &mut Report) {
    let path = kind_path.field("geometry");
    for (name, extent) in shape.geometry.extents() {
        positive(extent, name, &path.field(name), report);
    }
    if let ShapeGeometry::Rectangle { corner_radius, .. } = shape.geometry {
        non_negative(
            corner_radius,
            "corner radius",
            &path.field("cornerRadius"),
            report,
        );
    }
}

fn positive(value: DesignPixels, name: &str, path: &Path, report: &mut Report) {
    if !(value.0 > 0.0 && value.0.is_finite()) {
        report.push(
            ViolationCode::ValueOutOfRange,
            path,
            format!("{name} must be a size above 0 px, got {}", value.0),
        );
    }
}

fn non_negative(value: DesignPixels, name: &str, path: &Path, report: &mut Report) {
    if !(value.0 >= 0.0 && value.0.is_finite()) {
        report.push(
            ViolationCode::ValueOutOfRange,
            path,
            format!("{name} must be 0 px or more, got {}", value.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DesignPixels, ItemKind, Project, ShapeGeometry, ViolationCode,
        test_support::example_project, validate_project,
    };

    fn text_style(project: &mut Project) -> &mut crate::TextStyle {
        match &mut project.sequences[0].tracks[1].items[0].kind {
            ItemKind::Text(text) => &mut text.style,
            _ => unreachable!("the example's first V2 item is a text"),
        }
    }

    fn paths(project: &Project) -> Vec<String> {
        validate_project(project)
            .into_iter()
            .inspect(|violation| assert_eq!(violation.code, ViolationCode::ValueOutOfRange))
            .map(|violation| violation.path)
            .collect()
    }

    #[test]
    fn reports_invalid_text_styles() {
        let mut project = example_project();
        let style = text_style(&mut project);
        style.font_family = "  ".to_owned();
        style.font_weight = 0;
        style.line_height = 0.0;
        style.max_width = Some(DesignPixels(-5.0));
        style.background.padding.y = -1.0;
        style.background.corner_radius = DesignPixels(f64::INFINITY);
        let paths = paths(&project);
        let suffixes = [
            "style.fontFamily",
            "style.fontWeight",
            "style.lineHeight",
            "style.maxWidth",
            "style.background.padding.y",
            "style.background.cornerRadius",
        ];
        assert_eq!(paths.len(), suffixes.len(), "{paths:#?}");
        for (path, suffix) in paths.iter().zip(suffixes) {
            assert!(path.ends_with(suffix), "{path} should end with {suffix}");
        }
    }

    #[test]
    fn reports_shapes_without_extent() {
        let mut project = example_project();
        let ItemKind::Shape(shape) = &mut project.sequences[0].tracks[2].items[1].kind else {
            unreachable!("the example's second V3 item is a shape")
        };
        shape.geometry = ShapeGeometry::Line {
            length: DesignPixels(0.0),
            thickness: DesignPixels(4.0),
        };
        let paths = paths(&project);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("geometry.length"));
    }
}
