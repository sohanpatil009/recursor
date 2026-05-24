# Development Roadmap — Phase-by-Phase Guide

## Overview

This document provides a detailed, week-by-week development roadmap organized into 8 phases. Each phase builds on the previous one. The total estimated timeline is ~32 weeks for a production-ready MVP, with ongoing work for scalability and distribution.

**Key Principle:** Build the verification system early — it's the core differentiator between a toy and a production-grade agent platform.

---

## Phase 0 — Foundation (Weeks 1-4)

**Goal:** Get the skeleton in place with a working (but naive) single-agent single-tool loop.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W1 | Rust workspace, CI/CD, crate skeleton | Choose: SQLite vs PostgreSQL → SQLite for MVP |
| W2 | `agent-core` traits and types | Lock down Agent, Tool, Verifier, MemoryStore traits |
| W3 | `agent-llm` with OpenAI-compatible API | Support structured output via JSON schema from day 1 |
| W4 | CLI prototype: think → plan → execute (no verification) | Single agent, single tool (filesystem), no multi-agent |

### Week-by-Week Breakdown

#### Week 1: Workspace & Infrastructure
- [x] Initialize workspace: `cargo new agentic --workspace`
- [x] Create all crate directories with `Cargo.toml` stubs
- [x] Configure `rust-toolchain.toml` (stable + wasm target)
- [x] Set up CI/CD (GitHub Actions):
  - `cargo check` on all crates
  - `cargo test` for all crates
  - `cargo clippy` with deny warnings
  - `cargo fmt` check
- [x] Create `rustfmt.toml`, `clippy.toml`, `.editorconfig`
- [x] Set up `sccache` for faster local builds
- [x] Create `Makefile` or `justfile` for common commands

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
targets = ["wasm32-unknown-unknown"]
```

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo check --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --all --check
      - run: cargo test --workspace
```

#### Week 2: Core Types & Traits
- [ ] Define in `agent-core/src/lib.rs`:
  - `AgentId`, `TaskId`, `StepId`, `PlanId`, `ThoughtId`
  - `Task`, `Step`, `Plan`, `Thought`, `Context`
  - `StepResult`, `ToolOutput`, `Verdict`, `ConfidenceScore`
  - `Turn`, `Reflection`, `FinalOutput`
- [ ] Define all traits:
  - `Agent`, `Planner`, `Executor`, `Verifier`, `Critic`
  - `Tool`, `MemoryStore`, `LLMProvider`
- [ ] Define event types: `AgentEvent` enum
- [ ] Define error types: `AgentError`, `ToolError`, `VerificationError`, `MemoryError`
- [ ] Implement `Serialize`/`Deserialize` for all types (behind `serde` feature)

**Key constraint:** `agent-core` must have zero dependencies (or only `serde`). It should be `no_std`-compatible where possible.

```rust
// agent-core/src/lib.rs — entry point
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod agent;
mod error;
mod event;
mod memory;
mod task;
mod tool;
mod types;
mod verifier;

pub use agent::*;
pub use error::*;
pub use event::*;
pub use memory::*;
pub use task::*;
pub use tool::*;
pub use types::*;
pub use verifier::*;
```

#### Week 3: LLM Provider
- [ ] Define `LLMProvider` trait in `agent-llm`:
  - `async fn generate(&self, prompt: &str, options: GenerateOptions) -> Result<LLMResponse>`
  - `async fn generate_streaming(&self, prompt: &str, options: GenerateOptions) -> Result<Stream<Item = Token>>`
  - `async fn generate_structured<T: DeserializeOwned>(&self, prompt: &str) -> Result<T>`
  - `async fn count_tokens(&self, text: &str) -> Result<usize>`
- [ ] Implement `OpenAIProvider` (supports OpenAI, Anthropic, Groq, Together, etc.)
- [ ] Implement structured output parsing:
  - JSON schema validation on output
  - Retry on parse failure (up to 3 attempts)
  - Partial JSON recovery
- [ ] Add streaming support (SSE parsing via `reqwest_eventsource`)
- [ ] Add token counting and cost tracking

```rust
// provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, req: LLMRequest) -> Result<LLMResponse, LLMError>;
    async fn generate_streaming(&self, req: LLMRequest) -> Result<BoxStream<'static, LLMStreamEvent>, LLMError>;
    async fn generate_structured<T: DeserializeOwned>(&self, req: LLMRequest) -> Result<T, LLMError> {
        let response = self.generate(LLMRequest {
            response_format: Some(ResponseFormat::Json { schema: None }),
            ..req
        }).await?;
        serde_json::from_str(&response.content).map_err(|e| LLMError::ParseError(e.to_string()))
    }
}
```

