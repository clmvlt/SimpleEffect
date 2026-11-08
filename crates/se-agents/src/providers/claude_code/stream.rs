//! [`StreamParser`]: Claude Code's `--output-format stream-json --verbose` lines → [`AgentEvent`]s.
//!
//! One JSON object per line, tagged by `type`: `system` (`subtype: "init"` carries the session
//! id and model), `assistant` (a message whose content blocks are `text`, `tool_use`,
//! `thinking`…), `user` (tool results), `result` (the final answer, usage, cost, duration and
//! success or error). `rate_limit_event` lines (2.1.319: the account's usage windows) and other
//! unknown types and blocks are logged at debug level and ignored, so newer
//! CLI versions keep working; lines that are not JSON are counted (a run that ends without a
//! result after such lines failed for a protocol reason).

use std::collections::HashMap;

use se_protocol::{AgentEvent, AgentFailureKind};
use serde_json::Value;

use super::failure;

/// `message.model` of the messages the CLI writes itself instead of the model.
const SYNTHETIC_MODEL: &str = "<synthetic>";

/// Characters kept of a tool call's input or output.
const SUMMARY_CHARS: usize = 300;

/// Parses the lines of one run, in order.
#[derive(Debug, Default)]
pub(super) struct StreamParser {
    /// Tool names by call id: results only carry the id.
    tool_names: HashMap<String, String>,
    session_id: Option<String>,
    malformed_lines: usize,
}

impl StreamParser {
    /// The events one line produces (often none).
    pub fn parse_line(&mut self, line: &str) -> Vec<AgentEvent> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        let Ok(json @ Value::Object(_)) = serde_json::from_str::<Value>(line) else {
            self.malformed_lines += 1;
            tracing::warn!(line = %summarize(line), "unreadable line from Claude Code ignored");
            return Vec::new();
        };
        match json["type"].as_str().unwrap_or_default() {
            "system" => self.system(&json),
            "assistant" => self.assistant(&json),
            "user" => self.tool_results(&json),
            "result" => result(&json),
            other => {
                tracing::debug!(kind = other, "Claude Code line type ignored");
                Vec::new()
            }
        }
    }

    /// Lines that were not JSON objects.
    pub const fn malformed_lines(&self) -> usize {
        self.malformed_lines
    }

    fn system(&mut self, json: &Value) -> Vec<AgentEvent> {
        if json["subtype"].as_str() != Some("init") {
            tracing::debug!(subtype = ?json["subtype"].as_str(), "Claude Code system line ignored");
            return Vec::new();
        }
        let Some(session_id) = json["session_id"].as_str() else {
            return Vec::new();
        };
        if self.session_id.as_deref() == Some(session_id) {
            return Vec::new();
        }
        self.session_id = Some(session_id.to_owned());
        vec![AgentEvent::SessionStarted {
            session_id: session_id.to_owned(),
            model: json["model"].as_str().map(str::to_owned),
        }]
    }

    fn assistant(&mut self, json: &Value) -> Vec<AgentEvent> {
        // Messages the CLI writes itself (`Not logged in · Please run /login`, API errors) are
        // repeated by the `result` line, which reports them as the failure.
        if json["message"]["model"].as_str() == Some(SYNTHETIC_MODEL) {
            tracing::debug!("synthetic Claude Code message ignored");
            return Vec::new();
        }
        let mut events = Vec::new();
        for block in blocks(&json["message"]["content"]) {
            match block["type"].as_str().unwrap_or_default() {
                "text" => {
                    let text = block["text"].as_str().unwrap_or_default();
                    if !text.trim().is_empty() {
                        events.push(AgentEvent::AssistantText {
                            text: text.to_owned(),
                        });
                    }
                }
                "tool_use" => {
                    let call_id = block["id"].as_str().unwrap_or_default().to_owned();
                    let tool = block["name"].as_str().unwrap_or_default().to_owned();
                    self.tool_names.insert(call_id.clone(), tool.clone());
                    events.push(AgentEvent::ToolCallStarted {
                        call_id,
                        tool,
                        input_summary: summarize(&block["input"].to_string()),
                    });
                }
                other => tracing::debug!(block = other, "Claude Code content block ignored"),
            }
        }
        events
    }

    fn tool_results(&self, json: &Value) -> Vec<AgentEvent> {
        blocks(&json["message"]["content"])
            .filter(|block| block["type"].as_str() == Some("tool_result"))
            .map(|block| {
                let call_id = block["tool_use_id"].as_str().unwrap_or_default().to_owned();
                AgentEvent::ToolCallFinished {
                    tool: self.tool_names.get(&call_id).cloned().unwrap_or_default(),
                    call_id,
                    ok: !block["is_error"].as_bool().unwrap_or(false),
                    output_summary: summarize(&result_text(&block["content"])),
                }
            })
            .collect()
    }
}

