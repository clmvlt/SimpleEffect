# 0001 — Electron shell with a Rust core process

- Status: Accepted
- Date: 2026-11-06

## Context
SimpleEffect must run on Windows, macOS and Linux. Its rendering engine depends on WebCodecs, WebGL2/WebGPU and AudioWorklet behaving identically on every OS (see ADR-0002). The backend work (project state, media jobs, AI agent orchestration, MCP server, export) benefits from Rust's performance, safety and concurrency. The user asked for a fast stack and suggested Rust.

## Decision
- **Shell: Electron.** It ships the same Chromium on all three OSes. The main process only manages windows, menus, native dialogs and the core process; it holds no business logic.
- **Core: a separate Rust process, `se-core`**, spawned and supervised by Electron main. It serves, on 127.0.0.1 with a random port and token: a WebSocket RPC endpoint (`/rpc`), HTTP media routes with Range support (`/media`), and the MCP endpoint for AI agents (`/mcp`).
- The renderer talks to the core directly over WebSocket/HTTP. The handshake (port + token) is printed by the core on stdout and passed to the renderer through the preload script.

## Consequences
- The core can be run and tested without Electron (integration tests over HTTP/WS, headless use).
- A core crash does not kill the UI; the supervisor restarts it and the renderer reconnects.
- The shell is replaceable (e.g. Tauri on a future platform) without touching the core or the UI.
- App size is larger (~150 MB) — acceptable for a video editor.
- We must handle a local server's security: loopback only, bearer token, Host header check.

## Alternatives considered
- **Tauri 2**: smaller and Rust-native, but uses the system WebView: WebKitGTK on Linux lacks complete WebCodecs/WebGPU support. Rejected because Linux is a target.
- **Rust native addon (napi-rs) inside Electron main**: lower latency but a native crash kills the main process, harder to test standalone, and HTTP servers would still be needed. Rejected.
- **Native Rust UI (egui, iced, Slint, GPUI)**: excellent performance, but incompatible with a shadcn-style design system, much slower to build, less familiar to AI maintainers. Rejected.
- **Qt/C++ (MLT, like Kdenlive/Shotcut)**: proven, but heavy C++ maintenance and poor fit for AI-driven development. Rejected.
