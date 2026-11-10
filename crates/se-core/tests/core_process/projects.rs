//! Projects end to end: creation, events, crash recovery from the journal, the lock between two
//! cores, and the snapshot saved at shutdown.

use std::path::Path;

use serde_json::{Value, json};

use crate::running_core::{RpcSocket, RunningCore, call};

fn hd_format() -> Value {
    json!({
        "width": 1920,
        "height": 1080,
        "frameRate": { "numerator": 25, "denominator": 1 },
        "sampleRate": 48_000
    })
}

/// Creates `name` in `parent` and returns its session result.
async fn create(socket: &mut RpcSocket, parent: &Path, name: &str) -> Value {
    let (reply, _) = call(
        socket,
        1,
        json!({
            "kind": "createProject",
            "name": name,
            "parentDir": parent.to_string_lossy(),
            "format": hd_format()
        }),
    )
    .await;
    assert_eq!(reply["result"]["kind"], "createProject", "{reply}");
    reply["result"].clone()
}

async fn create_sequence(socket: &mut RpcSocket, project_id: &Value) -> Vec<Value> {
    let (reply, events) = call(
        socket,
        2,
        json!({ "kind": "createSequence", "projectId": project_id, "name": "Short", "format": null }),
    )
    .await;
    assert_eq!(reply["result"]["kind"], "createSequence", "{reply}");
    events
}

