//! Rules of the project's brand kit (prompt 25): `INVALID_BRAND_KIT` for its own values
//! (`BrandKit::problems`), `UNKNOWN_REFERENCE` and `ASSET_KIND_MISMATCH` for its logos.

use super::{Path, ProjectIndex, Report, ViolationCode};
use crate::{AssetKind, Project};

/// Checks the project's brand kit, if it has one.
pub(super) fn check(project: &Project, index: &ProjectIndex<'_>, report: &mut Report) {
    let Some(kit) = &project.brand_kit else {
        return;
    };
    let path = Path::default().field("brandKit");
    for (field, rule) in kit.problems() {
        let field_path = if field.is_empty() {
            path.clone()
        } else {
            path.field(&field)
        };
        report.push(
            ViolationCode::InvalidBrandKit,
            &field_path,
            format!("brand kit « {} »: {field} {rule}", kit.name),
        );
    }
    for (position, logo) in kit.logos.iter().enumerate() {
        let logo_path = path.field("logos").key(position).field("assetId");
        match index.asset(logo.asset_id) {
            None => report.push(
                ViolationCode::UnknownReference,
                &logo_path,
                format!(
                    "logo « {} » refers to asset {}, which does not exist in this project; import \
                     the image first",
                    logo.name, logo.asset_id
                ),
            ),
            Some(asset) if asset.kind != AssetKind::Image => report.push(
                ViolationCode::AssetKindMismatch,
                &logo_path,
                format!(
                    "logo « {} » refers to asset {}, which is not an image",
                    logo.name, logo.asset_id
                ),
            ),
            Some(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AssetId, Project, ViolationCode,
        test_support::{example_ids, example_project},
        validate_project,
    };

    fn codes(project: &Project) -> Vec<(ViolationCode, String)> {
        validate_project(project)
            .into_iter()
            .map(|violation| (violation.code, violation.path))
            .collect()
    }

    fn kit(project: &mut Project) -> &mut crate::BrandKit {
        project.brand_kit.as_mut().unwrap()
    }

    #[test]
    fn reports_unusable_kit_values() {
        let mut project = example_project();
        kit(&mut project).fonts.heading = String::new();
        assert_eq!(
            codes(&project),
            [(
                ViolationCode::InvalidBrandKit,
                "brandKit.fonts.heading".to_owned()
            )]
        );
    }

    #[test]
    fn logos_must_be_images_of_the_project() {
        let mut project = example_project();
        kit(&mut project).logos[0].asset_id = AssetId::from_test_index(99);
        assert_eq!(
            codes(&project),
            [(
                ViolationCode::UnknownReference,
                "brandKit.logos[0].assetId".to_owned()
            )]
        );
        kit(&mut project).logos[0].asset_id = example_ids::VIDEO_ASSET;
        assert_eq!(codes(&project)[0].0, ViolationCode::AssetKindMismatch);
    }
}
