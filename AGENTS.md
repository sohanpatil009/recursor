# AGENTS.md — Agentic Project Guide

## Status

Greenfield Rust workspace. No code exists yet. Deadline: **June 12, 2026** (20-day vibecode sprint). See [VIBECODE.md](./VIBECODE.md) for daily sprint plan, [TASK.md](./task.md) for task board.

## Commands

```bash
cargo check --workspace        # primary verification (verified: compiles clean)
cargo test --workspace         # all tests
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
cargo build --release -p agent-cli  # ship CLI binary
cargo run -p agent-cli -- run "<prompt>"   # run agent (requires OPENAI_API_KEY)
```

CI: GitHub Actions runs `check → test → clippy → fmt` on push/PR.

**ENV:** `OPENAI_API_KEY` must be set for OpenAI provider. No local LLM support yet.

## Workspace Structure

```
agentic/          workspace root (Cargo.toml)
├── crates/
│   ├── agent-core/       # Traits, types, event bus. Zero deps (only serde).
│   ├── agent-llm/        # LLMProvider trait, OpenAI implementation, structured output
│   ├── agent-tools/      # Tool trait + implementations (filesystem, shell, http, search)
│   ├── agent-verifier/   # Verification gates + reflection loop
│   ├── agent-runtime/    # Orchestrator, scheduler, AgentEngine facade
│   ├── agent-cli/        # CLI binary (clap)
│   └── agent-ui/         # Shared Dioxus components
└── apps/
    └── desktop/           # Dioxus desktop app binary
```

## Crate Dependency Order (ENFORCED — do not violate)

```
agent-core → agent-llm → agent-tools → agent-verifier → agent-runtime → agent-ui
                                                              ↓
                                                         agent-cli
                                                              ↓
                                                         apps/desktop
```

## Sprint Scope — What We ARE Building

- **Single agent loop:** think → plan → execute → verify → retry → finalize
- **4 tools:** filesystem (read/write/list), shell (restricted), HTTP (GET/POST), search
- **4 verification gates:** structural, deterministic, LLM critic, tool-based (compile/lint/test)
- **Reflection loop:** max 3 retries, exponential backoff, human escalation on failure
- **Memory:** SQLite task history + checkpoint/resume (no vector DB)
- **CLI:** `agentic run`, `agentic task list`, `agentic task status`
- **Dioxus desktop UI:** task input, live log, verdict display, approval dialog

## Hard Cut — DO NOT Build (deferred past June 12)

Multi-agent, local LLM, vector DB/embeddings, browser automation, WASM plugins, Docker/Firecracker sandbox, OpenTelemetry/Prometheus, mobile (Android/iOS), auth/RBAC, Gate 5 self-consistency, distributed workers, supervisor agent, data analysis tool, authentication.

## Architecture Rules

- **agent-core must have zero external deps** — only `serde` with derive feature allowed. All types derive Serialize/Deserialize.
- **Structured LLM output from day 1** — never parse free-form text. Every LLM call uses `generate_structured<T>()` with JSON schema.
- **Event bus:** `tokio::sync::broadcast` channel. Every state transition emits `AgentEvent`. UI subscribes to events — never calls engine internals directly.
- **Verification gates run sequentially, short-circuit on fail.**
- **Dioxus UI is thin:** read-only viewer + command dispatcher. All logic in `agent-runtime`.
- **No refactoring during sprint.** Ugly working code > beautiful nonexistent code.
- **Tool permissions:** simple allowlist per tool, path restriction to cwd. No elaborate RBAC.
- **Shell tool:** blocklist for dangerous commands (`rm -rf /`, `shutdown`, `format`), 30s timeout, 1MB output cap.
- **OpenAI API key:** read from `OPENAI_API_KEY` env var. No local LLM support yet.
- **If a feature isn't done by its target day, cut it.** No slipping the June 12 ship date.

## Verification Pipeline Order

```rust
Gate 1: StructuralVerifier   // JSON schema, required fields, regex
Gate 2: DeterministicVerifier // exit code, output bounds, tool executed
Gate 3: LLMVerifier          // critic LLM evaluates output, returns confidence 0-1
Gate 4: ToolVerifier         // compile, lint, test (language-specific)
// Gate 5 (SelfConsistency) — CUT. Not building.
```

## Language Support (Code Verifier)

| Language | Compile | Lint | Test |
|----------|---------|------|------|
| Rust | `cargo check` | `cargo clippy` | `cargo test` |
| Python | `python -m py_compile` | `ruff` | `pytest` |
| JS/TS | `node --check` / `tsc --noEmit` | `eslint` | `jest` |

Extend via `LanguageConfig` struct. Do not add languages without team vote.

## Testing Priorities

1. Unit tests for core types (serialization round-trips)
2. Integration test: full task execution with real OpenAI API
3. Integration test: broken code → retry → fix → pass (verification pipeline)
4. Unit tests for each verification gate with known pass/fail cases
5. Edge cases: empty task, very long output, shell special chars, tool timeouts

## Reference Docs

- [VIBECODE.md](./VIBECODE.md) — active sprint plan, daily tasks, hard cut list
- [TASK.md](./task.md) — 86-task sprint board with status tracking
- [architecture.md](./architecture.md) — full type definitions, code examples, data flow (reference only)
- [README.md](./README.md) — high-level overview, crate descriptions
