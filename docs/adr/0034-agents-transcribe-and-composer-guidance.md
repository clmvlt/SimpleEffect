# 0034 — Agents start transcriptions; the composer warns about scope and speech, and continues recent tasks

- Status: Accepted
- Date: 2026-11-08

## Context
A user asked an agent to « cut the silences and every repeated take » of a talking-head video. The
agent had the whole sequence in scope, cut the silences, but could not find the repeated takes:
the media had no transcript, no speech model was installed, and agents were told « you cannot
transcribe yourself » (ADR-0031 left transcription to the user's « Transcrire »). The user then
asked « why couldn't you touch the duplicates? Don't you have the whole timeline? » in the
composer: a **new** task, with no memory of the first one, whose scope came from a clip left
selected after the cut (8 frames on V1 and A1). The agent blamed that scope and promised to
compare the takes « in the transcript », which did not exist. Three gaps:
- agents could not unblock themselves when a transcript was missing, although transcribing with
  an installed model is local and changes no timeline item;
- the composer turned a stray selection into a tiny scope without drawing attention to it, and
  said nothing about media the agent would be unable to read;
- a question about the last task started a fresh session.

## Decision
- **MCP tool `transcribe_media`** (se-mcp) over **`TranscriptionService::transcribe_and_wait`**
  (se-app): it queues the transcription of the given media (default: those played by the sound
  clips of the task's range, else of the whole sequence) that have no transcript of their current
  content and none queued or running — failed or cancelled ones are tried again —, then waits up
  to `wait_seconds` (default 90, at most 110 — under the two minutes after which Claude Code moves
  an MCP call to the background —, polling the job queue every 250 ms) and reports each
  medium's `TranscriptionState` (`transcribed` with its word count, `queued`, `running` with
  progress, `failed`, `cancelled`, `unavailable` — missing file, no sound, still importing —,
  `not_started`). An unfinished wait answers `finished: false`: the agent calls again (nothing
  starts twice). Without a model it fails with `NO_TRANSCRIPTION_MODEL`: the agent tells the user
  to download one; **agents never download** (golden rule 1: downloads stay explicit user
  commands).
- The tool is **not limited by the task's scope** and is annotated `readOnlyHint: false`,
  `destructiveHint: false` (new `McpTool::DESTRUCTIVE`, defaulting to `!READ_ONLY`): a transcript
  is a fact stored next to the project like analyses, its summary a system transaction, and the
  task's changeset (and revert) is untouched.
- **Agent instructions**: transcribe untranscribed media with `transcribe_media` instead of asking
  for « Transcrire »; before saying something is blocked, check every cause (scope, transcripts,
  analyses), use the tool that unblocks it, and say how to widen a scope (nothing selected, or the
  scope chip removed). The smart-cut and caption tools' notes point at `transcribe_media` too.
- **Composer notices** (`features/ai`, `ComposerNotices`):
  - a scope taken from the **selection covering less than 10 % of the sequence** is spelled out
    (« L'IA ne pourra modifier que la sélection (…, pistes …) ») with « Toute la séquence »;
  - **untranscribed sound media while transcription cannot run** (no model, or whisper.cpp
    missing; from `getTranscriptionStatus`, followed through `transcriptionStatusChanged`) are
    named with « Réglages »; with a model installed nothing is said, since the agent transcribes;
  - **« Continuer la tâche #N »**, ticked by default when the project's most recently run task is
    over, works on the composer's sequence, has a session and finished less than 30 minutes ago:
    the prompt is sent as `followUpTask` with the composer's current scope (which replaces the
    task's, as follow-ups already allow). Unticking it (remembered per task until the next send)
    starts a new task. « Refaire avec l'IA » keeps its own resume option and widened scope.

## Consequences
- « Cut the repeated takes » works end to end with a model installed: the agent transcribes,
  reads, then cuts. Long media make the agent wait (seconds per minute of speech on a GPU, more on
  a CPU); a wait past 110 s takes several calls.
- An agent's transcription occupies the single transcription slot like the user's own; the user
  sees its job in the media bin and can cancel it (the agent then reads `cancelled`).
- A follow-up resumes a longer session (more input tokens) than a new task: the default is
  limited to recent tasks and can be unticked.
- `FakeTranscriber` gains `hold`/`release` so waits are testable.

## Alternatives considered
- **Keep asking the user to click « Transcrire »**: the cause of this ADR; the agent stops halfway
  through a request it could finish locally.
- **Transcribe automatically when a task starts**: costs GPU time for tasks that never read words
  (titles, motion); the agent knows when it needs them.
- **Let agents download a model**: breaks the explicit-consent rule for downloads (ADR-0031).
- **A non-blocking tool plus a status tool**: two tools and polling logic in every agent; one tool
  that waits a bounded time and is safe to call again does both.
- **Ignore the selection for AI scopes**: selecting what to change is a useful gesture; pointing
  out a small one keeps it while preventing accidents.
- **Always continue the last task** (chat mode): sessions grow without bound and unrelated
  requests inherit stale context; a recent-task default with an untick keeps both.
