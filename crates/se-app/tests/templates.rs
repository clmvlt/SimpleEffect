//! Motion templates (prompt 24, ADR-0035) over the example project: the renderer round trip of
//! template checks, templates and instances made by agents with their scope rule, new versions
//! in place or copied on write, reverts, and the user's commands.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_app::{
    AgentContext, AppError, EditFailure, TemplateCheckService,
    agent_edits::{AgentEdit, NewAgentItem},
    test_support::{FakeRendererLink, MediaTestbed, SequentialIds, probed_video, settle},
};
use se_domain::{
    Animatable, ItemId, ItemKind, Origin, Project, ProjectId, SequenceId, TemplateDefinition,
    TemplateId, TemplateItem, TemplateParamValue, TimeRange, Transform2D, parse_definition_json,
    test_support::{example_ids, example_project},
};
use se_protocol::{
    AddGraphicRequest, ErrorCode, NewGraphic, ProtocolError, SubmitTaskRequest, TaskScope,
    TemplateCheckFrame, TemplateCheckOutcome, TemplateCheckStage, TemplateProblem,
};

const PROJECT_PATH: &str = "/films/Example.seproj";
/// The example's lower-third instance: V3, 2–3 s.
const INSTANCE: u64 = 5;

fn secs(seconds: f64) -> se_domain::Flicks {
    se_domain::Flicks::from_seconds_f64(seconds).unwrap()
}

fn range(start: f64, end: f64) -> TimeRange {
    TimeRange::from_start_end(secs(start), secs(end)).unwrap()
}

fn code(failure: EditFailure) -> ErrorCode {
    ProtocolError::from(failure.error).code
}