#### Week 4: CLI Prototype
- [ ] Create `agent-cli` crate with `clap` CLI
- [ ] Implement simple agent loop (no verification):
  ```
  think(task) → thought
  plan(thought) → steps
  for each step:
    execute(step, tool) → result
  return result
  ```
- [ ] Implement `FilesystemTool` (read, write, list)
- [ ] Wire up: CLI → AgentEngine → Agent → Tool → output
- [ ] Test: CLI can "write a hello world Rust file"

```rust
// CLI entry point
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let llm = OpenAIProvider::new(&cli.api_key, "gpt-4o");
    let filesystem = FilesystemTool::new(vec![std::env::current_dir()?]);
    let mut registry = ToolRegistry::new(default_permissions());
    registry.register(Box::new(filesystem));

    let agent = SimpleAgent::new(llm, registry);
    let result = agent.run(Task::new(cli.prompt)).await?;

    println!("{}", result.output);
    Ok(())
}
```

### What to Avoid in Phase 0

- ❌ Multi-agent systems
- ❌ Verification/reflection
- ❌ UI of any kind
- ❌ Mobile/web targets
- ❌ Plugin systems
- ❌ Vector databases
- ❌ Browser automation
- ❌ Checkpoint/resume

**Validation check:** At the end of Phase 0, you should be able to run:
```bash
cargo run -- "Create a file called hello.txt with the text 'Hello, world!'"
```
And see it work.

---

## Phase 1 — Agent Loop (Weeks 5-8)

**Goal:** Complete the basic agent loop with multiple tools and task lifecycle management.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W5 | Think → Plan → Execute loop with step iteration | Plan structure with dependencies |
| W6 | `agent-tools`: Shell tool + HTTP tool + Search tool | Sandbox shell: block dangerous commands |
| W7 | Tool registry with permission checks | Define default deny-all permission model |
| W8 | Task lifecycle: queue, execute, cancel, pause | Use SQLite for task persistence |

### Week-by-Week Breakdown

#### Week 5: Agent Loop Completion
- [ ] Implement full think → plan → execute loop
- [ ] Plan supports step dependencies and parallel groups
- [ ] Agent maintains context across steps
- [ ] Task status tracking (pending → queued → in_progress → completed/failed)
- [ ] Error handling: tool failures don't crash the agent
- [ ] Unit tests with mocked LLM (return predefined plans/results)

#### Week 6: Tool Implementations
- [ ] `ShellTool`:
  - Command allowlist/blocklist (regex-based)
  - Timeout enforcement
  - Output size cap
  - Working directory restriction
- [ ] `HttpTool`:
  - GET, POST, PUT, DELETE
  - Configurable timeouts
  - Response size limits
  - Header injection protection
- [ ] `SearchTool` (web search):
  - Uses `reqwest` to call search API
  - Returns summarized results
- [ ] All tools implement `Tool` trait with JSON schema input specs
- [ ] Documentation: generate usage prompts for LLM context

```rust
// ShellTool config
pub struct ShellConfig {
    pub allowed_commands: Vec<Regex>,     // if empty, allow all
    pub blocked_commands: Vec<Regex>,     // e.g., "rm -rf /", "shutdown", "format"
    pub max_execution_time: Duration,     // default 30s
    pub max_output_size: usize,           // default 1MB
    pub allowed_workdirs: Vec<PathBuf>,   // restrict to project dirs
    pub requires_approval: Vec<Regex>,    // commands needing human OK
}
```

#### Week 7: Permission System
- [ ] `Permissions` struct with agent-level and role-level rules
- [ ] Default: deny-all, opt-in per tool
- [ ] Path glob filtering for filesystem tool
- [ ] Network policy (allowlist/blocklist) for HTTP tool
- [ ] Rate limiting per tool per agent
- [ ] `requires_human_approval` flag for dangerous operations
- [ ] Test: verify permission denial works correctly

#### Week 8: Task Lifecycle Management
- [ ] `TaskScheduler` with priority queue (SQLite-backed)
- [ ] Task lifecycle:
  - `Pending` → `Queued` → `InProgress` → `Completed`
  - `InProgress` → `Paused` (manual pause)
  - `InProgress` → `Failed` (error)
  - Any → `Cancelled`
- [ ] Task pause/resume
- [ ] Task cancellation with cleanup
- [ ] CLI commands for all lifecycle operations

### What to Avoid in Phase 1

- ❌ Verification/reflection (coming in Phase 2)
- ❌ Multi-agent
- ❌ UI
- ❌ Mobile/web
- ❌ Vector stores
- ❌ Browser automation

**Validation check:**
```bash
cargo run -- task submit "Search for Rust async frameworks and save results to frameworks.md"
cargo run -- task list
cargo run -- task status <id>
```

---

## Phase 2 — Verification (Weeks 9-12) — Core Differentiator

