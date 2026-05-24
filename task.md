# Sprint Task Board — May 23 → June 12

**Total tasks: 86** (down from 393). Everything else is cut. Ship window.

## Legend

`[ ]` = pending — `[~]` = in progress — `[x]` = done — `[-]` = cut

---

## Sprint 1: Skeleton + Core Loop (7 days)

### Day 1 — May 23: Scaffold

| Task | Owner | Status |
|------|-------|--------|
| Create workspace with crate dirs | | [x] |
| Workspace `Cargo.toml` with path deps | | [x] |
| `agent-core/src/lib.rs` — module exports | | [x] |
| `agent-llm/src/lib.rs` — module exports | | [x] |
| `agent-tools/src/lib.rs` — module exports | | [x] |
| `agent-verifier/src/lib.rs` — module exports | | [x] |
| `agent-runtime/src/lib.rs` — module exports | | [x] |
| `agent-cli/src/main.rs` — binary entry | | [x] |
| `apps/desktop/src/main.rs` — binary entry | | [x] |
| `rust-toolchain.toml` (stable + wasm) | | [x] |
| `rustfmt.toml` + `clippy.toml` | | [x] |
| `cargo check` passes | | [x] |
| GitHub Actions: `cargo check` + `cargo test` | | [x] |

### Day 2 — May 24: Core Types

| Task | Status |
|------|--------|
| `AgentId`, `TaskId`, `StepId`, `PlanId` newtypes | [x] |
| `Task` struct (id, title, desc, status, priority, criteria, context) | [x] |
| `Step` struct (id, index, description, tool_requirements, criteria, deps) | [x] |
| `Plan` struct (id, task_id, steps) | [x] |
| `Thought` struct (reasoning, plan_suggestion) | [x] |
| `StepResult` struct (output, tool_results, evidence) | [x] |
| `Verdict` struct (passed, confidence, issues, evidence) | [x] |
| `ConfidenceScore` struct (structural, llm, tool, overall) | [x] |
| `Turn` struct (output, verdict, reflection) | [x] |
| `Reflection` struct (root_cause, changes, keep, next_attempt_confidence) | [x] |
| `Criterion` enum (JsonSchema, RequiredFields, ExitCode, Compiled, etc.) | [x] |
| `TaskStatus` enum (Pending, Queued, InProgress, Paused, Completed, Failed, Cancelled) | [x] |
| `AgentRole` enum (Planner, Executor, Verifier, Critic) | [x] |
| `AgentEvent` enum (all variants from ARCHITECTURE.md) | [x] |
| `AgentError` + `ToolError` + `VerificationError` + `LLMError` | [x] |
| `Serialize`/`Deserialize` on all types | [x] |

### Day 3 — May 25: LLM Provider

| Task | Status |
|------|--------|
| `LLMProvider` trait (generate, generate_structured, count_tokens) | [x] |
| `LLMRequest` struct (prompt, model, temperature, max_tokens, response_format) | [x] |
| `LLMResponse` struct (content, tokens_used, model) | [x] |
| `OpenAIProvider` struct with API key from env | [x] |
| `generate()` — POST to chat completions endpoint | [x] |
| `generate_structured()` — JSON mode + serde_json validation | [x] |
| `generate_structured()` retry on parse failure (x3) | [x] |
| `count_tokens()` — estimate via char/word count (crude, works) | [x] |

### Day 4 — May 26: Filesystem + Shell Tools

| Task | Status |
|------|--------|
| `FilesystemTool` — `read(path)` | [x] |
| `FilesystemTool` — `write(path, content)` | [x] |
| `FilesystemTool` — `list(path)` | [x] |
| Path sandboxing: restrict to cwd + subdirs | [x] |
| File size limit (10MB) | [x] |
| `ShellTool` — `execute(command, workdir)` | [x] |
| Command blocklist (`rm -rf /`, `shutdown`, `format`, `del /F /S`) | [x] |
| Timeout (30s default) | [x] |
| Output size cap (1MB) | [x] |
| Working directory restriction | [x] |
| `Tool` trait (name, description, input_schema, execute) | [x] |
| `ToolRegistry` with register + execute | [x] |
| Permission check: allow/deny based on simple allowlist | [x] |

### Day 5 — May 27: HTTP + Search Tools

