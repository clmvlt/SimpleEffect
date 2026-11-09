//! Brand kits (prompt 25, ADR-0036): kits of the library with their logos, the copy a project
//! uses (logos imported as assets), the defaults new texts, captions and template instances take
//! from it, open projects following a saved kit, and a project's own kit added to the library.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use se_app::{
    brand_kits::labels,
    test_support::{MediaTestbed, probed_image, probed_video, settle},
};
use se_domain::{
    AssetKind, Color, ItemKind, Project, ProjectId, SequenceId, TemplateParamValue,
    test_support::{example_ids, example_project},
};
use se_protocol::{AddGraphicRequest, CoreEvent, ErrorCode, LibraryBrandKit, NewGraphic};

const PROJECT_PATH: &str = "/films/Example.seproj";
const LOGO: &str = "/images/acme.png";

fn secs(seconds: f64) -> se_domain::Flicks {
    se_domain::Flicks::from_seconds_f64(seconds).unwrap()
}

struct Bed {
    services: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
}

/// The example project without its kit, and a PNG logo on disk.
async fn bed() -> Bed {
    let services = MediaTestbed::new(Path::new("/films"));
    let mut project = example_project();
    project.brand_kit = None;
    for (content, asset) in (1..).zip(&project.assets) {
        let probed = if asset.kind == AssetKind::Image {
            probed_image()
        } else {
            probed_video()
        };
        services
            .files
            .add(asset.original_path.clone(), content, probed);
    }
    services.files.add(LOGO, 77, probed_image());
    services.files.add("/images/clip.mp4", 78, probed_video());
    services
        .repository
        .insert(PathBuf::from(PROJECT_PATH), project.clone(), Vec::new());
    let session = services
        .projects
        .open_project(Path::new(PROJECT_PATH))
        .await
        .unwrap();
    settle().await;
    Bed {
        project_id: session.snapshot.project.id,
        sequence_id: project.sequences[0].id,
        services,
    }
}

impl Bed {
    async fn project(&self) -> Arc<Project> {
        self.services
            .projects
            .project(self.project_id)
            .await
            .unwrap()
    }

    async fn kit_with_logo(&self) -> LibraryBrandKit {
        let kit = self.services.brand_kits.create("Acme", None).await.unwrap();
        self.services
            .brand_kits
            .add_logo(kit.id, Path::new(LOGO))
            .await
            .unwrap()
    }

    async fn undo_label(&self) -> Option<String> {
        self.services
            .projects
            .project_session(self.project_id)
            .await
            .unwrap()
            .history
            .undo_label
    }
}

#[tokio::test(start_paused = true)]
async fn kits_are_created_with_defaults_and_take_image_logos_only() {
    let bed = bed().await;
    let kit = bed.kit_with_logo().await;
    assert_eq!((kit.name.as_str(), kit.version), ("Acme", 2));
    assert_eq!(kit.fonts.heading, "Montserrat");
    assert_eq!(kit.logos.len(), 1);
    assert_eq!(kit.logos[0].name, "acme");
    assert!(
        kit.logos[0].file.starts_with("logos/")
            && Path::new(&kit.logos[0].file).extension() == Some("png".as_ref())
    );
    assert_eq!(
        bed.services.brand_kit_library.logo_files(kit.id),
        [kit.logos[0].file.clone()]
    );
    let refused = bed
        .services
        .brand_kits
        .add_logo(kit.id, Path::new("/images/clip.mp4"))
        .await
        .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::InvalidLibraryFile);

    let copy = bed
        .services
        .brand_kits
        .create("Acme 2", Some(kit.id))
        .await
        .unwrap();
    assert_eq!(copy.version, 1);
    assert_eq!(copy.colors, kit.colors);
    assert_ne!(
        copy.logos[0].file, kit.logos[0].file,
        "its own copy of the logo"
    );

    let mut broken = kit.clone();
    broken.name = String::new();
    let error = bed.services.brand_kits.save(&broken).await.unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidBrandKit);
}

