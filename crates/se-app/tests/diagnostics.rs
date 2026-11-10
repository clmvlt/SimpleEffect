//! Diagnostics archives (prompt 30): the report and logs of the testbed, written with the home
//! folder, file names, tokens and e-mail addresses masked and no project content.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{path::Path, sync::Arc};

use se_app::{
    AppError, DiagnosticsReport, DiagnosticsService,
    test_support::{FakeDiagnosticsFiles, MediaTestbed},
};
use se_protocol::{ClientDiagnostics, CodecSupport, ComponentVersion, CoreInfo};

const HOME: &str = "C:\\Users\\Marie Dupont";
const TOKEN: &str = "3f9a1c0b7d2e4f6a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a";

async fn report(services: &MediaTestbed) -> DiagnosticsReport {
    DiagnosticsReport {
        core: CoreInfo {
            version: "0.1.0".to_owned(),
            os: "windows".to_owned(),
            arch: "x86_64".to_owned(),
            app_data_dir: format!("{HOME}\\AppData\\Roaming\\SimpleEffect"),
            logs_dir: format!("{HOME}\\AppData\\Roaming\\SimpleEffect\\logs"),
        },
        ai_status: services.tasks.ai_status().await,
        ai_settings: services.tasks.ai_settings(),
        media_tools: services.media.tools_status().await,
        media_settings: services.media.media_settings().await.unwrap(),
        transcription: services.transcription.status().await.unwrap(),
        vision: services.reframing.status().await,
        client: ClientDiagnostics {
            app_version: "0.1.0".to_owned(),
            components: vec![ComponentVersion {
                name: "Electron".to_owned(),
                version: "45.2.3".to_owned(),
            }],
            gpu: Some("ANGLE (NVIDIA GeForce RTX 3080)".to_owned()),
            codecs: vec![CodecSupport {
                codec: "H.264".to_owned(),
                decode: true,
                encode: true,
            }],
            locale: "fr".to_owned(),
        },
    }
}

fn text_of(entries: &[se_app::ports::ArchiveEntry], name: &str) -> String {
    let entry = entries
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("no {name} in the archive"));
    String::from_utf8(entry.bytes.clone()).unwrap()
}

#[tokio::test]
async fn the_archive_holds_the_report_and_logs_with_personal_data_masked() {
    let services = MediaTestbed::new(Path::new("/films"));
    let files = FakeDiagnosticsFiles::default();
    files.put_log(
        "se-core.log.2026-11-10",
        &format!(
            "INFO opened project {HOME}\\Videos\\Mariage Julie.seproj\n\
             INFO importing \"D:\\Rushes\\Julie et Marc.MP4\"\n\
             DEBUG ws://127.0.0.1:52000/rpc?t={TOKEN}\n\
             INFO signed in as marie.dupont@example.fr\n"
        ),
    );
    files.put_log("se-core.log.2026-11-09", "INFO core ready\n");
    let service = DiagnosticsService::new(
        Arc::new(files.clone()),
        services.clock.clone(),
        "C:\\logs".into(),
        Some(Path::new(HOME)),
    );
    let path = Path::new("C:\\Exports\\diagnostic.zip");

    let summary = service
        .export(path, &report(&services).await)
        .await
        .unwrap();

    assert_eq!(
        summary.files,
        [
            "README.txt",
            "report.json",
            "logs/se-core.log.2026-11-10",
            "logs/se-core.log.2026-11-09"
        ]
    );
    let entries = files.archive(path).unwrap();
    assert_eq!(
        summary.bytes,
        entries
            .iter()
            .map(|entry| entry.bytes.len() as u64)
            .sum::<u64>()
    );
    let everything: String = entries
        .iter()
        .map(|entry| String::from_utf8(entry.bytes.clone()).unwrap())
        .collect();
    for secret in [
        "Marie Dupont",
        "Julie",
        "Marc",
        TOKEN,
        "marie.dupont@example.fr",
    ] {
        assert!(
            !everything.contains(secret),
            "{secret} leaked:\n{everything}"
        );
    }
    let report = text_of(&entries, "report.json");
    let value: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(
        value["core"]["appDataDir"],
        "~\\AppData\\Roaming\\SimpleEffect"
    );
    assert_eq!(value["client"]["gpu"], "ANGLE (NVIDIA GeForce RTX 3080)");
    assert!(value["aiStatus"]["installation"].is_object());
    let log = text_of(&entries, "logs/se-core.log.2026-11-10");
    assert!(log.contains("~\\Videos\\"), "{log}");
    assert!(log.contains(".seproj") && log.contains(".MP4"), "{log}");
    assert!(log.contains("t=<redacted>"), "{log}");
}

#[tokio::test]
async fn an_archive_must_be_an_absolute_zip() {
    let services = MediaTestbed::new(Path::new("/films"));
    let service = DiagnosticsService::new(
        Arc::new(FakeDiagnosticsFiles::default()),
        services.clock.clone(),
        "/logs".into(),
        None,
    );
    let report = report(&services).await;
    for path in ["diagnostic.zip", "/tmp/diagnostic.txt"] {
        let error = service.export(Path::new(path), &report).await.unwrap_err();
        assert!(
            matches!(error, AppError::InvalidSetting { .. }),
            "{path}: {error}"
        );
    }
}