**Goal:** Build the self-verification and reflection system. This is what makes the platform production-grade.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W9 | Structural + Deterministic verification gates | Verifier trait, Gate trait |
| W10 | LLM-based verifier (critic agent) | Critic prompt template, confidence scoring |
| W11 | Reflection loop with retry + escalation | RetryPolicy, FailureTracker |
| W12 | Code verification (compile, lint, test) + full integration | Tool-based verification for code tasks |

### Week-by-Week Breakdown

#### Week 9: Gate 1 & 2 — Structural + Deterministic
- [ ] `StructuralVerifier`: JSON schema validation, required fields, regex patterns
- [ ] `DeterministicVerifier`: exit code checks, output bounds, tool call count, file existence
- [ ] `GateResult` type with pass/fail/skip + details
- [ ] `VerifierPipeline` that runs gates sequentially, short-circuiting on fail
- [ ] Integration: pipeline runs after every step execution

```rust
pub struct VerifierPipeline {
    gates: Vec<Box<dyn VerificationGate>>,
}

#[async_trait]
pub trait VerificationGate: Send + Sync {
    fn name(&self) -> &str;
    async fn verify(&self, result: &StepResult, step: &Step, context: &Context) -> Result<GateVerdict, VerificationError>;
}

pub enum GateVerdict {
    Pass,
    Fail { reason: String, details: Value },
    Skip { reason: String },
}
```

#### Week 10: Gate 3 — LLM-Based Verification
- [ ] `LLMVerifier` with critic agent prompt
- [ ] Structured output: `CriticEvaluation { passed, confidence, evidence, issues, suggestions }`
- [ ] Confidence scoring: combine structural + LLM + tool scores
- [ ] Confidence thresholds:
  - `>= 0.9`: auto-pass
  - `0.7 - 0.9`: pass with low confidence warning
  - `< 0.7`: fail → trigger reflection
  - `< 0.4`: escalate to human
- [ ] Evidence citation: verifier must cite specific parts of the output

#### Week 11: Reflection Loop + Retry
- [ ] `ReflectionLoop` orchestrator
- [ ] `Reflection` generation: what went wrong, what to change, what to keep
- [ ] `RetryPolicy`: Fixed, ExponentialBackoff, Adaptive, Escalate
- [ ] `FailureTracker`: pattern recognition, consecutive same-failure detection
- [ ] Human escalation: when confidence is too low or max retries exceeded
- [ ] History injection: previous reflections fed into next attempt's context

```rust
pub enum EscalationReason {
    MaxRetriesExceeded { attempts: usize, last_verdict: Verdict },
    ConfidenceTooLow { score: ConfidenceScore, threshold: f64 },
    RecurringFailure { pattern: String, count: usize },
    SecurityConcern { detail: String },
    HumanRequested,
}

pub struct Escalation {
    pub reason: EscalationReason,
    pub task: Task,
    pub history: Vec<Turn>,
    pub suggested_action: String,
}
```

#### Week 12: Gate 4 — Tool-Based Code Verification
- [ ] `CodeVerifier` tool:
  - Syntax check (language-specific parser call)
  - Type check (`cargo check`, `mypy`, `tsc --noEmit`)
  - Lint (`cargo clippy`, `ruff`, `eslint`)
  - Test execution (compile and run tests)
  - Format check (`rustfmt`, `prettier --check`)
  - Security audit (basic patterns: no `eval`, no `exec`, no hardcoded secrets)
- [ ] Language configuration system:
  ```rust
  pub struct LanguageConfig {
      pub name: String,
      pub file_extensions: Vec<String>,
      pub compile_command: Option<String>,
      pub test_command: Option<String>,
      pub lint_command: Option<String>,
      pub format_command: Option<String>,
      pub security_patterns: Vec<Regex>,
  }
  ```
- [ ] Integration: code verification runs after code-writing steps
- [ ] Full end-to-end test: agent writes code → all 5 gates pass → success

### Hallucination Mitigation (Implement in Phase 2)

| Strategy | Implementation | When to Apply |
|----------|---------------|---------------|
| Ground in tool output | Require tool call evidence for factual claims | All steps |
| Structured output | JSON schema enforcement on all LLM outputs | Always |
| Contrastive verification | Verifier checks conclusion vs. cited evidence | Gate 3 |
| Self-ask verification | Spawn verification task for specific claims | High-uncertainty steps |
| Uncertainty estimation | Use log probs from LLM (if available) | Gate 3 confidence |
| RAG grounding | Memory reads must cite source | Memory retrieval |

### What to Avoid in Phase 2

- ❌ UI
- ❌ Multi-agent coordination
- ❌ Mobile/web
- ❌ Browser automation
- ❌ Vector databases (simple SQLite is fine)