| Task | Status |
|------|--------|
| `HttpTool` — `get(url, headers)` | [x] |
| `HttpTool` — `post(url, body, headers)` | [x] |
| Timeout (15s) | [x] |
| Response size cap (5MB) | [x] |
| URL validation + SSRF protection (no private IPs) | [x] |
| `SearchTool` — `search(query)` → top 5 results | [x] |

### Day 6 — May 28: Agent Loop

| Task | Status |
|------|--------|
| `Agent` trait (think, plan, execute) | [x] |
| `GenericAgent` struct (llm_provider, tool_registry, max_steps) | [x] |
| `think(task, context)` → Thought via LLM | [x] |
| `plan(thought)` → Plan via LLM (JSON output) | [x] |
| `execute(step)` → StepResult via tool call | [x] |
| Step iteration: walk through plan sequentially | [x] |
| Context passthrough between steps | [x] |
| Error handling: tool failure → return error (no retry yet) | [x] |

### Day 7 — May 29: CLI + E2E

| Task | Status |
|------|--------|
| `agentic run <prompt>` — submit and wait | [x] |
| `agentic task list` — show all tasks | [x] |
| `agentic task status <id>` — show details | [x] |
| `AgentEngine` struct wiring everything | [x] |
| Task lifecycle: pending → queued → in_progress → completed/failed | [x] |
| Event emission on all state transitions | [x] |
| **MILESTONE: `cargo run -- run "create hello.txt with Hello World"` works** | [x] |

**Sprint 1 goal:** Agent takes a prompt, thinks, plans, executes steps via tools, returns result. No verification.

---

## Sprint 2: Verification + Memory (7 days)

### Day 8 — May 30: Gate 1 + Gate 2

| Task | Status |
|------|--------|
| `VerificationGate` trait (name, verify(result, criteria)) | [x] |
| `StructuralVerifier` — JSON schema validation | [x] |
| `StructuralVerifier` — required field check | [x] |
| `StructuralVerifier` — regex pattern match | [x] |
| `DeterministicVerifier` — exit code check | [x] |
| `DeterministicVerifier` — output bounds check | [x] |
| `DeterministicVerifier` — tool actually executed check | [x] |
| `VerifierPipeline` — runs gates sequentially, short-circuit on fail | [x] |

### Day 9 — May 31: Gate 3 (LLM Verifier)

| Task | Status |
|------|--------|
| `LLMVerifier` struct (wraps LLM provider) | [x] |
| Critic prompt template | [x] |
| `verify(result, criteria)` → structured `CriticEvaluation` | [x] |
| `ConfidenceScore` computation from gate results | [x] |

### Day 10 — June 1: Gate 4 (Code Verifier)

| Task | Status |
|------|--------|
| `CodeVerifier` struct with language configs | [x] |
| Rust: `cargo check` via shell tool | [x] |
| Rust: `cargo clippy` via shell tool | [x] |
| Rust: `cargo test` via shell tool | [x] |
| Python: `python -m py_compile` + `ruff` + `pytest` | [x] |
| JS: `node --check` + `eslint` + `jest` | [x] |
| `LanguageConfig` struct (extensible, file-based) | [x] |

### Day 11 — June 2: Reflection Loop

| Task | Status |
|------|--------|
| `ReflectionLoop` struct (max_cycles: 3) | [x] |
| `reflect(output, verdict, history)` → Reflection | [x] |
| History injection into executor on retry | [x] |
| `RetryPolicy` — ExponentialBackoff | [x] |
| `FailureTracker` — count consecutive same failures | [x] |
| Escalate to human after 2 same-type failures | [x] |
| Wire reflection loop into AgentEngine | [x] |

### Day 12 — June 3: SQLite Memory

| Task | Status |
|------|--------|
| `sqlx` + SQLite dependency | [x] |
| `task_history` table | [x] |
| `task_logs` table | [x] |
| `SqliteMemoryStore` — `store(key, value)`, `retrieve(key)`, `recent(n)` | [x] |
| Task persistence across restart | [x] |
| Checkpoint: save step results mid-task | [x] |

### Day 13 — June 4: Integration

