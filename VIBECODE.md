# Vibecode Sprint — May 23 → June 12 (20 Days)

Zero meetings. Zero planning debates. Ship or it didn't happen.

## The Cut List (Won't Build)

**Hard no — no debate:**
- ❌ Multi-agent coordination
- ❌ Local LLM / offline mode
- ❌ Vector databases
- ❌ Browser automation tool
- ❌ WASM plugins
- ❌ Mobile (Android/iOS)
- ❌ Docker/MicroVM sandboxing
- ❌ OpenTelemetry / Prometheus
- ❌ Distributed workers
- ❌ Plugin system
- ❌ Supervisor agent
- ❌ Self-consistency gate (Gate 5)
- ❌ Data analysis tool
- ❌ Authentication/authorization

**Shippable v2 (after June 12):** Everything above.

## What We ARE Building

```
┌─────────────────────────────────────┐
│  Single Agent Loop                   │
│  Think → Plan → Execute → Verify    │
│         ↓ Retry ↑                   │
│  (reflection loop, max 3 retries)    │
└─────────────────────────────────────┘
├── Filesystem tool (read/write/list)
├── Shell tool (restricted)
├── HTTP tool (GET/POST)
├── Search tool (web search API)
├── Code verifier (compile + lint)
├── Deterministic verifier (exit codes, schema)
├── LLM verifier (critic agent)
├── SQLite memory (task history only)
├── CLI interface
└── Dioxus desktop UI (dashboard + task view)
```

That's it. That's the scope. Anything else is scope creep. Say no.

---

## Sprint 1: Skeleton + Core Loop (May 23-29, 7 days)

### Day 1 (May 23): Scaffold
```bash
cargo new agentic --workspace
cd agentic
cargo add serde serde_json tokio async-trait clap
mkdir -p crates/{agent-core,agent-llm,agent-tools,agent-verifier,agent-runtime,agent-cli}
mkdir -p apps/desktop
```
- [ ] Workspace `Cargo.toml` with path deps
- [ ] `agent-core/src/lib.rs` — Agent trait + types
- [ ] `agent-llm/src/lib.rs` — LLMProvider trait stub

### Day 2 (May 24): Core Types
- [ ] All types in agent-core: `Task`, `Step`, `Plan`, `Thought`, `StepResult`, `Verdict`, `ConfidenceScore`, `Turn`, `Reflection`
- [ ] `AgentError`, `ToolError`, `VerificationError`
- [ ] `AgentEvent` enum with 15 variants
- [ ] `Serialize`/`Deserialize` on everything

### Day 3 (May 25): LLM Provider
- [ ] `OpenAIProvider` struct (API key from env var)
- [ ] `generate()` — non-streaming text completion
- [ ] `generate_structured()` — JSON mode with schema validation
- [ ] `count_tokens()` — crude character-based fallback

### Day 4 (May 26): Tools — Filesystem + Shell
- [ ] `FilesystemTool`: read, write, list (path-restricted to cwd)
- [ ] `ShellTool`: execute command, blocklist for dangerous commands, 30s timeout
- [ ] `ToolRegistry` with basic permission check (no elaborate system, just allow/deny)

### Day 5 (May 27): Tools — HTTP + Search
- [ ] `HttpTool`: GET, POST (timeout, response cap, SSRF protection via URL validation)
- [ ] `SearchTool`: web search API wrapper (returns top 5 results as text)

### Day 6 (May 28): Basic Agent Loop
- [ ] Agent trait → `GenericAgent` struct
- [ ] `think()` → calls LLM with task, gets thought
- [ ] `plan()` → calls LLM, parses JSON plan with steps
- [ ] `execute()` → calls tools, returns result
- [ ] `Plan` supports: sequential steps only (no parallel/HTG — YAGNI)

### Day 7 (May 29): CLI + First E2E
- [ ] `agent-cli` with `clap`: `agentic run "prompt"`, `agentic task list`, `agentic task status <id>`
- [ ] `AgentEngine` struct wiring everything together
- [ ] **Milestone:** `cargo run -- run "create file hello.txt"` works end-to-end

---

## Sprint 2: Verification + Memory (May 30 - June 5, 7 days)

### Day 8 (May 30): Gate 1 + Gate 2 — Structural + Deterministic
- [ ] `StructuralVerifier`: JSON schema check, required fields, regex
- [ ] `DeterministicVerifier`: exit code check, output bounds, tool actually ran
- [ ] `VerificationGate` trait
- [ ] `VerifierPipeline` — runs gates sequentially, fails fast

### Day 9 (May 31): Gate 3 — LLM Verifier
- [ ] `LLMVerifier` struct
- [ ] Critic prompt: "evaluate this output against these criteria"
- [ ] Structured output: `CriticEvaluation { passed, confidence, evidence, issues }`
- [ ] `ConfidenceScore` computation

### Day 10 (June 1): Gate 4 — Code Verifier
- [ ] `CodeVerifier`: syntax check via shell (language-specific)
- [ ] Rust support: `cargo check`, `cargo clippy`, `cargo test`
- [ ] Python support: `py_compile`, `ruff`, `pytest`
- [ ] JS support: `node --check`, `eslint`, `jest`
- [ ] Language config struct (extensible)

### Day 11 (June 2): Reflection Loop
- [ ] `ReflectionLoop` struct
- [ ] `reflect()` prompt: "analyze what went wrong, what to change"
- [ ] History injection into next attempt
- [ ] Max 3 retries, then escalate to human
- [ ] `RetryPolicy`: just ExponentialBackoff (KISS)
- [ ] `FailureTracker`: count consecutive same failures → escalate