**Validation check:**
```bash
cargo run -- "Write a Rust function that reverses a string, save it to reverse.rs, and verify it compiles"
# Should show: Think → Plan → Execute → Gate1 ✓ → Gate2 ✓ → Gate3 ✓ → Gate4 ✓ → ReflectionLoop(0 retries) → Finalized
```

---

## Phase 3 — UI & Cross-Platform (Weeks 13-16)

**Goal:** Add a real UI with Dioxus. Start with desktop, then web.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W13 | Dioxus project setup, component library | Use Dioxus 0.6+, TailwindCSS via Dioxus |
| W14 | Task dashboard: submit, list, view details | Reactive event stream from engine |
| W15 | Live agent log + status visualization | Real-time updates via broadcast channel |
| W16 | Human-in-the-loop approval dialogs + web build | Approval flow: request → render → approve/reject |

### Architecture Decisions

```
UI (Dioxus)                 Engine (Rust)
    │                              │
    │  subscribe() ←───────────────│ ← AgentEvents
    │                              │
    │  dispatch(UserCommand) ──────→│ → orchestrator
    │                              │
    └──────────────────────────────┘
    Both in same process (desktop)
    or over WebSocket (web)
```

### Week-by-Week Breakdown

#### Week 13: UI Foundation
- [ ] Create `agent-ui` crate with Dioxus dependency
- [ ] Set up TailwindCSS styling (via `dioxus-tailwind` or manual CSS)
- [ ] Component hierarchy:
  ```
  App
  ├── Sidebar
  │   ├── SessionList
  │   └── AgentList
  ├── MainContent
  │   ├── TaskDashboard
  │   │   ├── TaskInput
  │   │   ├── TaskList
  │   │   └── TaskDetail
  │   ├── AgentPanel
  │   │   ├── AgentStatusCard
  │   │   └── AgentLog
  │   └── VerificationPanel
  │       ├── VerdictDisplay
  │       ├── ConfidenceGauge
  │       └── ApprovalDialog
  └── StatusBar
  ```
- [ ] Create `AppState` with reactive signals
- [ ] Subscribing to `AgentEngine` events

#### Week 14: Task Dashboard
- [ ] `TaskInput`: text area for submitting tasks with natural language
- [ ] `TaskList`: shows all tasks with status badges, progress bars
- [ ] `TaskDetail`: expanded view of a single task (plan, steps, results)
- [ ] Icons for task status: pending(○), in_progress(●), paused(⏸), completed(✓), failed(✗)
- [ ] Color coding: green(pass), yellow(pending), red(fail), blue(in_progress)

#### Week 15: Live Agent Visualization
- [ ] `AgentStatusCard`: agent name, role icon, current task, state (idle/busy)
- [ ] `AgentLog`: scrollable real-time log of events
- [ ] `VerdictDisplay`: shows gate-by-gate pass/fail with confidence bars
- [ ] `ConfidenceGauge`: radial gauge showing 0-100% confidence
- [ ] Smooth animations on state transitions (CSS transitions)

#### Week 16: Human Oversight + Cross-Platform
- [ ] `ApprovalDialog`:
  - Shows when agent requests human input
  - Displays task context, proposed action, risks
  - Approve / Reject with reason / Modify buttons
- [ ] Desktop build: `cargo run` in `apps/desktop`
- [ ] Web build: `dx build --platform web` in `apps/web`
- [ ] CI builds for both targets

### What to Avoid in Phase 3

- ❌ Mobile builds (Android/iOS) — too early, packaging complexity
- ❌ Plugin system
- ❌ Vector databases
- ❌ Distributed agents
- ❌ Performance optimization

**Validation check:**
```bash
# Desktop
cd apps/desktop && cargo run
# Submit a task via UI, watch it execute, see verification gates pass
```

---

## Phase 4 — Memory & Persistence (Weeks 17-20)

**Goal:** Persistent memory across sessions, vector search, checkpoint/resume.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W17 | SQLite schema + `agent-memory` crate | Working memory (LRU) + Long-term (SQLite) |
| W18 | Vector memory with `sqlite-vec` | Embedding generation via `fastembed` |
| W19 | Checkpoint/resume system | bincode serialization, crash recovery |
| W20 | Context management & memory integration | Three-tier retrieval, automatic summarization |

### Week-by-Week Breakdown