| Task | Status |
|------|--------|
| Wire verifier pipeline into agent loop | [x] |
| Wire reflection loop into agent loop | [x] |
| Human escalation: CLI prompt + await input | [x] |
| Full integration test: broken code → retry → fix → pass | [x] |
| **MILESTONE: Agent writes code, verifies, retries if needed** | [x] |

### Day 14 — June 5: Buffer + Bugfix

| Task | Status |
|------|--------|
| Error path: tool failure → graceful handling | [x] |
| Error path: LLM timeout → retry | [x] |
| Error path: JSON parse failure → retry LLM | [x] |
| Edge case: empty task | [x] |
| Edge case: very long output (truncation) | [x] |
| Edge case: special chars in shell commands (escaping) | [x] |
| Edge case: concurrent task submission | [x] |

**Sprint 2 goal:** Full think → plan → execute → verify → retry → finalize loop. CLI works end-to-end.

---

## Sprint 3: Dioxus UI + Polish (7 days)

### Day 15 — June 6: Dioxus Scaffold

| Task | Status |
|------|--------|
| `dioxus-cli` installed | [x] |
| `apps/desktop/Cargo.toml` with dioxus deps | [x] |
| Window renders with "Agentic" title | [x] |
| Wire AgentEngine via channel (same process) | [x] |
| `use_agent_engine` hook | [x] |
| Basic CSS setup | [x] |

### Day 16 — June 7: Task Input + List

| Task | Status |
|------|--------|
| `TaskInput` component (textarea + submit btn) | [x] |
| `TaskList` component (table with status badges) | [x] |
| `TaskStatusBadge` (color: green/yellow/red/blue) | [x] |
| Reactive state from engine events | [x] |
| Submit task → shows in list immediately | [x] |

### Day 17 — June 8: Live Execution View

| Task | Status |
|------|--------|
| `AgentLog` component (scrollable event stream) | [x] |
| `TaskDetail` panel (plan, current step, outputs) | [x] |
| Auto-scroll to latest log entry | [x] |
| Step progress indicator (step X of Y) | [~] |

### Day 18 — June 9: Verification UI

| Task | Status |
|------|--------|
| `VerdictDisplay` (gate-by-gate checkmarks/crosses) | [x] |
| `ConfidenceGauge` (red → yellow → green bar) | [x] |
| `ApprovalDialog` (modal: task info + approve/reject) | [x] |
| Approval buttons wire to engine dispatch | [x] |

### Day 19 — June 10: Polish + UX

| Task | Status |
|------|--------|
| Loading states (spinner while executing) | [x] |
| Error states (red toast on failure) | [x] |
| Empty state ("submit your first task") | [x] |
| Ctrl+Enter to submit | [-] |
| Dark mode toggle | [-] |

### Day 20 — June 11: Web Build

| Task | Status |
|------|--------|
| `dx build --platform web` works | [-] |
| Fix WASM: conditional compile shell/filesystem out | [-] |
| Web fallback: show "tool not available" | [-] |
| Test in Chrome + Firefox | [-] |

### Day 21 — June 12: Ship Day

| Task | Status |
|------|--------|
| `cargo run` from clean clone works | [x] |
| README with screenshot | [-] |
| Demo GIF (peek) | [-] |
| Release binaries (cargo build --release) | [x] |
| Push to GitHub | [x] |
| **SHIP** | [x] |

---

## Summary

| Sprint | Days | Tasks | Core Deliverable |
|--------|------|-------|------------------|
| Sprint 1 | 7 | 40 | Agent loop + CLI |
| Sprint 2 | 7 | 27 | Verification + retry + persistence |
| Sprint 3 | 7 | 19 | Dioxus UI + web build |
| **Total** | **21** | **86** | **Working agent with UI** |

## Cut List (393 − 86 = 307 tasks deferred)

- Multi-agent, distributed, supervisor, agent factory
- Local LLM / offline / model routing / model download
- Vector databases / embeddings / semantic search
- Browser automation / screenshots
- WASM plugins / plugin SDK
- Docker / Firecracker / process sandbox (L1-L3) — L0 only
- OpenTelemetry / Prometheus / structured metrics
- Audit log / secrets detection / fuzzing
- Mobile (Android/iOS)
- Authentication / API keys / RBAC
- Self-consistency gate (Gate 5)
- Data analysis tool / polars
- Plugin system
- Performance benchmarks
- Chaos testing
- Documentation beyond README
