//! Diagnostics files (prompt 30): the newest log files cut to their end, and zip archives that
//! read back with their entries.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{fs, io::Read, time::Duration};

use se_app::ports::{ArchiveEntry, DiagnosticsFiles};
use se_store::FsDiagnosticsFiles;

#[tokio::test]
async fn the_newest_logs_are_read_cut_to_their_end() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    fs::create_dir(&logs).unwrap();
    for (index, name) in ["se-core.log.1", "se-core.log.2", "se-core.log.3"]
        .iter()
        .enumerate()
    {
        fs::write(
            logs.join(name),
            format!("start {index}\n{}", "x".repeat(100)),
        )
        .unwrap();
        // Distinct modification times, oldest first.
        std::thread::sleep(Duration::from_millis(30));
    }

    let read = FsDiagnosticsFiles.recent_logs(&logs, 2, 10).await.unwrap();

    let names: Vec<_> = read.iter().map(|log| log.name.as_str()).collect();
    assert_eq!(names, ["se-core.log.3", "se-core.log.2"]);
    assert_eq!(read[0].text, "x".repeat(10));
    let missing = FsDiagnosticsFiles
        .recent_logs(&dir.path().join("nothing"), 7, 10)
        .await
        .unwrap();
    assert!(missing.is_empty());
}

#[tokio::test]
async fn an_archive_reads_back_with_its_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out").join("diagnostic.zip");
    let entries = vec![
        ArchiveEntry {
            name: "report.json".to_owned(),
            bytes: b"{}".to_vec(),
        },
        ArchiveEntry {
            name: "logs/se-core.log".to_owned(),
            bytes: b"INFO ready".to_vec(),
        },
    ];

    let size = FsDiagnosticsFiles
        .write_archive(&path, entries)
        .await
        .unwrap();

    assert_eq!(size, fs::metadata(&path).unwrap().len());
    let mut archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("logs/se-core.log")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    assert_eq!(text, "INFO ready");
    assert_eq!(archive.len(), 2);
}
