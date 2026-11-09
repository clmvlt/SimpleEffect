//! The app's library folders (prompt 25) in temporary folders: template entries and their
//! `.setemplate` archives, brand kits and their logo copies.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{fs, io::Write};

use se_app::ports::{BrandKitLibrary, StorageError, TemplateLibrary};
use se_domain::{
    BrandFonts, BrandKitId, BrandPalette, CaptionStyleId, Color, LibraryTemplateId,
    TemplateCategory, TextStyle, caption_style_from_preset, test_support::example_project,
};
use se_protocol::{LibraryBrandKit, LibraryContent, LibraryLogo, LibraryTemplate};
use se_store::{FsBrandKitLibrary, FsTemplateLibrary};

fn motion_entry() -> LibraryTemplate {
    let template = example_project().templates[0].clone();
    LibraryTemplate {
        id: LibraryTemplateId::from_test_index(1),
        version: 3,
        name: "Bandeau".to_owned(),
        description: "Nom et fonction".to_owned(),
        tags: vec!["interview".to_owned()],
        updated_at: 1_700_000_000_000,
        content: LibraryContent::MotionTemplate {
            category: TemplateCategory::LowerThird,
            source: template.source,
            params: template.params,
            default_duration: template.default_duration,
        },
    }
}

fn style_entry() -> LibraryTemplate {
    LibraryTemplate {
        id: LibraryTemplateId::from_test_index(2),
        version: 1,
        name: "Sobre".to_owned(),
        description: String::new(),
        tags: Vec::new(),
        updated_at: 0,
        content: LibraryContent::CaptionStyle {
            style: caption_style_from_preset("box", CaptionStyleId::from_test_index(1)).unwrap(),
        },
    }
}

#[tokio::test]
async fn entries_are_folders_with_their_module_apart() {
    let root = tempfile::tempdir().unwrap();
    let library = FsTemplateLibrary::new(root.path().join("templates"));
    assert!(library.list().await.unwrap().is_empty(), "no folder yet");
    let entry = motion_entry();
    library.save(&entry).await.unwrap();
    library.save(&style_entry()).await.unwrap();
    let folder = root.path().join("templates").join(entry.id.to_string());
    let source = fs::read_to_string(folder.join("source.ts")).unwrap();
    assert!(source.starts_with("export const meta"));
    let json = fs::read_to_string(folder.join("template.json")).unwrap();
    assert!(json.contains("\"formatVersion\": 1"));
    assert!(
        !json.contains("export const meta"),
        "the module lives in source.ts"
    );
    assert_eq!(library.get(entry.id).await.unwrap(), Some(entry.clone()));
    let mut listed = library.list().await.unwrap();
    listed.sort_by_key(|entry| entry.id);
    assert_eq!(listed, [entry.clone(), style_entry()]);

    // A folder written by a newer SimpleEffect is skipped when listing, refused alone.
    fs::write(
        folder.join("template.json"),
        json.replace("\"formatVersion\": 1", "\"formatVersion\": 9"),
    )
    .unwrap();
    assert_eq!(library.list().await.unwrap(), [style_entry()]);
    assert!(matches!(
        library.get(entry.id).await,
        Err(StorageError::SchemaTooNew { found: 9, .. })
    ));
}

#[tokio::test]
async fn archives_round_trip_and_refuse_other_files() {
    let root = tempfile::tempdir().unwrap();
    let library = FsTemplateLibrary::new(root.path().join("templates"));
    for entry in [motion_entry(), style_entry()] {
        let path = root.path().join(format!("{}.setemplate", entry.name));
        library.write_archive(&entry, &path).await.unwrap();
        assert_eq!(library.read_archive(&path).await.unwrap(), entry);
    }
    let text = root.path().join("notes.setemplate");
    fs::write(&text, "hello").unwrap();
    let error = library.read_archive(&text).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("not a SimpleEffect template archive"),
        "{error}"
    );
    let empty_zip = root.path().join("empty.setemplate");
    let mut writer = zip::ZipWriter::new(fs::File::create(&empty_zip).unwrap());
    writer
        .start_file("readme.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"nothing").unwrap();
    writer.finish().unwrap();
    let error = library.read_archive(&empty_zip).await.unwrap_err();
    assert!(
        error.to_string().contains("template.json is missing"),
        "{error}"
    );
}

fn kit(logos: Vec<LibraryLogo>) -> LibraryBrandKit {
    LibraryBrandKit {
        id: BrandKitId::from_test_index(1),
        version: 1,
        updated_at: 0,
        name: "Acme".to_owned(),
        fonts: BrandFonts {
            heading: "Montserrat".to_owned(),
            body: "Inter".to_owned(),
        },
        colors: BrandPalette {
            primary: Color::opaque(1, 2, 3),
            secondary: Color::BLACK,
            accent: Color::WHITE,
            text: Color::WHITE,
            background: Color::BLACK,
        },
        logos,
        text_style: TextStyle::default(),
        caption_style: caption_style_from_preset("minimal", CaptionStyleId::from_test_index(1))
            .unwrap(),
        intro_templates: Vec::new(),
        outro_templates: Vec::new(),
    }
}

#[tokio::test]
async fn kits_keep_the_logos_they_list_and_copy_them_into_projects() {
    let root = tempfile::tempdir().unwrap();
    let library = FsBrandKitLibrary::new(root.path().join("brand-kits"));
    let original = root.path().join("acme.png");
    fs::write(&original, b"PNG bytes").unwrap();
    let id = BrandKitId::from_test_index(1);
    library
        .import_logo(id, &original, "logos/a.png")
        .await
        .unwrap();
    library
        .import_logo(id, &original, "logos/b.png")
        .await
        .unwrap();
    let logo = |file: &str| LibraryLogo {
        name: "Logo".to_owned(),
        file: file.to_owned(),
    };
    library.save(&kit(vec![logo("logos/a.png")])).await.unwrap();
    assert!(library.logo_path(id, "logos/a.png").exists());
    assert!(
        !library.logo_path(id, "logos/b.png").exists(),
        "unlisted copies go"
    );
    assert!(original.exists(), "the user's file stays");
    assert_eq!(
        library.get(id).await.unwrap(),
        Some(kit(vec![logo("logos/a.png")]))
    );
    assert_eq!(library.list().await.unwrap().len(), 1);

    let destination = root
        .path()
        .join("Film.seproj")
        .join("brand-kit")
        .join("a.png");
    library
        .copy_logo(id, "logos/a.png", &destination)
        .await
        .unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"PNG bytes");
    library
        .copy_logo(id, "logos/a.png", &destination)
        .await
        .unwrap();

    for bad in ["../brand-kit.json", "logos/../../x.png"] {
        assert!(
            library.import_logo(id, &original, bad).await.is_err(),
            "{bad}"
        );
        assert!(
            library.copy_logo(id, bad, &destination).await.is_err(),
            "{bad}"
        );
    }
    assert!(
        library
            .get(BrandKitId::from_test_index(2))
            .await
            .unwrap()
            .is_none()
    );
}
