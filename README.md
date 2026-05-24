# Agentic — Production-Grade Autonomous AI Agent Platform

A cross-platform autonomous AI agent platform built in Rust and Dioxus, focused on reasoning, planning, execution, and self-verification. Sprint target: **June 12, 2026**.

## Status — Day 2 (May 24, 2026)

**Full end-to-end agent loop verified working with Gemini 3.5 Flash:**

```
User Prompt → Think → Plan → Execute → Verify → Complete
```

| Component | Status |
|-----------|--------|
| Filesystem (read/write/list) | ✅ Verified |
| Search (DuckDuckGo) | ✅ Verified |
| HTTP (GET/POST) | ✅ Verified |
| Shell (restricted, blocked commands) | ✅ Verified |
| 4 verification gates pipeline | ✅ Integrated, error-tolerant |
| Reflection loop (max 3 retries) | ✅ Wired in CLI |
| Human escalation (CLI prompt) | ✅ Implemented |
| SQLite memory persistence | ✅ Wired in CLI |
| Dioxus desktop UI | ✅ Engine-wired |
| Edge case handling (empty task, special chars) | ✅ Tested |

**34 tests pass** across all crates — including gate tests, pipeline tests, shell blocklist tests, and failure tracker tests.

## Workspace Structure

```
agentic/
├── crates/
│   ├── agent-core/         # Traits, types, event bus. Zero deps (only serde)
│   ├── agent-llm/          # LLMProvider trait, OpenAI + Gemini implementations
│   ├── agent-tools/        # Tool trait + filesystem, shell, http, search tools
│   ├── agent-verifier/     # 4 verification gates + reflection loop
│   ├── agent-runtime/      # Orchestrator, GenericAgent, SqliteMemoryStore, AgentEngine
│   ├── agent-cli/          # CLI binary (clap: run, task list, task status)
│   └── agent-ui/           # Shared Dioxus components
├── apps/
│   └── desktop/            # Dioxus desktop binary
├── Cargo.toml              # Workspace root
└── rust-toolchain.toml
```

### Crate Dependency Order (ENFORCED)

```
agent-core → agent-llm → agent-tools → agent-verifier → agent-runtime → agent-ui
                                                                    ↓
                                                               agent-cli
                                                                    ↓
                                                               apps/desktop
```

## Architecture

### Agent Loop

```
think() → create_plan() → execute_step() → verify() → retry if failed → complete
```

### Verification Pipeline (4 Gates)

1. **StructuralVerifier** — JSON schema, required fields, regex
2. **DeterministicVerifier** — exit code, output bounds, tool executed
3. **LLMVerifier** — critic LLM evaluates output, returns confidence 0-1
4. **CodeVerifier** — compile (cargo check / py_compile / tsc --noEmit), lint, test

Gates run sequentially, short-circuit on fail.

### Event Bus

All state transitions emit `AgentEvent` via `tokio::sync::broadcast`. UI subscribes to events — never calls engine internals directly.

## Quick Start

```bash
# Prerequisites
rustup target add wasm32-unknown-unknown  # for web build

# Set API key
echo "GEMINI_API_KEY=your_key_here" > .env

# Run CLI
cargo run -p agent-cli -- run "create hello.txt with Hello World"
cargo run -p agent-cli -- run "search for rust programming language"
cargo run -p agent-cli -- run "fetch https://example.com"

# Run desktop app
cargo run -p apps-desktop

# Run tests
cargo test --workspace
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

## Key Design Principles

1. **Verification-first** — Build self-verification early. Without robust verification, it's just a fancy autocomplete.
2. **Structured outputs from day one** — Every LLM call uses `generate_structured<T>()` with JSON schema. Never parse free-form text.
3. **UI is thin** — Read-only viewer + command dispatcher. All logic in `agent-runtime`.
4. **No refactoring during sprint** — Ugly working code > beautiful dead code. If it's not done by target day, cut it.
5. **SQLite for MVP** — Task history, logs, memory, checkpoints all in SQLite.
6. **Start with coding tasks** — Filesystem + code tools are the highest-value first use case.
7. **Hard cuts** — No multi-agent, no local LLM, no vector DB, no browser automation, no plugins, no Docker sandbox, no mobile.

## Hard Cuts (post-June 12)

Multi-agent, local LLM, vector DB/embeddings, browser automation, WASM plugins, Docker/Firecracker sandbox, OpenTelemetry/Prometheus, mobile (Android/iOS), auth/RBAC, Gate 5 self-consistency, distributed workers, supervisor agent, data analysis tool, authentication.

## Language Support (Code Verifier)

| Language | Compile | Lint | Test |
|----------|---------|------|------|
| Rust | `cargo check` | `cargo clippy` | `cargo test` |
| Python | `python -m py_compile` | `ruff` | `pytest` |
| JS/TS | `node --check` / `tsc --noEmit` | `eslint` | `jest` |

## Commands

```bash
cargo check --workspace         # primary verification
cargo test --workspace          # all tests
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
cargo build --release -p agent-cli  # ship CLI binary
cargo run -p agent-cli -- run "<prompt>"  # run agent
```

## Reference

- [VIBECODE.md](./VIBECODE.md) — daily sprint plan
- [TASK.md](./task.md) — 86-task sprint board
- [ARCHITECTURE.md](./architecture.md) — full type definitions, data flow
- [AGENTS.md](./AGENTS.md) — agentic project guide for AI assistants

## License

MIT