### Day 12 (June 3): SQLite Memory
- [ ] `sqlx` with SQLite
- [ ] Tables: `task_history`, `task_logs`
- [ ] `SqliteMemoryStore`: `store()`, `retrieve()`, `recent()`
- [ ] Task persistence across restarts
- [ ] Checkpoint: save step results after each step

### Day 13 (June 4): Verification Integration + Human Escalation
- [ ] Wire verification pipeline into agent loop
- [ ] Wire reflection loop into agent loop
- [ ] Human escalation: print to CLI, wait for input
- [ ] Full integration test: agent writes broken code → verifier fails → retry → fix → pass

### Day 14 (June 5): Buffer + Bugfix
- [ ] Integration testing of full pipeline
- [ ] All error paths: tool failure, LLM timeout, parse failure
- [ ] Edge cases: empty task, very long output, special characters in shell
- [ ] **Milestone:** Agent can take a coding task, write code, verify it, retry if needed

---

## Sprint 3: Dioxus UI + Polish (June 6-12, 7 days)

### Day 15 (June 6): Dioxus Scaffold
- [ ] `dioxus-cli` setup
- [ ] `apps/desktop` — Dioxus desktop project
- [ ] Wire `AgentEngine` to Dioxus via channel (engine in same process)
- [ ] Basic window with title "Agentic"

### Day 16 (June 7): Task Input + List
- [ ] `TaskInput` component: text area + submit button
- [ ] `TaskList` component: shows tasks with status badges
- [ ] Colors: green=done, yellow=running, red=error, blue=pending
- [ ] Reactive state from `AgentEngine.subscribe()`

### Day 17 (June 8): Live Execution View
- [ ] `AgentLog` component: real-time scrolling log of events
- [ ] `TaskDetail` panel: shows plan, current step, outputs
- [ ] Auto-scroll to latest event

### Day 18 (June 9): Verification UI
- [ ] `VerdictDisplay`: gate-by-gate pass/fail with checkmarks/crosses
- [ ] `ConfidenceGauge`: simple progress bar (red → yellow → green)
- [ ] `ApprovalDialog`: modal with task info + approve/reject buttons

### Day 19 (June 10): Polish + UX
- [ ] Loading states (spinners, progress bars)
- [ ] Error states (toast notifications)
- [ ] Empty states ("no tasks yet" messages)
- [ ] Keyboard shortcuts (Ctrl+Enter to submit)
- [ ] Dark mode toggle (one CSS variable swap)

### Day 20 (June 11): Web Build
- [ ] `dx build --platform web`
- [ ] Fix WASM-specific issues (no `std::process::Command`, no filesystem)
- [ ] Web fallback: disable shell/filesystem tools, show "not available in web"

### Day 21 (June 12): Ship Day
- [ ] `README.md` with screenshots + demo GIF
- [ ] `cargo run` from clean clone works
- [ ] Release binaries for Windows + macOS + Linux
- [ ] Post to GitHub
- [ ] **Deal with whatever breaks, no new features**

---

## Non-Negotiable Rules

1. **No new crates** beyond: `agent-core`, `agent-llm`, `agent-tools`, `agent-verifier`, `agent-runtime`, `agent-cli`, `agent-api` (optional), `agent-ui` (shared components), `apps/desktop` (the actual app)

2. **No new dependencies without team vote.** Default is to use what we have. stdlib > crate.

3. **If it's not tested in CLI, it doesn't exist.** UI is decoration. The CLI must work first.

4. **If a feature isn't done by end of its day, cut it.** No slipping. Ship what works.

5. **One PR per day minimum.** Even if it's 10 lines. Momentum matters.

6. **No refactoring.** Ugly code that works > beautiful code that doesn't exist yet.

7. **Copy-paste from the architecture doc is encouraged.** Types are already designed. Just type them.

---

## Daily Vibe Check

```
May 23 ░░░░░░░░░░░░░░░░░░░░ [  0%] Scaffold
May 29 ▓▓▓▓░░░░░░░░░░░░░░░░ [ 33%] Core loop ships
Jun  5 ▓▓▓▓▓▓▓░░░░░░░░░░░░░ [ 66%] Verification ships
Jun 12 ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ [100%] UI ships
```

---

## What Success Looks Like on June 12

```bash
# CLI
agentic run "Write a Rust function that calculates fibonacci to fib.rs, verify it compiles"
# Output:
# ╭────────────────────────────────────╮
# │ Think: Write fibonacci function    │
# │ Plan: 3 steps                      │
# │ Execute: writing fib.rs...         │
# │ Verify:                            │
# │   ✓ Gate 1: Structure OK           │
# │   ✓ Gate 2: Deterministic OK       │
# │   ✓ Gate 3: LLM OK (conf 0.91)     │
# │   ✓ Gate 4: cargo check passed     │
# │ ✓ Task completed (0 retries)       │
# ╰────────────────────────────────────╯

# Desktop UI
# Window opens, submit task, see live execution in real-time
# Gates show green checkmarks, confidence bar fills up
# If agent fails → retry → watch it fix its own work

# Web
# Same UI runs in browser (no shell/filesystem tools)
```

**That's it. 3 tools, 4 verification gates, 1 agent, 1 reflection loop, 1 UI. Ship.**
