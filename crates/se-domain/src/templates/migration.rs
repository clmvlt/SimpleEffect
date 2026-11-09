//! [`migrate_params`]: the values of a template item carried over to a new version of its
//! template (ADR-0035).

use std::collections::BTreeMap;

use crate::{MotionTemplate, TemplateParamValue};

/// Values of an item checked against a new version of its template.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamMigration {
    /// The values that still fit (same name, same kind, within the new constraints).
    pub params: BTreeMap<String, TemplateParamValue>,
    /// Names whose value was dropped: the item now uses the new default (the parameter is gone,
    /// changed kind, or the value breaks a new constraint).
    pub reset: Vec<String>,
}

impl ParamMigration {
    /// Whether every value carried over.
    #[must_use]
    pub fn is_lossless(&self) -> bool {
        self.reset.is_empty()
    }
}

/// Keeps the values of `params` that fit `template` and lists the others, which fall back to the
/// template's defaults.
#[must_use]
pub fn migrate_params(
    params: &BTreeMap<String, TemplateParamValue>,
    template: &MotionTemplate,
) -> ParamMigration {
    let mut kept = BTreeMap::new();
    let mut reset = Vec::new();
    for (name, value) in params {
        let fits = template
            .param(name)
            .is_some_and(|param| param.value_problem(value).is_none());
        if fits {
            kept.insert(name.clone(), value.clone());
        } else {
            reset.push(name.clone());
        }
    }
    ParamMigration {
        params: kept,
        reset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Animatable, Origin, TemplateId, TemplateParam, TemplateParamSpec, time::Flicks};

    fn template(params: Vec<TemplateParam>) -> MotionTemplate {
        MotionTemplate {
            id: TemplateId::from_test_index(1),
            name: "Title".to_owned(),
            description: String::new(),
            version: 2,
            source: "export function render() {}".to_owned(),
            params,
            default_duration: Flicks::one_second(),
            origin: Origin::User,
        }
    }

    fn param(name: &str, spec: TemplateParamSpec) -> TemplateParam {
        TemplateParam {
            name: name.to_owned(),
            label: name.to_owned(),
            description: None,
            spec,
        }
    }

    #[test]
    fn keeps_fitting_values_and_resets_the_others() {
        let new_version = template(vec![
            param(
                "title",
                TemplateParamSpec::Text {
                    default: String::new(),
                    multiline: false,
                    max_length: 100,
                },
            ),
            param(
                "size",
                TemplateParamSpec::Number {
                    default: 10.0,
                    minimum: Some(0.0),
                    maximum: Some(50.0),
                    step: None,
                },
            ),
            param(
                "shown",
                TemplateParamSpec::Text {
                    default: String::new(),
                    multiline: false,
                    max_length: 10,
                },
            ),
        ]);
        let params = BTreeMap::from([
            (
                "title".to_owned(),
                TemplateParamValue::Text {
                    value: "Hi".to_owned(),
                },
            ),
            (
                "size".to_owned(),
                TemplateParamValue::Number {
                    value: Animatable::constant(80.0),
                },
            ),
            (
                "shown".to_owned(),
                TemplateParamValue::Boolean { value: true },
            ),
            (
                "gone".to_owned(),
                TemplateParamValue::Boolean { value: false },
            ),
        ]);
        let migration = migrate_params(&params, &new_version);
        assert_eq!(migration.params.keys().collect::<Vec<_>>(), ["title"]);
        assert_eq!(migration.reset, ["gone", "shown", "size"]);
        assert!(!migration.is_lossless());
    }
}