/// The usage and terminal events of the `result` line.
fn result(json: &Value) -> Vec<AgentEvent> {
    let usage = &json["usage"];
    let tokens = |field: &str| usage[field].as_u64().unwrap_or(0);
    let mut events = vec![AgentEvent::Usage {
        input_tokens: tokens("input_tokens")
            + tokens("cache_creation_input_tokens")
            + tokens("cache_read_input_tokens"),
        output_tokens: tokens("output_tokens"),
        cost_usd: json["total_cost_usd"].as_f64(),
        duration_ms: json["duration_ms"].as_u64(),
    }];
    let subtype = json["subtype"].as_str().unwrap_or_default();
    let is_error = json["is_error"].as_bool().unwrap_or(false) || subtype != "success";
    let text = json["result"].as_str().unwrap_or_default();
    if !is_error {
        events.push(AgentEvent::Completed {
            summary: text.to_owned(),
        });
        return events;
    }
    let detail = error_detail(json, subtype, text);
    events.push(failure::failed(&detail, None));
    events
}

/// The content blocks of a message (an array, or a lone string standing for one text block).
fn blocks(content: &Value) -> impl Iterator<Item = &Value> {
    content.as_array().into_iter().flatten()
}

/// What went wrong according to a `result` line.
fn error_detail(json: &Value, subtype: &str, text: &str) -> String {
    if !text.trim().is_empty() {
        return text.to_owned();
    }
    let errors: Vec<String> = json["errors"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|error| {
            error
                .as_str()
                .map_or_else(|| error.to_string(), str::to_owned)
        })
        .collect();
    if !errors.is_empty() {
        return errors.join("; ");
    }
    match subtype {
        "error_max_turns" => "the agent stopped after reaching its turn limit".to_owned(),
        "error_during_execution" => "Claude Code failed while running the task".to_owned(),
        other => format!("Claude Code ended with `{other}`"),
    }
}

/// The text of a tool result: a string, or text blocks (images shown as `[image]`).
fn result_text(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts
            .iter()
            .map(|part| match part["type"].as_str() {
                Some("text") => part["text"].as_str().unwrap_or_default().to_owned(),
                Some("image") => "[image]".to_owned(),
                _ => String::new(),
            })
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// `text` shortened to [`SUMMARY_CHARS`] characters, with an ellipsis when cut.
fn summarize(text: &str) -> String {
    match text.char_indices().nth(SUMMARY_CHARS) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_owned(),
    }
}

