//! The template library (prompt 25, ADR-0036) over the example project: saving templates and
//! caption styles, versions, insertion into another project (copies reused), updates proposed
//! and applied on request, `.setemplate` export and import (checked by the renderer).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_app::{
    AgentContext,
    agent_edits::AgentEdit,
    test_support::{MediaTestbed, probed_video, settle},
};
use se_domain::{
    ItemKind, LibraryLink, Project, ProjectId, SequenceId, TemplateCategory, parse_definition_json,
    test_support::{LIBRARY_LOWER_THIRD, example_ids, example_project},
};
use se_protocol::{
    CoreEvent, ErrorCode, InsertLibraryTemplateRequest, LibraryContent, LibraryImportOutcome,
    LibraryTemplate, LibraryTemplateInfo, SubmitTaskRequest, TaskScope, TemplateCheckOutcome,
};

const PROJECT_PATH: &str = "/films/Example.seproj";

fn secs(seconds: f64) -> se_domain::Flicks {
    se_domain::Flicks::from_seconds_f64(seconds).unwrap()
}

fn info(name: &str) -> LibraryTemplateInfo {
    LibraryTemplateInfo {
        name: name.to_owned(),
        description: "Nom et fonction".to_owned(),
        tags: vec!["Interview".to_owned()],
        category: Some(TemplateCategory::LowerThird),
    }
}

struct Bed {
    services: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
}