#### Week 17: Base Memory System
- [ ] SQLite schema:
  ```sql
  CREATE TABLE long_term_memory (
      key TEXT PRIMARY KEY,
      context BLOB NOT NULL,       -- bincode serialized
      importance INTEGER NOT NULL,
      created_at TIMESTAMP NOT NULL,
      accessed_at TIMESTAMP NOT NULL,
      access_count INTEGER DEFAULT 0
  );

  CREATE TABLE task_history (
      task_id TEXT PRIMARY KEY,
      task BLOB NOT NULL,
      result BLOB,
      verdict BLOB,
      attempts INTEGER DEFAULT 0,
      created_at TIMESTAMP NOT NULL,
      completed_at TIMESTAMP
  );

  CREATE TABLE agent_logs (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      agent_id TEXT NOT NULL,
      event_type TEXT NOT NULL,
      payload BLOB NOT NULL,
      timestamp TIMESTAMP NOT NULL
  );

  CREATE INDEX idx_agent_logs_agent ON agent_logs(agent_id);
  CREATE INDEX idx_agent_logs_time ON agent_logs(timestamp);
  ```
- [ ] `WorkingMemory`: in-memory LRU cache with configurable capacity
- [ ] `LongTermMemory`: SQLite-backed persistent store
- [ ] `MemoryManager`: routes queries to appropriate tier

#### Week 18: Vector Memory
- [ ] Integrate `fastembed` for embedding generation
- [ ] Integrate `sqlite-vec` for vector storage
- [ ] Create `VectorMemory`:
  ```rust
  pub struct VectorMemory {
      embedder: EmbeddingModel,
      db: sqlx::SqlitePool,
      dimension: usize,
  }

  impl VectorMemory {
      pub async fn store(&self, key: &str, context: Context) -> Result<(), MemoryError> {
          let text = context.to_embedding_text();
          let embedding = self.embedder.embed(&text).await?;
          // Store in sqlite-vec with FTS5 for keyword fallback
      }

      pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
          let query_embedding = self.embedder.embed(query).await?;
          // Cosine similarity search via sqlite-vec
      }
  }
  ```
- [ ] Hybrid search: semantic (vector) + keyword (FTS5) with weighted scoring
- [ ] Automatic memory pruning: archive entries older than configurable threshold

#### Week 19: Checkpoint/Resume
- [ ] Checkpoint table:
  ```sql
  CREATE TABLE checkpoints (
      task_id TEXT NOT NULL,
      step_id TEXT NOT NULL,
      state BLOB NOT NULL,        -- bincode serialized CheckpointState
      created_at TIMESTAMP NOT NULL,
      PRIMARY KEY (task_id, step_id)
  );

  CREATE TABLE completed_tasks (
      task_id TEXT PRIMARY KEY,
      result BLOB NOT NULL,
      completed_at TIMESTAMP NOT NULL
  );
  ```
- [ ] `Checkpointer` trait + implementation
- [ ] Auto-save after each step execution
- [ ] Crash recovery on startup: load uncompleted tasks, resume
- [ ] Checkpoint verification: deserialize + validate before resume
- [ ] GC for old checkpoints (delete after 7 days)

#### Week 20: Context Management
- [ ] Context window management: trim + summarize when too long
- [ ] Automatic memory retrieval: inject relevant memories into agent context
- [ ] Memory importance scoring: what to keep, what to archive
- [ ] Session persistence: save/restore entire session state
- [ ] Context compression: summarize long histories

### What to Avoid in Phase 4

- ❌ Distributed state
- ❌ Multi-node vector search
- ❌ Browser automation
- ❌ Plugin system

**Validation check:**
- Agent executes a task → session ends → restart → agent recalls previous context
- Agent can search for "that thing we discussed about authentication" and find it
- Kill the process mid-task → restart → task resumes from last checkpoint

---

## Phase 5 — Multi-Agent & Orchestration (Weeks 21-24)

**Goal:** Multiple agents collaborating on complex tasks.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W21 | Agent factory + lifecycle management | AgentPool with spawn/kill/healthcheck |
| W22 | Task decomposition + dependency graph | Planner splits task into sub-tasks |
| W23 | Multi-agent workflow execution | Parallel execution with synchronization |
| W24 | Supervisor agent + monitoring | Anomaly detection, health checks, recovery |

### Week-by-Week Breakdown

#### Week 21: Agent Factory
- [ ] `AgentFactory` trait:
  ```rust
  #[async_trait]
  pub trait AgentFactory: Send + Sync {
      fn role(&self) -> AgentRole;
      async fn spawn(&self, config: AgentConfig) -> Result<Box<dyn Agent>, AgentError>;
      async fn healthcheck(&self, agent: &Box<dyn Agent>) -> Result<HealthStatus, AgentError>;
  }
  ```
- [ ] `AgentPool` with bounded capacity
- [ ] Agent lifecycle: spawn → init → ready → busy → idle → kill
- [ ] Resource tracking per agent (tokens used, time spent, tool calls)
- [ ] Agent heartbeat monitoring

#### Week 22: Task Decomposition
- [ ] `PlannerAgent.decompose(task) → Vec<SubTask>`
- [ ] Dependency graph construction:
  ```rust
  pub struct DependencyGraph {
      pub tasks: Vec<TaskId>,
      pub edges: Vec<(TaskId, TaskId)>, // (depends_on, dependent)
  }
  ```
