//! Benchmarks of the core's hot paths (prompt 29, `docs/performance.md`): committing an edit on a
//! large project, the patch and snapshot sent to clients, `project.json` serialisation, and the
//! store (journal append with `synchronous = FULL`, snapshot write, opening with a journal to
//! replay). Run with `pnpm bench` (or `cargo bench -p se-devtools`); add `-- --quick` for a fast
//! pass. Projects come from the generator of `pnpm generate-test-project`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{hint::black_box, path::Path, time::Duration};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use se_app::{
    ports::{JournalEntry, ProjectRepository},
    project_view::{project_patch, project_snapshot},
    recovery::replay,
};
use se_devtools::project::{build_project, synthetic_assets};
use se_domain::{
    Actor, Flicks, ItemId, MoveItem, Project, TrackId, Transaction, TransactionId, commit,
};
use se_store::FsProjectRepository;

/// Item counts measured: the budget's 500 and a stress size.
const SIZES: [usize; 2] = [500, 2000];

fn project(items: usize) -> Project {
    build_project("Bench", synthetic_assets(), items, 1).expect("a valid generated project")
}

/// Moves the last item of the first video track 10 s later: always valid, touches one item.
fn nudge(project: &Project, index: u64) -> Transaction {
    let track = &project.sequences[0].tracks[0];
    let item = track.items.last().expect("V1 holds items");
    let start = item
        .timeline_range
        .start()
        .checked_add(Flicks::new(10 * Flicks::PER_SECOND).unwrap())
        .unwrap();
    move_transaction(index, item.id, track.id, start)
}

fn move_transaction(index: u64, item: ItemId, track: TrackId, start: Flicks) -> Transaction {
    Transaction {
        id: TransactionId::from_test_index(index),
        actor: Actor::User,
        label: "Move items".to_owned(),
        ops: vec![
            MoveItem {
                item,
                track,
                start,
                expected_revision: None,
            }
            .into(),
        ],
    }
}

fn domain(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("domain");
    for size in SIZES {
        let project = project(size);
        let transaction = nudge(&project, 1);
        group.bench_function(format!("commit_move/{size}"), |bench| {
            bench.iter(|| commit(black_box(&project), black_box(&transaction)).unwrap());
        });
        let committed = commit(&project, &transaction).unwrap();
        group.bench_function(format!("patch/{size}"), |bench| {
            bench.iter(|| {
                project_patch(
                    black_box(&project),
                    black_box(&committed.project),
                    &committed.touched,
                    Actor::User,
                )
            });
        });
        group.bench_function(format!("snapshot/{size}"), |bench| {
            bench.iter(|| project_snapshot(black_box(&project)));
        });
    }
    group.finish();
}

fn serialisation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("serialisation");
    for size in SIZES {
        let project = project(size);
        let json = serde_json::to_vec_pretty(&project).unwrap();
        group.bench_function(format!("write_project_json/{size}"), |bench| {
            bench.iter(|| serde_json::to_vec_pretty(black_box(&project)).unwrap());
        });
        group.bench_function(format!("read_project_json/{size}"), |bench| {
            bench.iter(|| serde_json::from_slice::<Project>(black_box(&json)).unwrap());
        });
    }
    group.finish();
}

/// A store over a fresh folder holding `project`.
async fn open_store(parent: &Path, project: &Project) -> Box<dyn se_app::ports::ProjectStore> {
    let path = FsProjectRepository.create(parent, project).await.unwrap();
    let store = FsProjectRepository.open(&path).await.unwrap();
    store.load_snapshot().await.unwrap();
    store
}

/// `count` journal entries nudging the last item of V1 back and forth from `project`.
fn journal(project: &Project, count: u64) -> Vec<JournalEntry> {
    let mut current = project.clone();
    (1..=count)
        .map(|index| {
            let transaction = nudge(&current, index);
            current = commit(&current, &transaction).unwrap().project;
            JournalEntry {
                revision: current.revision,
                committed_at: 0,
                transaction,
            }
        })
        .collect()
}

fn store(criterion: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut group = criterion.benchmark_group("store");
    group.measurement_time(Duration::from_secs(8));
    let project = project(500);
    let folder = tempfile::tempdir().unwrap();
    let store = runtime.block_on(open_store(folder.path(), &project));
    let mut entry = journal(&project, 1).remove(0);
    group.bench_function("append_journal/500", |bench| {
        bench.iter(|| {
            // Revisions are unique in the journal: each append takes the next one.
            entry.revision += 1;
            runtime.block_on(store.append_journal(&entry)).unwrap();
        });
    });
    group.bench_function("save_snapshot/500", |bench| {
        let mut snapshot = project.clone();
        bench.iter(|| {
            snapshot.revision += 1;
            runtime.block_on(store.save_snapshot(&snapshot)).unwrap();
        });
    });
    let replayed = journal(&project, 200);
    group.bench_function("replay_200/500", |bench| {
        bench.iter_batched(
            || project.clone(),
            |snapshot| replay(snapshot, black_box(&replayed)).unwrap(),
            BatchSize::LargeInput,
        );
    });
    group.bench_function("open_with_200_entries/500", |bench| {
        bench.iter_batched(
            || {
                let parent = tempfile::tempdir().unwrap();
                let path: std::path::PathBuf = runtime.block_on(async {
                    let store = open_store(parent.path(), &project).await;
                    for entry in &replayed {
                        store.append_journal(entry).await.unwrap();
                    }
                    let path = store.path().to_path_buf();
                    drop(store);
                    path
                });
                (parent, path)
            },
            |(parent, path): (tempfile::TempDir, std::path::PathBuf)| {
                runtime.block_on(async {
                    let store = FsProjectRepository.open(&path).await.unwrap();
                    let snapshot = store.load_snapshot().await.unwrap().project;
                    let entries = store.read_journal_since(snapshot.revision).await.unwrap();
                    replay(snapshot, &entries).unwrap()
                });
                drop(parent);
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

criterion_group!(benches, domain, serialisation, store);
criterion_main!(benches);
