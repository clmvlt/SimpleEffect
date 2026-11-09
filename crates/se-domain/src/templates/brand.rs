//! [`brand_values`]: what a brand kit gives a new instance of a template (prompt 25).

use std::collections::BTreeMap;

use crate::{BrandKit, MotionTemplate, TemplateParamValue};

/// The values a new instance of `template` starts with in a project using `kit`: for each
/// parameter bound to a brand role (`brand` in its schema), the kit's value, when the kit has one
/// and the parameter accepts it. Values the caller gives explicitly win over these (merge them on
/// top). Without a kit, nothing: instances keep the schema defaults.
///
/// Instances keep these values as ordinary data: changing the project's kit later does not
/// restyle them (like texts made with the previous kit).
#[must_use]
pub fn brand_values(
    template: &MotionTemplate,
    kit: Option<&BrandKit>,
) -> BTreeMap<String, TemplateParamValue> {
    let Some(kit) = kit else {
        return BTreeMap::new();
    };
    template
        .params
        .iter()
        .filter_map(|param| {
            let value = kit.value_for(param.brand?)?;
            param
                .value_problem(&value)
                .is_none()
                .then(|| (param.name.clone(), value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Animatable, BrandRole, Color, TemplateParam, TemplateParamSpec,
        test_support::example_project,
    };

    fn template_with(params: Vec<TemplateParam>) -> MotionTemplate {
        let mut template = example_project().templates[0].clone();
        template.params = params;
        template
    }

    fn param(name: &str, spec: TemplateParamSpec, brand: Option<BrandRole>) -> TemplateParam {
        TemplateParam {
            name: name.to_owned(),
            label: name.to_owned(),
            description: None,
            spec,
            brand,
        }
    }

    #[test]
    fn bound_parameters_take_the_kits_values() {
        let kit = example_project().brand_kit.unwrap();
        let template = template_with(vec![
            param(
                "accent",
                TemplateParamSpec::Color {
                    default: Color::WHITE,
                },
                Some(BrandRole::Accent),
            ),
            param(
                "font",
                TemplateParamSpec::Font {
                    default: "Inter".to_owned(),
                },
                Some(BrandRole::Heading),
            ),
            param("logo", TemplateParamSpec::Asset, Some(BrandRole::Logo)),
            param(
                "plain",
                TemplateParamSpec::Color {
                    default: Color::BLACK,
                },
                None,
            ),
        ]);
        let values = brand_values(&template, Some(&kit));
        assert_eq!(
            values.get("accent"),
            Some(&TemplateParamValue::Color {
                value: Animatable::constant(kit.colors.accent)
            })
        );
        assert_eq!(
            values.get("font"),
            Some(&TemplateParamValue::Font {
                value: kit.fonts.heading.clone()
            })
        );
        assert_eq!(
            values.get("logo"),
            Some(&TemplateParamValue::Asset {
                value: Some(kit.logos[0].asset_id)
            })
        );
        assert!(!values.contains_key("plain"));
        assert!(brand_values(&template, None).is_empty());
    }

    #[test]
    fn a_kit_without_logo_leaves_the_logo_unset() {
        let mut kit = example_project().brand_kit.unwrap();
        kit.logos.clear();
        let template = template_with(vec![param(
            "logo",
            TemplateParamSpec::Asset,
            Some(BrandRole::Logo),
        )]);
        assert!(brand_values(&template, Some(&kit)).is_empty());
    }
}