/// The terminal event of a run whose process ended without a `result` line.
pub(super) fn missing_result(
    parser: &StreamParser,
    exited_ok: bool,
    status: &str,
    stderr: &str,
) -> AgentEvent {
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return failure::failed(stderr, Some(&format!("Claude Code stopped ({status})")));
    }
    if parser.malformed_lines() > 0 || exited_ok {
        return AgentEvent::Failed {
            failure: AgentFailureKind::ProtocolError,
            message: format!(
                "Claude Code ended without a result ({status}, {} unreadable lines): this version \
                 of Claude Code may not be supported; update SimpleEffect or report it",
                parser.malformed_lines()
            ),
        };
    }
    AgentEvent::Failed {
        failure: AgentFailureKind::Crashed,
        message: format!("Claude Code stopped unexpectedly ({status}): retry the task"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(lines: &[&str]) -> (StreamParser, Vec<AgentEvent>) {
        let mut parser = StreamParser::default();
        let events = lines
            .iter()
            .flat_map(|line| parser.parse_line(line))
            .collect();
        (parser, events)
    }

    #[test]
    fn a_tool_call_and_its_result_are_paired_by_id() {
        let (_, events) = parse(&[
            r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"hm"},{"type":"text","text":"Let me look."},{"type":"tool_use","id":"toolu_1","name":"mcp__simpleeffect__get_timeline","input":{"sequence":"s"}}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"toolu_1","is_error":true,"content":[{"type":"text","text":"OUT_OF_SCOPE"},{"type":"image","source":{}}]}]}}"#,
        ]);
        assert_eq!(
            events,
            [
                AgentEvent::AssistantText {
                    text: "Let me look.".to_owned()
                },
                AgentEvent::ToolCallStarted {
                    call_id: "toolu_1".to_owned(),
                    tool: "mcp__simpleeffect__get_timeline".to_owned(),
                    input_summary: r#"{"sequence":"s"}"#.to_owned(),
                },
                AgentEvent::ToolCallFinished {
                    call_id: "toolu_1".to_owned(),
                    tool: "mcp__simpleeffect__get_timeline".to_owned(),
                    ok: false,
                    output_summary: "OUT_OF_SCOPE\n[image]".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn the_session_is_announced_once() {
        let init = r#"{"type":"system","subtype":"init","session_id":"abc","model":"claude-opus"}"#;
        let (_, events) = parse(&[init, init, r#"{"type":"system","subtype":"status"}"#]);
        assert_eq!(
            events,
            [AgentEvent::SessionStarted {
                session_id: "abc".to_owned(),
                model: Some("claude-opus".to_owned())
            }]
        );
    }

    #[test]
    fn error_results_are_classified() {
        let (_, events) = parse(&[
            r#"{"type":"result","subtype":"success","is_error":true,"result":"Claude AI usage limit reached|1767225600","duration_ms":12,"total_cost_usd":0,"usage":{"input_tokens":0,"output_tokens":0}}"#,
        ]);
        let AgentEvent::Failed { failure, message } = &events[1] else {
            panic!("expected a failure, got {events:?}");
        };
        assert_eq!(*failure, AgentFailureKind::RateLimited);
        assert!(message.contains("retry the task once it resets"));
        let (_, events) =
            parse(&[r#"{"type":"result","subtype":"error_max_turns","is_error":true}"#]);
        assert!(matches!(
            &events[1],
            AgentEvent::Failed { failure: AgentFailureKind::Crashed, message } if message.contains("turn limit")
        ));
    }

    #[test]
    fn malformed_and_unknown_lines_are_ignored() {
        let (parser, events) = parse(&[
            "not json",
            "[1,2]",
            "",
            r#"{"type":"stream_event","event":{}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"server_tool_use"}]}}"#,
        ]);
        assert_eq!(events, []);
        assert_eq!(parser.malformed_lines(), 2);
        let event = missing_result(&parser, true, "exit status: 0", "");
        assert!(matches!(
            event,
            AgentEvent::Failed {
                failure: AgentFailureKind::ProtocolError,
                ..
            }
        ));
    }

    #[test]
    fn runs_ending_without_a_result_are_explained_from_stderr() {
        let parser = StreamParser::default();
        let lost = missing_result(
            &parser,
            false,
            "exit status: 1",
            "No conversation found with session ID: 42",
        );
        assert!(
            matches!(&lost, AgentEvent::Failed { message, .. } if message.contains("no longer exists"))
        );
        let logged_out = missing_result(
            &parser,
            false,
            "exit status: 1",
            "Invalid API key · Please run /login",
        );
        assert!(matches!(
            logged_out,
            AgentEvent::Failed {
                failure: AgentFailureKind::NotAuthenticated,
                ..
            }
        ));
        let crashed = missing_result(&parser, false, "signal: 9", "");
        assert!(matches!(
            crashed,
            AgentEvent::Failed {
                failure: AgentFailureKind::Crashed,
                ..
            }
        ));
    }

    #[test]
    fn long_summaries_are_cut_on_a_character_boundary() {
        let text = "é".repeat(400);
        let summary = summarize(&text);
        assert_eq!(summary.chars().count(), SUMMARY_CHARS + 1);
        assert!(summary.ends_with('…'));
    }

    /// Events of a recorded run (`tests/fixtures/claude_code/`).
    fn replay(fixture: &str) -> Vec<AgentEvent> {
        parse(&fixture.lines().collect::<Vec<_>>()).1
    }

    fn text(event: &AgentEvent) -> &str {
        match event {
            AgentEvent::AssistantText { text } | AgentEvent::Completed { summary: text } => text,
            AgentEvent::Failed { message, .. } => message,
            other => panic!("expected a text, got {other:?}"),
        }
    }

    const SESSION: &str = "0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f91";

    #[test]
    fn a_recorded_success_gives_the_session_the_text_the_usage_and_the_answer() {
        let events = replay(include_str!(
            "../../../tests/fixtures/claude_code/success.jsonl"
        ));
        assert_eq!(events.len(), 4, "{events:?}");
        assert_eq!(
            events[0],
            AgentEvent::SessionStarted {
                session_id: SESSION.to_owned(),
                model: Some("claude-haiku-5-5".to_owned())
            }
        );
        assert_eq!(text(&events[1]), "Bonjour");
        let AgentEvent::Usage {
            input_tokens,
            output_tokens,
            cost_usd,
            duration_ms,
        } = events[2]
        else {
            panic!("expected usage, got {:?}", events[2]);
        };
        assert!(input_tokens > 0 && output_tokens > 0);
        assert!(cost_usd.is_some() && duration_ms.is_some());
        assert_eq!(text(&events[3]), "Bonjour");
    }

    #[test]
    fn a_recorded_mcp_run_reports_its_tool_call_and_result() {
        // Recorded through se-core (prompt 18): Claude Code 2.1.319, sonnet, asked to read the
        // timeline with the `simpleeffect` MCP server.
        let events = replay(include_str!(
            "../../../tests/fixtures/claude_code/mcp-get-timeline.jsonl"
        ));
        assert!(matches!(
            &events[0],
            AgentEvent::SessionStarted { session_id, .. } if session_id == SESSION
        ));
        let AgentEvent::ToolCallStarted {
            call_id,
            tool,
            input_summary,
        } = &events[1]
        else {
            panic!("expected the tool call, got {:?}", events[1]);
        };
        assert_eq!(tool, "mcp__simpleeffect__get_timeline");
        assert!(input_summary.contains("range"), "{input_summary}");
        let AgentEvent::ToolCallFinished {
            call_id: finished,
            tool: finished_tool,
            ok,
            output_summary,
        } = &events[2]
        else {
            panic!("expected the tool result, got {:?}", events[2]);
        };
        assert_eq!((finished, finished_tool, *ok), (call_id, tool, true));
        assert!(output_summary.contains("sequence_id"), "{output_summary}");
        assert!(text(&events[3]).contains("3 éléments"));
        assert!(matches!(events.last(), Some(AgentEvent::Completed { .. })));
    }

    #[test]
    fn a_recorded_resume_keeps_the_session() {
        let events = replay(include_str!(
            "../../../tests/fixtures/claude_code/resume.jsonl"
        ));
        assert!(matches!(
            &events[0],
            AgentEvent::SessionStarted { session_id, .. } if session_id == SESSION
        ));
        assert_eq!(text(events.last().unwrap()), "Bonjour");
    }

    #[test]
    fn a_recorded_long_answer_completes_with_every_line() {
        let events = replay(include_str!(
            "../../../tests/fixtures/claude_code/long-text.jsonl"
        ));
        let summary = text(events.last().unwrap());
        assert!(summary.lines().count() >= 5, "{summary}");
        assert!(matches!(events.last(), Some(AgentEvent::Completed { .. })));
    }

    #[test]
    fn a_recorded_lost_session_says_to_retry() {
        let events = replay(include_str!(
            "../../../tests/fixtures/claude_code/lost-session.jsonl"
        ));
        let Some(AgentEvent::Failed { failure, message }) = events.last() else {
            panic!("expected a failure, got {events:?}");
        };
        assert_eq!(*failure, AgentFailureKind::Crashed);
        assert!(
            message.starts_with("the agent session to resume no longer exists"),
            "{message}"
        );
    }

    #[test]
    fn a_recorded_logged_out_run_fails_once_without_the_synthetic_text() {
        let events = replay(include_str!(
            "../../../tests/fixtures/claude_code/not-logged-in.jsonl"
        ));
        assert!(matches!(events[0], AgentEvent::SessionStarted { .. }));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::AssistantText { .. }))
        );
        assert!(matches!(
            events.last(),
            Some(AgentEvent::Failed {
                failure: AgentFailureKind::NotAuthenticated,
                ..
            })
        ));
    }
}