- [ ] Parallel group detection: independent sub-tasks run concurrently
- [ ] Estimated cost per sub-task (tokens + time)
- [ ] Serialization: plan can be saved and restored

#### Week 23: Workflow Execution
- [ ] `WorkflowExecutor`:
  - Topological sort of dependency graph
  - Parallel execution of independent sub-tasks
  - Barrier synchronization at merge points
  - Per-sub-task verification with independent verifier
- [ ] Sub-task result aggregation
- [ ] Partial failure handling: failed sub-task can be retried without restarting the whole workflow

#### Week 24: Supervisor Agent
- [ ] `SupervisorAgent`:
  - Monitors other agent health (heartbeat timeouts)
  - Detects anomaly patterns (infinite loops, spiraling costs)
  - Can kill and restart stuck agents
  - Can reallocate resources between agents
  - Escalates persistent failures to human
- [ ] Monitoring dashboard (UI updates for supervisor state)
- [ ] Automated recovery procedures

### What to Avoid in Phase 5

- ❌ Distributed agents across machines
- ❌ Browser automation
- ❌ Plugin system (WASM)

**Validation check:**
```bash
cargo run -- "Build a simple calculator: research patterns, implement in Rust, write tests, verify all pass"
# Should show: Planner → 4 sub-tasks → 2 parallel executors → all pass
```

---

## Phase 6 — Local LLM & Offline (Weeks 25-28)

**Goal:** Run entirely offline with local models.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W25 | `llama-cpp-2` integration | GGUF model loading, GPU support |
| W26 | Tiered model routing | Edge/Local/Server tiers with automatic selection |
| W27 | Model management + download | UI for browsing/downloading/switching models |
| W28 | Offline-first architecture + fallback | Lock remote APIs behind feature flag |

### Week-by-Week Breakdown

#### Week 25: Local Inference Engine
- [ ] Integrate `llama-cpp-2` crate
- [ ] `LocalLLMProvider` implementing `LLMProvider` trait:
  ```rust
  pub struct LocalLLMProvider {
      model: LlamaModel,
      context: LlamaContext,
      config: LocalLLMConfig,
  }

  pub struct LocalLLMConfig {
      pub model_path: PathBuf,
      pub n_gpu_layers: usize,    // 0 = CPU only
      pub context_size: usize,    // 2048, 4096, 8192
      pub batch_size: usize,      // 512
      pub threads: usize,         // CPU threads
      pub use_mmap: bool,
  }
  ```
- [ ] Streaming token generation
- [ ] Structured output support (grammar-based via `lm-format-enforcer`)
- [ ] GPU acceleration detection and configuration

#### Week 26: Tiered Model Routing
- [ ] `ModelRouter`: selects best model for each task:
  ```rust
  pub struct ModelRouter {
      edge: LocalLLMProvider,   // 1-3B params (quick tasks)
      local: LocalLLMProvider,  // 7-8B params (standard)
      server: LocalLLMProvider, // 13B+ params (complex, GPU)
      remote: Option<OpenAIProvider>, // fallback
  }

  impl ModelRouter {
      pub async fn select(&self, task: &Task, context: &Context) -> &dyn LLMProvider {
          match task.complexity() {
              Complexity::Simple => &self.edge,
              Complexity::Moderate => &self.local,
              Complexity::Complex => {
                  // Check GPU availability
                  if self.server.is_available() { &self.server }
                  else { &self.local } // fallback
              }
              Complexity::VeryComplex => {
                  self.remote.as_ref().unwrap_or(&self.server)
              }
          }
      }
  }
  ```
- [ ] Task complexity heuristics:
  - Token count of prompt
  - Number of steps in plan
  - Tool variety required
  - Verification gate count
- [ ] Automatic fallback: local → remote if local fails

#### Week 27: Model Management
- [ ] Model downloader (HTTP client with resume support)
- [ ] Model registry: HuggingFace integration for GGUF models
- [ ] Local model cache management (disk space tracking)
- [ ] UI for model management:
  - Browse available models (by size, quality, language)
  - Download progress indicator
  - Model switching
  - Performance benchmarks (tokens/sec, memory usage)
- [ ] Model quantization selection (Q4_K_M, Q5_K_M, Q8_0)

#### Week 28: Offline-First Architecture
- [ ] Feature gate for remote APIs: `cfg(feature = "remote-llm")`
- [ ] Default: all providers local, no network required
- [ ] Network detection: graceful degradation when offline
- [ ] Caching layer: cached remote responses survive offline mode
- [ ] Offline CI testing: full test suite runs without network
- [ ] Documentation: "Running fully offline" guide

### What to Avoid in Phase 6

