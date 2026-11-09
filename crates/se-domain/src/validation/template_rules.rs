//! Rules of motion templates (prompt 24, ADR-0035): `INVALID_TEMPLATE` for a template's
//! definition, and for template items `UNKNOWN_REFERENCE` (template, image asset),
//! `ASSET_KIND_MISMATCH`, `INVALID_TEMPLATE_PARAMS` and the keyframe rules of animated values.

use super::{
    Path, Report, ViolationCode, animation_rules::check_keyframes, item_rules::ItemContext,
};
use crate::{AssetKind, Project, TemplateItem, TemplateParamValue};

/// Every template of the project has a usable definition.
pub(super) fn check_templates(project: &Project, report: &mut Report) {
    let templates = Path::default().field("templates");
    for template in &project.templates {
        let path = templates.key(template.id);
        for (field, rule) in template.problems() {
            report.push(
                ViolationCode::InvalidTemplate,
                &path.field(&field),
                format!("template « {} »: {field} {rule}", template.name),
            );
        }
    }
}

/// A template item refers to an existing template and holds values its parameters accept.
pub(super) fn check_template_item(
    item: &TemplateItem,
    context: &ItemContext<'_>,
    path: &Path,
    report: &mut Report,
) {
    let Some(template) = context.index.template(item.template_id) else {
        report.push(
            ViolationCode::UnknownReference,
            &path.field("templateId"),
            format!(
                "template {} does not exist in this project; use the id of an existing template",
                item.template_id
            ),
        );
        return;
    };
    let duration = context.item.timeline_range.duration();
    for (name, value) in &item.params {
        let value_path = path.field("params").field(name);
        let Some(param) = template.param(name) else {
            report.push(
                ViolationCode::InvalidTemplateParams,
                &value_path,
                format!(
                    "template « {} » has no parameter '{name}'; its parameters are: {}",
                    template.name,
                    parameter_names(&template.params)
                ),
            );
            continue;
        };
        if let Some(problem) = param.value_problem(value) {
            report.push(
                ViolationCode::InvalidTemplateParams,
                &value_path,
                format!("parameter '{name}' of « {} »: {problem}", template.name),
            );
        }
        let inner = value_path.field("value");
        match value {
            TemplateParamValue::Number { value } => {
                check_keyframes(value, &inner, duration, report)
            }
            TemplateParamValue::Color { value } => check_keyframes(value, &inner, duration, report),
            TemplateParamValue::Asset { value: Some(asset) } => {
                check_image(*asset, context, &inner, report);
            }
            _ => {}
        }
    }
}

fn check_image(asset: crate::AssetId, context: &ItemContext<'_>, path: &Path, report: &mut Report) {
    match context.index.asset(asset) {
        None => report.push(
            ViolationCode::UnknownReference,
            path,
            format!("asset {asset} does not exist in this project; pick one of its images"),
        ),
        Some(found) if found.kind != AssetKind::Image => report.push(
            ViolationCode::AssetKindMismatch,
            path,
            format!("asset {asset} is not an image; template asset parameters take images"),
        ),
        Some(_) => {}
    }
}

fn parameter_names(params: &[crate::TemplateParam]) -> String {
    if params.is_empty() {
        return "none".to_owned();
    }
    params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::{
        Animatable, AssetId, ItemKind, Project, TemplateItem, TemplateParamValue, ViolationCode,
        test_support::{example_ids, example_project},
        validate_project,
    };

    fn codes(project: &Project) -> Vec<ViolationCode> {
        validate_project(project)
            .into_iter()
            .map(|violation| violation.code)
            .collect()
    }

    /// The template item of V3 (`[2 s, 3 s)`).
    fn instance(project: &mut Project) -> &mut TemplateItem {
        match &mut project.sequences[0].tracks[2].items[0].kind {
            ItemKind::MotionTemplate(template) => template,
            _ => unreachable!("the example's first V3 item is a template"),
        }
    }

    #[test]
    fn reports_broken_definitions() {
        let mut project = example_project();
        project.templates[0].name = String::new();
        project.templates[0].version = 0;
        assert_eq!(codes(&project), [ViolationCode::InvalidTemplate; 2]);
    }

    #[test]
    fn reports_values_parameters_refuse() {
        let mut project = example_project();
        let params = &mut instance(&mut project).params;
        params.insert(
            "unknown".to_owned(),
            TemplateParamValue::Boolean { value: true },
        );
        params.insert(
            "slide".to_owned(),
            TemplateParamValue::Number {
                value: Animatable::constant(10_000.0),
            },
        );
        params.insert(
            "name".to_owned(),
            TemplateParamValue::Boolean { value: false },
        );
        assert_eq!(codes(&project), [ViolationCode::InvalidTemplateParams; 3]);
    }

    #[test]
    fn reports_asset_values_that_are_not_images() {
        let mut project = example_project();
        let video = project.assets[0].id;
        let params = &mut instance(&mut project).params;
        params.insert(
            "logo".to_owned(),
            TemplateParamValue::Asset { value: Some(video) },
        );
        assert_eq!(codes(&project), [ViolationCode::AssetKindMismatch]);
        let params = &mut instance(&mut project).params;
        params.insert(
            "logo".to_owned(),
            TemplateParamValue::Asset {
                value: Some(AssetId::from_test_index(99)),
            },
        );
        assert_eq!(codes(&project), [ViolationCode::UnknownReference]);
    }

    #[test]
    fn keyframes_of_values_follow_the_keyframe_rules() {
        let mut project = example_project();
        let params = &mut instance(&mut project).params;
        params.insert(
            "slide".to_owned(),
            TemplateParamValue::Number {
                value: Animatable::Keyframed {
                    keyframes: Vec::new(),
                },
            },
        );
        assert_eq!(codes(&project), [ViolationCode::EmptyKeyframes]);
        assert_eq!(project.templates[0].id, example_ids::TEMPLATE);
    }
}