async fn bed() -> Bed {
    let services = MediaTestbed::new(Path::new("/films"));
    services.tasks.start().await.unwrap();
    let project = example_project();
    for (content, asset) in (1..).zip(&project.assets) {
        services
            .files
            .add(asset.original_path.clone(), content, probed_video());
    }
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
    async fn project(&self, id: ProjectId) -> Arc<Project> {
        self.services.projects.project(id).await.unwrap()
    }

    /// A second, empty project (no brand kit), with its sequence.
    async fn other_project(&self) -> (ProjectId, SequenceId) {
        let id = self
            .services
            .create_project(Path::new("/films"), "Autre")
            .await
            .unwrap();
        (id, self.project(id).await.sequences[0].id)
    }

    async fn save(&self) -> LibraryTemplate {
        self.services
            .library_templates
            .save_template(self.project_id, example_ids::TEMPLATE, &info("Bandeau"))
            .await
            .unwrap()
    }

    async fn insert(
        &self,
        project: ProjectId,
        sequence: SequenceId,
        entry: &LibraryTemplate,
        at: f64,
    ) -> se_protocol::InsertedLibraryTemplate {
        self.services
            .library_templates
            .insert(&InsertLibraryTemplateRequest {
                project_id: project,
                template_id: entry.id,
                sequence_id: sequence,
                track_id: None,
                start: secs(at),
            })
            .await
            .unwrap()
    }

    /// A new version of the example's template written by an agent (its `slide` narrowed).
    async fn agent_version(&self) {
        let scope = TaskScope {
            sequence_id: self.sequence_id,
            range: None,
            tracks: None,
        };
        let task = self
            .services
            .tasks
            .submit(SubmitTaskRequest {
                project_id: self.project_id,
                prompt: "Change".to_owned(),
                scope: scope.clone(),
                provider: None,
                model: None,
                effort: None,
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let agent = AgentContext {
            project_id: self.project_id,
            task_id: task.id,
            run: 0,
            scope,
        };
        let definition = parse_definition_json(
            r#"{"name":"Lower third","defaultDuration":5,"category":"lowerThird"}"#,
            r#"[["name",{"type":"string","default":"Jane"}],
                ["slide",{"type":"number","default":50,"minimum":0,"maximum":100}]]"#,
        )
        .unwrap();
        self.services
            .edits
            .apply(
                &agent,
                "test",
                &[AgentEdit::UpdateTemplate {
                    template: example_ids::TEMPLATE,
                    definition: Box::new(definition),
                    source: "export function render() { /* v2 */ }".to_owned(),
                }],
            )
            .await
            .unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn saving_creates_an_entry_and_links_the_project_copy() {
    let bed = bed().await;
    bed.services.events.take();
    let entry = bed.save().await;
    assert_ne!(
        entry.id, LIBRARY_LOWER_THIRD,
        "the old link pointed nowhere: a new entry"
    );
    assert_eq!((entry.version, entry.name.as_str()), (1, "Bandeau"));
    assert_eq!(entry.tags, ["interview"]);
    let LibraryContent::MotionTemplate { category, .. } = &entry.content else {
        panic!("a motion template");
    };
    assert_eq!(*category, TemplateCategory::LowerThird);
    let template = bed.project(bed.project_id).await.templates[0].clone();
    assert_eq!(
        template.library,
        Some(LibraryLink {
            template_id: entry.id,
            version: 1
        })
    );
    assert_eq!(template.name, "Bandeau");
    assert!(
        bed.services
            .events
            .take()
            .contains(&CoreEvent::LibraryChanged {
                templates: true,
                brand_kits: false
            })
    );

    let again = bed.save().await;
    assert_eq!((again.id, again.version), (entry.id, 1), "nothing changed");
    bed.agent_version().await;
    let newer = bed.save().await;
    assert_eq!((newer.id, newer.version), (entry.id, 2));
    assert_eq!(
        bed.project(bed.project_id).await.templates[0].library,
        Some(LibraryLink {
            template_id: entry.id,
            version: 2
        })
    );
}

#[tokio::test(start_paused = true)]
async fn library_templates_go_into_other_projects_reusing_copies() {
    let bed = bed().await;
    let entry = bed.save().await;
    let (other, sequence) = bed.other_project().await;
    let first = bed.insert(other, sequence, &entry, 1.0).await;
    let second = bed.insert(other, sequence, &entry, 8.0).await;
    assert_eq!(first.template_id, second.template_id, "one copy");
    let project = bed.project(other).await;
    assert_eq!(project.templates.len(), 1);
    let copy = &project.templates[0];
    assert_eq!(copy.library.unwrap().template_id, entry.id);
    assert_eq!(copy.version, 1);
    let item = project
        .items()
        .find(|item| item.id == first.item_id)
        .unwrap();
    assert_eq!(item.timeline_range.start(), secs(1.0));
    assert_eq!(item.timeline_range.duration(), copy.default_duration);
    let ItemKind::MotionTemplate(instance) = &item.kind else {
        panic!("an instance");
    };
    assert!(instance.params.is_empty(), "no brand kit: schema defaults");

    // The first insertion was one step: undoing it removes the instance and the copy.
    bed.services.projects.undo(other).await.unwrap();
    bed.services.projects.undo(other).await.unwrap();
    let project = bed.project(other).await;
    assert_eq!(project.templates.len(), 0);
    assert_eq!(project.items().count(), 0);
}

#[tokio::test(start_paused = true)]
async fn newer_versions_are_proposed_then_applied_on_request() {
    let bed = bed().await;
    let entry = bed.save().await;
    let (other, sequence) = bed.other_project().await;
    let inserted = bed.insert(other, sequence, &entry, 0.0).await;
    bed.agent_version().await;
    let newer = bed.save().await;
    let copy = bed.project(other).await.templates[0].clone();
    assert!(
        copy.library.unwrap().is_outdated_by(newer.version),
        "the other project still holds version 1: an update is proposed"
    );
    let report = bed
        .services
        .library_templates
        .update_project_template(other, inserted.template_id)
        .await
        .unwrap();
    assert_eq!((report.version, report.library_version), (2, 2));
    let updated = bed.project(other).await.templates[0].clone();
    assert_eq!(updated.source, "export function render() { /* v2 */ }");
    assert_eq!(updated.library.unwrap().version, 2);
    let current = bed
        .services
        .library_templates
        .update_project_template(other, inserted.template_id)
        .await
        .unwrap_err();
    assert_eq!(current.code(), ErrorCode::EditRejected);
    bed.services.projects.undo(other).await.unwrap();
    assert_eq!(bed.project(other).await.templates[0].version, 1);
}

#[tokio::test(start_paused = true)]
async fn exported_files_import_back_after_a_check() {
    let bed = bed().await;
    let entry = bed.save().await;
    let path = PathBuf::from("/exports/bandeau.setemplate");
    bed.services
        .library_templates
        .export(entry.id, &path)
        .await
        .unwrap();
    assert_eq!(
        bed.services.template_library.archive(&path),
        Some(entry.clone())
    );
    bed.services
        .library_templates
        .remove(entry.id)
        .await
        .unwrap();
    assert_eq!(bed.services.template_library.removed(), [entry.id]);

    let imported = import_answering(&bed, &[path.clone(), PathBuf::from("/nope.setemplate")]).await;
    assert!(
        matches!(&imported[0], LibraryImportOutcome::Imported { template } if template.id == entry.id)
    );
    assert!(
        matches!(&imported[1], LibraryImportOutcome::Failed { message } if message.contains("not a SimpleEffect template"))
    );
    let again = import_answering(&bed, &[path]).await;
    assert!(matches!(&again[0], LibraryImportOutcome::Unchanged { .. }));
    let listed = bed.services.library_templates.list().await.unwrap();
    assert_eq!(listed.len(), 1);
    let LibraryContent::MotionTemplate { params, .. } = &listed[0].content else {
        panic!("a motion template");
    };
    assert_eq!(
        params.len(),
        1,
        "the module's checked declaration replaced the archive's"
    );
}

/// Imports `paths`, answering each template check with a passed check declaring one parameter.
async fn import_answering(bed: &Bed, paths: &[PathBuf]) -> Vec<LibraryImportOutcome> {
    let paths: Vec<String> = paths
        .iter()
        .map(|path| path.display().to_string())
        .collect();
    let library = bed.services.library_templates.clone();
    let project = bed.project_id;
    let waiting = tokio::spawn(async move { library.import(project, &paths).await });
    let mut answered = bed.services.renderer.template_checks().len();
    while !waiting.is_finished() {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let checks = bed.services.renderer.template_checks();
        for request in &checks[answered..] {
            bed.services
                .template_checks
                .deliver(
                    &request.request_id,
                    TemplateCheckOutcome::Passed {
                        meta_json: r#"{"name":"Lower third","defaultDuration":5}"#.to_owned(),
                        params_json: r#"[["name",{"type":"string","default":"Jane"}]]"#.to_owned(),
                        frames: Vec::new(),
                        slowest_frame_ms: 1.0,
                    },
                )
                .unwrap();
        }
        answered = checks.len();
    }
    waiting
        .await
        .unwrap()
        .unwrap()
        .into_iter()
        .map(|result| result.outcome)
        .collect()
}

#[tokio::test(start_paused = true)]
async fn caption_styles_are_saved_and_entries_renamed() {
    let bed = bed().await;
    let entry = bed
        .services
        .library_templates
        .save_caption_style(bed.project_id, example_ids::CAPTION_STYLE, &info("Sobre"))
        .await
        .unwrap();
    assert!(matches!(entry.content, LibraryContent::CaptionStyle { .. }));
    let renamed = bed
        .services
        .library_templates
        .update_info(entry.id, &info("Sobre blanc"))
        .await
        .unwrap();
    assert_eq!((renamed.name.as_str(), renamed.version), ("Sobre blanc", 1));
    let refused = bed
        .services
        .library_templates
        .update_info(entry.id, &info("  "))
        .await
        .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::InvalidBrandKit);
    let (other, sequence) = bed.other_project().await;
    let not_motion = bed
        .services
        .library_templates
        .insert(&InsertLibraryTemplateRequest {
            project_id: other,
            template_id: entry.id,
            sequence_id: sequence,
            track_id: None,
            start: secs(0.0),
        })
        .await
        .unwrap_err();
    assert!(
        not_motion.to_string().contains("caption style"),
        "{not_motion}"
    );
}