- ❌ Distributed agents
- ❌ Plugin system
- ❌ Mobile builds

**Validation check:**
```bash
# Disconnect network
cargo run -- "Write a Fibonacci function in Rust"
# Should work entirely offline with local model
```

---

## Phase 7 — Production Hardening (Weeks 29-32)

**Goal:** Make the system production-ready: observability, security, performance.

### Milestones

| Week | Deliverable | Key Decisions |
|------|-------------|---------------|
| W29 | Observability: tracing, metrics, OpenTelemetry | Structured logging with correlation IDs |
| W30 | Sandboxing: Docker containers + process isolation | Container per task for untrusted code |
| W31 | WASM plugin system | Plugin SDK, manifest, sandboxed execution |
| W32 | Security audit + performance optimization | Load testing, profiling, fuzzing |

### Week-by-Week Breakdown

#### Week 29: Observability
- [ ] `tracing` setup with OpenTelemetry exporter:
  ```rust
  fn init_telemetry() -> Result<()> {
      let tracer = opentelemetry_otlp::new_pipeline()
          .tracing()
          .with_exporter(opentelemetry_otlp::new_exporter().tonic())
          .install_simple()?;

      let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
      let subscriber = tracing_subscriber::Registry::default()
          .with(telemetry)
          .with(tracing_subscriber::fmt::layer().json())
          .with(EnvFilter::from_default_env());

      tracing::subscriber::set_global_default(subscriber)?;
      Ok(())
  }
  ```
- [ ] Correlation IDs: every task gets a trace ID propagated to all spans
- [ ] Metrics: Prometheus export via `metrics-exporter-prometheus`:
  - `agent_tasks_total{status="completed|failed|cancelled"}`
  - `agent_task_duration_seconds{task_type="..."}`
  - `agent_verification_pass_rate{gate="1|2|3|4|5"}`
  - `agent_retry_count{failure_type="..."}`
  - `agent_llm_tokens_total{model="..."}`
  - `agent_tool_call_duration{tool="..."}`
  - `agent_memory_hits{type="working|longterm|vector"}`
  - `agent_pool_utilization`
- [ ] Health check endpoint (`GET /health`)
- [ ] Structured logging to file with rotation (`tracing-appender`)

#### Week 30: Sandboxing
- [ ] `SandboxManager` with tiered isolation:
  ```rust
  pub enum SandboxLevel {
      L0, // In-process (read-only tools)
      L1, // Process-level (restricted user, job object)
      L2, // Container (Docker, per-task container)
      L3, // MicroVM (Firecracker, maximum isolation)
  }
  ```
- [ ] Docker integration via `bollard`:
  - Per-task ephemeral containers
  - Resource limits (CPU, memory, disk)
  - Network policy (default: no network)
  - Auto-cleanup after task completion
- [ ] Process-level sandbox (L1):
  - Windows: Job Object + restricted token
  - Linux: cgroups + namespaces + seccomp
  - macOS: sandbox-exec
- [ ] Configuration: per-agent sandbox level

#### Week 31: Plugin System
- [ ] Plugin SDK crate (`agent-plugin-sdk`):
  ```rust
  // In plugin SDK
  #[no_mangle]
  pub extern "C" fn plugin_manifest() -> PluginManifest;

  #[no_mangle]
  pub extern "C" fn plugin_execute(tool_name: *const c_char, params: *const c_char) -> *const c_char;
  ```
- [ ] WASM-backed plugins via `wasmtime`:
  - Safe sandbox (no WASI file/net unless granted)
  - Manifest declares required permissions
  - Engine validates permissions at load time
  - Plugin hot-reload (swap WASM at runtime)
- [ ] Plugin registry: store/load from filesystem
- [ ] Example plugin: Jira ticket creator, Slack notifier

#### Week 32: Security + Performance
- [ ] Security audit:
  - Prompt injection testing
  - Tool permission boundary testing
  - Path traversal in filesystem tool
  - Command injection in shell tool
  - SSRF protection in HTTP tool
  - Secrets leak detection (regex patterns for API keys, passwords)
- [ ] Performance optimization:
  - Profile with `perf`/`flamegraph` on Linux, `Superflare` on Windows
  - Optimize hot paths: event serialization, memory retrieval, LLM calls
  - Connection pooling for SQLite and HTTP
  - LLM response caching (semantic cache)
  - Batch embedding generation
- [ ] Load testing:
  - 10 concurrent tasks → measure throughput
  - 100 concurrent tasks → find bottlenecks
  - 1000+ concurrent → identify scaling limits
- [ ] Fuzzing: `cargo fuzz` for tool input parsers

### What to Avoid in Phase 7

- ❌ Distributed agents (Phase 8)
- ❌ Mobile builds