/// A definition with a `name` text and a `size` number (0–100), as a renderer would report it.
fn definition(name: &str) -> TemplateDefinition {
    parse_definition_json(
        &format!(r#"{{"name":"{name}","defaultDuration":4}}"#),
        r#"[["name",{"type":"string","default":"Jane"}],
            ["size",{"type":"number","default":40,"minimum":0,"maximum":100}]]"#,
    )
    .unwrap()
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
    async fn task(&self, range: Option<TimeRange>) -> AgentContext {
        let scope = TaskScope {
            sequence_id: self.sequence_id,
            range,
            tracks: None,
        };
        let task = self
            .services
            .tasks
            .submit(SubmitTaskRequest {
                project_id: self.project_id,
                prompt: "Motion".to_owned(),
                scope: scope.clone(),
                provider: None,
                model: None,
                effort: None,
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        AgentContext {
            project_id: self.project_id,
            task_id: task.id,
            run: 0,
            scope,
        }
    }

    async fn project(&self) -> Arc<Project> {
        self.services
            .projects
            .project(self.project_id)
            .await
            .unwrap()
    }

    async fn apply(
        &self,
        agent: &AgentContext,
        edits: Vec<AgentEdit>,
    ) -> Result<se_app::agent_edits::AgentEditReport, EditFailure> {
        self.services.edits.apply(agent, "test", &edits).await
    }
}

fn instance_of(project: &Project, item: ItemId) -> TemplateItem {
    match &project
        .items()
        .find(|candidate| candidate.id == item)
        .unwrap()
        .kind
    {
        ItemKind::MotionTemplate(instance) => instance.clone(),
        other => panic!("{item} is a {}", other.label()),
    }
}

fn new_instance(template: TemplateId, start: f64, end: f64) -> AgentEdit {
    AgentEdit::AddItem(Box::new(NewAgentItem {
        kind: ItemKind::MotionTemplate(TemplateItem {
            template_id: template,
            params: [(
                "size".to_owned(),
                TemplateParamValue::Number {
                    value: Animatable::constant(80.0),
                },
            )]
            .into(),
            transform: Transform2D::default(),
        }),
        range: range(start, end),
        track: None,
        presets: Vec::new(),
    }))
}

#[tokio::test(start_paused = true)]
async fn agents_create_templates_and_instances_that_revert_with_their_task() {
    let bed = bed().await;
    let agent = bed.task(Some(range(4.0, 9.0))).await;
    let report = bed
        .apply(
            &agent,
            vec![AgentEdit::CreateTemplate {
                definition: Box::new(definition("Title")),
                source: "export function render() {}".to_owned(),
            }],
        )
        .await
        .unwrap();
    let change = &report.templates[0];
    assert_eq!((change.version, change.copied_from), (1, None));
    let project = bed.project().await;
    let template = project
        .templates
        .iter()
        .find(|t| t.id == change.template_id)
        .unwrap();
    assert_eq!(template.name, "Title");
    assert_eq!(
        template.origin,
        Origin::AiTask {
            task_id: agent.task_id
        }
    );

    let placed = bed
        .apply(&agent, vec![new_instance(change.template_id, 5.0, 6.0)])
        .await
        .unwrap();
    let item = placed.created[0][0];
    assert_eq!(
        instance_of(&*bed.project().await, item).params.len(),
        1,
        "the instance keeps its value"
    );
    let outside = bed
        .apply(&agent, vec![new_instance(change.template_id, 8.5, 9.5)])
        .await
        .unwrap_err();
    assert_eq!(code(outside), ErrorCode::OutOfScope);

    let report = bed
        .services
        .tasks
        .revert(bed.project_id, agent.task_id)
        .await
        .unwrap();
    assert!(report.reverted);
    let project = bed.project().await;
    assert_eq!(project.templates.len(), 1, "the task's template is gone");
    assert!(project.items().all(|candidate| candidate.id != item));
}

#[tokio::test(start_paused = true)]
async fn a_new_version_changes_instances_in_place_and_resets_refused_values() {
    let bed = bed().await;
    let agent = bed.task(None).await;
    let report = bed
        .apply(
            &agent,
            vec![AgentEdit::UpdateTemplate {
                template: example_ids::TEMPLATE,
                definition: Box::new(definition("Lower third v2")),
                source: "export function render() { /* v2 */ }".to_owned(),
            }],
        )
        .await
        .unwrap();
    let change = &report.templates[0];
    assert_eq!(change.template_id, example_ids::TEMPLATE);
    assert_eq!((change.version, change.copied_from), (2, None));
    let instance = ItemId::from_test_index(INSTANCE);
    assert_eq!(change.instances, [instance]);
    // The example sets `name` (still a string), `accent`, `slide` and `showLine` (gone).
    assert_eq!(
        change.reset,
        [(
            instance,
            vec![
                "accent".to_owned(),
                "showLine".to_owned(),
                "slide".to_owned()
            ]
        )]
    );
    let project = bed.project().await;
    assert_eq!(project.templates[0].version, 2);
    assert_eq!(
        instance_of(&project, instance)
            .params
            .keys()
            .collect::<Vec<_>>(),
        ["name"]
    );
}

#[tokio::test(start_paused = true)]
async fn a_new_version_is_copied_for_the_instances_of_the_scope() {
    let bed = bed().await;
    // The example's instance (2–3 s) lies outside a 4–9 s scope: an instance made in the scope
    // gets the new version, the old one keeps the old template.
    let agent = bed.task(Some(range(4.0, 9.0))).await;
    let placed = bed
        .apply(&agent, vec![new_instance(example_ids::TEMPLATE, 4.0, 5.0)])
        .await
        .unwrap_err();
    // `size` is not a parameter of the example's template: refused by validation.
    assert_eq!(code(placed), ErrorCode::EditRejected);
    let own = AgentEdit::AddItem(Box::new(NewAgentItem {
        kind: ItemKind::MotionTemplate(TemplateItem {
            template_id: example_ids::TEMPLATE,
            params: [(
                "name".to_owned(),
                TemplateParamValue::Text {
                    value: "Grace".to_owned(),
                },
            )]
            .into(),
            transform: Transform2D::default(),
        }),
        range: range(4.0, 5.0),
        track: None,
        presets: Vec::new(),
    }));
    let mine = bed.apply(&agent, vec![own]).await.unwrap().created[0][0];
    let report = bed
        .apply(
            &agent,
            vec![AgentEdit::UpdateTemplate {
                template: example_ids::TEMPLATE,
                definition: Box::new(definition("Lower third v2")),
                source: "export function render() {}".to_owned(),
            }],
        )
        .await
        .unwrap();
    let change = &report.templates[0];
    assert_eq!(change.copied_from, Some(example_ids::TEMPLATE));
    assert_ne!(change.template_id, example_ids::TEMPLATE);
    assert_eq!(change.instances, [mine]);
    assert_eq!(change.reset, []);
    let project = bed.project().await;
    assert_eq!(project.templates.len(), 2);
    assert_eq!(project.templates[0].version, 1, "the original is untouched");
    assert_eq!(instance_of(&project, mine).template_id, change.template_id);
    assert_eq!(
        instance_of(&project, ItemId::from_test_index(INSTANCE)).template_id,
        example_ids::TEMPLATE
    );
}

#[tokio::test(start_paused = true)]
async fn invalid_definitions_name_the_parameter() {
    let bed = bed().await;
    let agent = bed.task(None).await;
    let definition = parse_definition_json(
        r#"{"name":"Twice","defaultDuration":4}"#,
        r#"[["size",{"type":"number","default":1}],["size",{"type":"boolean","default":true}]]"#,
    )
    .unwrap();
    let failure = bed
        .apply(
            &agent,
            vec![AgentEdit::CreateTemplate {
                definition: Box::new(definition),
                source: "export function render() {}".to_owned(),
            }],
        )
        .await
        .unwrap_err();
    let AppError::TemplateInvalid { problems } = failure.error else {
        panic!("a template declaring a parameter twice is refused");
    };
    assert_eq!(problems[0].stage, TemplateCheckStage::Definition);
    assert!(problems[0].message.contains("params.size"), "{problems:?}");
}

fn checks() -> (Arc<FakeRendererLink>, Arc<TemplateCheckService>) {
    let link = Arc::new(FakeRendererLink::default());
    let service = Arc::new(TemplateCheckService::with_timeout(
        link.clone(),
        Arc::new(SequentialIds::default()),
        Duration::from_secs(5),
    ));
    (link, service)
}

#[tokio::test(start_paused = true)]
async fn checks_go_to_the_renderer_and_come_back_read() {
    let (link, service) = checks();
    let format = example_project().sequences[0].format;
    let waiting = {
        let service = service.clone();
        tokio::spawn(async move {
            service
                .check(ProjectId::from_test_index(1), &format, "source")
                .await
        })
    };
    tokio::task::yield_now().await;
    let request = link.template_checks().pop().unwrap();
    assert_eq!((request.width, request.height), (1920, 1080));
    assert_eq!(request.picture_width, 640);
    service
        .deliver(
            &request.request_id,
            TemplateCheckOutcome::Passed {
                meta_json: r#"{"name":"Title","defaultDuration":3}"#.to_owned(),
                params_json: "[]".to_owned(),
                frames: vec![TemplateCheckFrame {
                    time: secs(1.5),
                    jpeg_base64: "/9j/".to_owned(),
                }],
                slowest_frame_ms: 3.0,
            },
        )
        .unwrap();
    let checked = waiting.await.unwrap().unwrap();
    assert_eq!(checked.definition.default_duration, secs(3.0));
    assert_eq!(checked.frames.len(), 1);
    assert_eq!(service.pending_count(), 0);
    let late = service.deliver(
        &request.request_id,
        TemplateCheckOutcome::Failed { problems: vec![] },
    );
    assert!(matches!(late, Err(AppError::TemplateCheckNotFound { .. })));
}

#[tokio::test(start_paused = true)]
async fn failed_checks_and_missing_renderers_are_explained() {
    let (link, service) = checks();
    let format = example_project().sequences[0].format;
    let waiting = {
        let service = service.clone();
        tokio::spawn(async move {
            service
                .check(ProjectId::from_test_index(1), &format, "x")
                .await
        })
    };
    tokio::task::yield_now().await;
    let request = link.template_checks().pop().unwrap();
    let problem = TemplateProblem {
        stage: TemplateCheckStage::Render,
        message: "fetch is not available in templates".to_owned(),
        line: Some(4),
        column: Some(3),
        time: Some(secs(0.5)),
    };
    service
        .deliver(
            &request.request_id,
            TemplateCheckOutcome::Failed {
                problems: vec![problem],
            },
        )
        .unwrap();
    let error = waiting.await.unwrap().unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("- [render] line 4, column 3, at 0.500 s: fetch is not available"),
        "{message}"
    );
    assert_eq!(error.code(), ErrorCode::TemplateInvalid);

    let silent = service
        .check(ProjectId::from_test_index(1), &format, "x")
        .await;
    assert!(matches!(silent, Err(AppError::RendererUnavailable { .. })));
    link.set_connected(false);
    let closed = service
        .check(ProjectId::from_test_index(1), &format, "x")
        .await;
    assert!(matches!(closed, Err(AppError::RendererUnavailable { .. })));
}

#[tokio::test(start_paused = true)]
async fn users_insert_instances_and_remove_unused_templates() {
    let bed = bed().await;
    let placed = bed
        .services
        .projects
        .add_graphic_item(&AddGraphicRequest {
            project_id: bed.project_id,
            sequence_id: bed.sequence_id,
            graphic: NewGraphic::Template {
                template_id: example_ids::TEMPLATE,
            },
            at: secs(11.0),
            duration: secs(5.0),
            track_id: None,
        })
        .await
        .unwrap();
    let project = bed.project().await;
    // Only the accent, bound to the brand kit's accent colour, gets a value (prompt 25).
    let accent = project.brand_kit.as_ref().unwrap().colors.accent;
    assert_eq!(
        instance_of(&project, placed.item_id).params,
        std::collections::BTreeMap::from([(
            "accent".to_owned(),
            se_domain::TemplateParamValue::Color {
                value: se_domain::Animatable::constant(accent)
            }
        )])
    );
    let used = bed
        .services
        .projects
        .remove_template(bed.project_id, example_ids::TEMPLATE)
        .await
        .unwrap_err();
    assert_eq!(used.code(), ErrorCode::EditRejected);
    bed.services.projects.undo(bed.project_id).await.unwrap();
    bed.services
        .projects
        .delete_items(bed.project_id, &[ItemId::from_test_index(INSTANCE)], false)
        .await
        .unwrap();
    bed.services
        .projects
        .remove_template(bed.project_id, example_ids::TEMPLATE)
        .await
        .unwrap();
    assert_eq!(bed.project().await.templates, []);
    bed.services.projects.undo(bed.project_id).await.unwrap();
    assert_eq!(bed.project().await.templates.len(), 1);
}