#[tokio::test]
async fn a_created_project_is_open_listed_and_patched_by_commits() {
    let core = RunningCore::start().await;
    let projects = tempfile::tempdir().expect("projects folder");
    let mut socket = core.connect().await;
    let session = create(&mut socket, projects.path(), "Film").await;
    assert!(session["path"].as_str().unwrap().ends_with("Film.seproj"));
    let snapshot = &session["snapshot"];
    assert_eq!(snapshot["revision"], 0);
    assert_eq!(snapshot["tracks"].as_array().unwrap().len(), 4);
    let project_id = snapshot["project"]["id"].clone();

    let (recent, _) = call(&mut socket, 3, json!({ "kind": "listRecentProjects" })).await;
    let listed = &recent["result"]["projects"][0];
    assert_eq!(listed["name"], "Film");
    assert_eq!(listed["available"], true);

    let events = create_sequence(&mut socket, &project_id).await;
    let kinds: Vec<&str> = events
        .iter()
        .map(|event| event["event"]["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["projectPatched", "historyChanged"]);
    assert_eq!(events[0]["event"]["revision"], 1);
    assert_eq!(events[1]["event"]["history"]["canUndo"], true);

    let (undo, events) = call(
        &mut socket,
        4,
        json!({ "kind": "undo", "projectId": project_id }),
    )
    .await;
    assert_eq!(undo["result"]["kind"], "undo");
    assert_eq!(events[0]["event"]["revision"], 2);
    let (nothing, _) = call(
        &mut socket,
        5,
        json!({ "kind": "undo", "projectId": project_id }),
    )
    .await;
    assert_eq!(nothing["error"]["code"], "NOTHING_TO_UNDO");
}

#[tokio::test]
async fn a_commit_survives_killing_the_core() {
    let data = tempfile::tempdir().expect("data folder");
    let projects = tempfile::tempdir().expect("projects folder");
    let mut core = RunningCore::start_in(data.path()).await;
    let mut socket = core.connect().await;
    let session = create(&mut socket, projects.path(), "Crash").await;
    let path = session["path"].clone();
    create_sequence(&mut socket, &session["snapshot"]["project"]["id"]).await;
    // Killed right after the commit, long before the 2 s snapshot.
    core.child.kill().await.expect("kill se-core");

    let core = RunningCore::start_in(data.path()).await;
    let mut socket = core.connect().await;
    let (reply, _) = call(
        &mut socket,
        1,
        json!({ "kind": "openProject", "path": path }),
    )
    .await;
    let snapshot = &reply["result"]["snapshot"];
    assert_eq!(snapshot["revision"], 1, "{reply}");
    assert_eq!(snapshot["sequences"].as_array().unwrap().len(), 2);
    let recovery = &reply["result"]["recovery"];
    assert_eq!(recovery["replayedChanges"], 1, "{reply}");
    assert_eq!(recovery["restoredFrom"], Value::Null);
}

#[tokio::test]
async fn a_corrupted_project_json_opens_from_its_backup_and_the_journal() {
    let data = tempfile::tempdir().expect("data folder");
    let projects = tempfile::tempdir().expect("projects folder");
    let mut core = RunningCore::start_in(data.path()).await;
    let mut socket = core.connect().await;
    let session = create(&mut socket, projects.path(), "Damaged").await;
    let path = session["path"].clone();
    let project_id = session["snapshot"]["project"]["id"].clone();
    create_sequence(&mut socket, &project_id).await;
    // Saved (`project.json` at revision 1, `.bak.1` at 0), then one more commit, then a kill.
    let (closed, _) = call(
        &mut socket,
        5,
        json!({ "kind": "closeProject", "projectId": project_id }),
    )
    .await;
    assert_eq!(closed["result"]["kind"], "closeProject", "{closed}");
    let (opened, _) = call(
        &mut socket,
        6,
        json!({ "kind": "openProject", "path": path }),
    )
    .await;
    assert_eq!(opened["result"]["recovery"], Value::Null, "{opened}");
    create_sequence(&mut socket, &project_id).await;
    core.child.kill().await.expect("kill se-core");
    let folder = Path::new(path.as_str().unwrap());
    std::fs::write(folder.join("project.json"), "{\"id\": ").expect("damage project.json");

    let core = RunningCore::start_in(data.path()).await;
    let mut socket = core.connect().await;
    let (reply, _) = call(
        &mut socket,
        1,
        json!({ "kind": "openProject", "path": path }),
    )
    .await;
    let snapshot = &reply["result"]["snapshot"];
    assert_eq!(snapshot["revision"], 2, "nothing lost: {reply}");
    assert_eq!(snapshot["sequences"].as_array().unwrap().len(), 3);
    let recovery = &reply["result"]["recovery"];
    assert_eq!(recovery["restoredFrom"], "project.json.bak.1", "{reply}");
    assert_eq!(recovery["lostChanges"], 0);
    assert!(folder.join("project.json.corrupted").is_file());
}

#[tokio::test]
async fn a_project_open_in_another_core_is_refused_until_closed() {
    let first = RunningCore::start().await;
    let second = RunningCore::start().await;
    let projects = tempfile::tempdir().expect("projects folder");
    let mut first_socket = first.connect().await;
    let session = create(&mut first_socket, projects.path(), "Shared").await;
    let path = session["path"].clone();

    let mut second_socket = second.connect().await;
    let (refused, _) = call(
        &mut second_socket,
        1,
        json!({ "kind": "openProject", "path": path }),
    )
    .await;
    assert_eq!(refused["error"]["code"], "PROJECT_LOCKED", "{refused}");
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("already open")
    );

    let project_id = session["snapshot"]["project"]["id"].clone();
    let (closed, events) = call(
        &mut first_socket,
        2,
        json!({ "kind": "closeProject", "projectId": project_id }),
    )
    .await;
    assert_eq!(closed["result"]["kind"], "closeProject");
    assert_eq!(events[0]["event"]["kind"], "projectClosed");
    let (opened, _) = call(
        &mut second_socket,
        3,
        json!({ "kind": "openProject", "path": path }),
    )
    .await;
    assert_eq!(opened["result"]["kind"], "openProject", "{opened}");
}

#[tokio::test]
async fn stopping_the_core_saves_open_projects() {
    let mut core = RunningCore::start().await;
    let projects = tempfile::tempdir().expect("projects folder");
    let mut socket = core.connect().await;
    let session = create(&mut socket, projects.path(), "Saved").await;
    create_sequence(&mut socket, &session["snapshot"]["project"]["id"]).await;
    assert!(core.stop().await.success());

    let folder = Path::new(session["path"].as_str().unwrap());
    let saved: Value = serde_json::from_str(
        &std::fs::read_to_string(folder.join("project.json")).expect("project.json"),
    )
    .expect("valid JSON");
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["sequences"].as_array().unwrap().len(), 2);
}