**Validation check:**
```bash
# Load test
cargo run --bench throughput -- --concurrent 10 --tasks 50
# Security test
cargo run -- "rm -rf /"  # Should be blocked
```

---

## Phase 8 — Scalability & Distribution (Ongoing)

**Goal:** Scale beyond a single machine. Support distributed agent workers.

### Milestones

| Item | Deliverable | Timeline |
|------|-------------|----------|
| A | Distributed task queue (NATS / Kafka) | Post-Phase 7 |
| B | Remote agent workers | Post-A |
| C | Shared state across machines | Post-B |
| D | SaaS deployment (WebSocket API) | Post-C |
| E | Third-party integration API | Post-D |

### Architecture for Distribution

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Controller  │    │  Worker 1    │    │  Worker 2    │
│  Node        │    │  (Linux)     │    │  (Windows)   │
│              │    │              │    │              │
│ • Scheduler  │───→│ • Agent Pool │    │ • Agent Pool │
│ • State      │    │ • Tools      │    │ • Tools      │
│ • API server │    │ • Local LLM  │    │ • Local LLM  │
│ • Event bus  │    │              │    │              │
└──────┬───────┘    └──────────────┘    └──────────────┘
       │                                        ▲
       │    ┌──────────────┐                    │
       └───→│  NATS Queue   │───────────────────┘
            │  / Task Bus   │
            └──────────────┘
```

### When to Start Each Distributable Component

| Component | Trigger to Build | Alternative |
|-----------|-----------------|-------------|
| NATS/Kafka | Single node can't handle task volume | Stay with SQLite queue |
| Remote workers | Need heterogeneous hardware (GPU) | Use local only |
| Shared state | Workers need to coordinate | Accept each worker has own memory |
| SaaS API | External clients want to connect | Run local CLI/UI only |
| 3rd-party API | Developers want to extend | Stay with WASM plugins |

---

## Success Criteria by Phase

### Phase 0
```
cargo run -- "create hello.txt with content Hello World"
→ File exists with correct content
```

### Phase 1
```
cargo run -- "search for Rust HTTP frameworks and save to a file"  
→ File exists with search results
cargo run -- task list
→ Shows lifecycle state
```

### Phase 2
```
cargo run -- "write a function that calculates fibonacci, save to fib.rs, verify it compiles"
→ 5 gates pass, output verified
→ If agent writes broken code → retry → fix → pass
```

### Phase 3
```
# Desktop app opens
# User types task, sees execution in real-time
# Approval dialog appears for dangerous operations
```

### Phase 4
```
# Agent remembers previous conversation
# Task can be interrupted and resumed
# Search memory: "what did we build yesterday?"
```

### Phase 5
```
# Planner decomposes: "build a full CRUD API"
# Multiple executor agents run in parallel
# All pass verification independently
```

### Phase 6
```
# No internet connection
# All tasks execute using local model
# Same capabilities as Phase 2-5
```

### Phase 7
```
# Prometheus metrics available
# Docker sandbox runs untrusted code safely
# Plugin loads and executes
# 1000 concurrent tasks stable
```

---

## Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| LLM hallucinations break verification | High | High | Multi-gate pipeline, tool-based verification as safety net |
| Local models are too slow/small for complex tasks | Medium | High | Tiered routing, remote fallback, model caching |
| Permission system is bypassed | Low | Critical | Defense-in-depth: process sandbox + Docker + WASM |
| Build times slow down iteration | High | Medium | sccache, workspace optimization, incremental compilation |
| Cross-platform compatibility issues | Medium | Medium | CI matrix on all targets, WASM is the fallback |
| Async complexity causes subtle bugs | Medium | High | Extensive testing, chaos engineering, structured concurrency |
| SQLite doesn't scale for memory needs | Medium | Medium | Design MemoryStore trait to swap in pgvector/postgres later |

---

## Key Dates Summary

| Phase | Duration | Start | End | Core Deliverable |
|-------|----------|-------|-----|------------------|
| 0 | 4 weeks | W1 | W4 | Workspace + skeleton + basic agent |
| 1 | 4 weeks | W5 | W8 | Multi-tool agent + task lifecycle |
| 2 | 4 weeks | W9 | W12 | Self-verification + reflection |
| 3 | 4 weeks | W13 | W16 | Dioxus UI + human oversight |
| 4 | 4 weeks | W17 | W20 | Memory + checkpoint + persistence |
| 5 | 4 weeks | W21 | W24 | Multi-agent orchestration |
| 6 | 4 weeks | W25 | W28 | Local LLM + offline-first |
| 7 | 4 weeks | W29 | W32 | Production hardening |
| 8 | Ongoing | W33+ | — | Scalability + distribution |

**Total to production-ready MVP:** ~32 weeks (8 months)
**Total to enterprise-ready:** ~48 weeks (12 months)