#[tokio::test(start_paused = true)]
async fn projects_use_a_copy_whose_values_style_new_graphics() {
    let bed = bed().await;
    let kit = bed.kit_with_logo().await;
    bed.services
        .brand_kits
        .set_project_kit(bed.project_id, Some(kit.id))
        .await
        .unwrap();
    assert_eq!(
        bed.undo_label().await.as_deref(),
        Some(labels::CHOOSE_BRAND_KIT)
    );
    let project = bed.project().await;
    let copy = project.brand_kit.clone().unwrap();
    assert_eq!(copy.source.unwrap().kit_id, kit.id);
    let logo = project
        .assets
        .iter()
        .find(|asset| asset.id == copy.logos[0].asset_id)
        .unwrap();
    assert_eq!(logo.kind, AssetKind::Image);
    assert!(
        Path::new(&logo.original_path).starts_with(Path::new(PROJECT_PATH).join("brand-kit")),
        "the logo is copied into the project: {}",
        logo.original_path
    );

    let text = bed
        .services
        .projects
        .add_graphic_item(&AddGraphicRequest {
            project_id: bed.project_id,
            sequence_id: bed.sequence_id,
            at: secs(20.0),
            duration: secs(2.0),
            graphic: NewGraphic::Text {
                content: "Titre".to_owned(),
            },
            track_id: None,
        })
        .await
        .unwrap();
    let instance = bed
        .services
        .projects
        .add_graphic_item(&AddGraphicRequest {
            project_id: bed.project_id,
            sequence_id: bed.sequence_id,
            at: secs(30.0),
            duration: secs(2.0),
            graphic: NewGraphic::Template {
                template_id: example_ids::TEMPLATE,
            },
            track_id: None,
        })
        .await
        .unwrap();
    let project = bed.project().await;
    let kind = |id| &project.items().find(|item| item.id == id).unwrap().kind;
    let ItemKind::Text(text) = kind(text.item_id) else {
        panic!("a text");
    };
    assert_eq!(text.style, kit.text_style);
    let ItemKind::MotionTemplate(instance) = kind(instance.item_id) else {
        panic!("an instance");
    };
    assert_eq!(
        instance.params["accent"],
        TemplateParamValue::Color {
            value: se_domain::Animatable::constant(kit.colors.accent)
        }
    );

    let removal = bed
        .services
        .media
        .remove_asset(bed.project_id, copy.logos[0].asset_id)
        .await
        .unwrap_err();
    assert_eq!(removal.code(), ErrorCode::EditRejected);
    assert!(removal.to_string().contains("brand kit"), "{removal}");
}

#[tokio::test(start_paused = true)]
async fn saving_a_kit_updates_open_projects_in_one_undoable_step() {
    let bed = bed().await;
    let kit = bed.kit_with_logo().await;
    bed.services
        .brand_kits
        .set_project_kit(bed.project_id, Some(kit.id))
        .await
        .unwrap();
    let mut changed = kit.clone();
    changed.colors.accent = Color::opaque(0x00, 0xff, 0x88);
    bed.services.events.take();
    let saved = bed.services.brand_kits.save(&changed).await.unwrap();
    assert_eq!(saved.version, kit.version + 1);
    let copy = bed.project().await.brand_kit.clone().unwrap();
    assert_eq!(copy.colors.accent, changed.colors.accent);
    assert_eq!(copy.source.unwrap().version, saved.version);
    assert_eq!(
        bed.undo_label().await.as_deref(),
        Some(labels::UPDATE_BRAND_KIT)
    );
    assert!(
        bed.services
            .events
            .take()
            .contains(&CoreEvent::LibraryChanged {
                templates: false,
                brand_kits: true
            })
    );
    bed.services.projects.undo(bed.project_id).await.unwrap();
    assert_eq!(
        bed.project().await.brand_kit.clone().unwrap().colors.accent,
        kit.colors.accent
    );

    bed.services
        .brand_kits
        .set_project_kit(bed.project_id, None)
        .await
        .unwrap();
    assert_eq!(bed.project().await.brand_kit, None);
}

#[tokio::test(start_paused = true)]
async fn removed_kits_stay_in_projects_which_can_add_them_back() {
    let bed = bed().await;
    let kit = bed.kit_with_logo().await;
    bed.services
        .brand_kits
        .set_project_kit(bed.project_id, Some(kit.id))
        .await
        .unwrap();
    bed.services.brand_kits.remove(kit.id).await.unwrap();
    assert_eq!(bed.services.brand_kit_library.removed(), [kit.id]);
    assert!(bed.project().await.brand_kit.is_some());
    let missing = bed.services.brand_kits.kit(kit.id).await.unwrap_err();
    assert_eq!(missing.code(), ErrorCode::BrandKitNotFound);

    let added = bed
        .services
        .brand_kits
        .add_project_kit_to_library(bed.project_id)
        .await
        .unwrap();
    assert_ne!(added.id, kit.id);
    assert_eq!(added.name, "Acme");
    assert_eq!(added.logos.len(), 1);
    assert_eq!(
        bed.project()
            .await
            .brand_kit
            .clone()
            .unwrap()
            .source
            .unwrap()
            .kit_id,
        added.id
    );
    assert_eq!(bed.services.brand_kits.list().await.unwrap().len(), 1);
}
