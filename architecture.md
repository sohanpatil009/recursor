# Architecture Document

## Table of Contents

1. [System Architecture Overview](#1-system-architecture-overview)
2. [Crate Architecture & Dependency Graph](#2-crate-architecture--dependency-graph)
3. [Core Agent Architecture](#3-core-agent-architecture)
4. [Event Bus System](#4-event-bus-system)
5. [Task Orchestration](#5-task-orchestration)
6. [Self-Verification & Reflection System](#6-self-verification--reflection-system)
7. [Memory & Persistence Architecture](#7-memory--persistence-architecture)
8. [Tool System Architecture](#8-tool-system-architecture)
9. [Cross-Platform Architecture](#9-cross-platform-architecture)
10. [Concurrency Model](#10-concurrency-model)
11. [Security Architecture](#11-security-architecture)
12. [Plugin Architecture](#12-plugin-architecture)
13. [Data Flow Diagrams](#13-data-flow-diagrams)

---

## 1. System Architecture Overview

Agentic is built as a layered, modular system with strict dependency ordering. The architecture follows the **ports and adapters** pattern: the core is pure business logic with zero dependencies, and all side effects are injected through traits.

```
┌──────────────────────────────────────────────────────────┐
│                    Application Layer                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Desktop  │  │   Web    │  │  Mobile  │  │   CLI    │  │
│  │  (Dioxus)│  │  (Dioxus)│  │  (Dioxus)│  │          │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       └──────────────┴──────────────┴──────────────┘      │
│                          │                                  │
├──────────────────────────┴──────────────────────────────────┤
│                    API Layer                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  agent-api (REST + WebSocket + JSON-RPC)             │  │
│  │  • Task submission/management endpoints               │  │
│  │  • Event streaming (SSE / WebSocket)                  │  │
│  │  • Agent lifecycle control                            │  │
│  │  • Human-in-the-loop approval                         │  │
│  └──────────────────────────┬───────────────────────────┘  │
├─────────────────────────────┴──────────────────────────────┤
│                     Runtime Layer                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  agent-runtime                                        │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │  │
│  │  │ Orchestrator│  │  Scheduler   │  │   Worker    │  │  │
│  │  │             │  │  (priority   │  │    Pool     │  │  │
│  │  │ Multi-agent │  │   queue)     │  │  (bounded   │  │  │
│  │  │ coordination│  │              │  │  semaphore) │  │  │
│  │  └─────────────┘  └──────────────┘  └─────────────┘  │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │  │
│  │  │ Sandbox     │  │ Checkpointer │  │  Event Bus  │  │  │
│  │  │ Manager     │  │ (resume/crash│  │  (broadcast)│  │  │
│  │  │             │  │  recovery)   │  │             │  │  │
│  │  └─────────────┘  └──────────────┘  └─────────────┘  │  │
│  └──────────────────────────┬───────────────────────────┘  │
├─────────────────────────────┴──────────────────────────────┤
│                   Agent Layer                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ Planner  │ │ Executor │ │ Verifier │ │  Critic  │      │
│  │ Agent    │ │ Agent    │ │ Agent    │ │ Agent    │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  agent-verifier: ReflectionLoop, ConfidenceScoring    │  │
│  │  agent-memory: WorkingMemory, LongTermMemory, VectorDB│  │
│  └──────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────┤
│                   Tool Layer                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │FileSystem│ │  Shell   │ │ Browser  │ │   HTTP   │      │
│  │          │ │          │ │          │ │          │      │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤     │
│  │  Search  │ │  Code    │ │ Python   │ │  Custom  │      │
│  │          │ │ (lint/   │ │ REPL     │ │ Plugins  │      │
│  │          │ │ test/cmp)│ │          │ │ (WASM)   │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │
├────────────────────────────────────────────────────────────┤
│                   LLM Layer                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  agent-llm                                            │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │  │
│  │  │  Provider   │  │  Structured  │  │  Streaming  │  │  │
│  │  │  Trait      │  │  Output      │  │  Handler    │  │  │
│  │  ├─────────────┤  │  (lm-format- │  ├─────────────┤  │  │
│  │  │ OpenAI-     │  │   enforcer,  │  │  Token      │  │  │
│  │  │ Compatible  │  │   JSON       │  │  Accounting  │  │  │
│  │  │ llama-cpp-2 │  │   Schema)    │  │             │  │  │
│  │  │ candle      │  │              │  │             │  │  │
│  │  └─────────────┘  └──────────────┘  └─────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────┤
│                  Core Layer (Zero Dependencies)              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  agent-core: Traits, Types, Errors                    │  │
│  │  • Agent trait, Verifier trait, Tool trait             │  │
│  │  • Task, Step, Plan, Thought, Verdict types            │  │
│  │  • AgentEvent enum, Error types                        │  │
│  │  • No_std compatible where possible                    │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

---

## 2. Crate Architecture & Dependency Graph

### Strict Dependency Direction

```
agent-core
  ↑        ↑
  |        |
agent-llm  agent-memory
  ↑        ↑
  |        |
agent-tools
  ↑
  |
agent-verifier
  ↑
  |
agent-runtime
  ↑
  |
agent-api
  ↑
  |
agent-ui
```

**Rules:**
- `agent-core` depends on **nothing** (zero external deps, `no_std` compatible types)
- A crate can only depend on crates below it in the stack
- No circular dependencies allowed (enforced by cargo)
- `agent-runtime` is the only crate that composes agents, tools, memory, and verifiers together

### Crate Responsibilities

#### agent-core
- All trait definitions: `Agent`, `Planner`, `Executor`, `Verifier`, `Critic`, `Tool`, `MemoryStore`, `LLMProvider`
- All type definitions: `Task`, `Step`, `Plan`, `Thought`, `Verdict`, `ConfidenceScore`, `Turn`, `Reflection`
- Event types: `AgentEvent`, `UserCommand`
- Error types: `AgentError`, `ToolError`, `VerificationError`
- No I/O, no async (synchronous pure logic)
- `Serialize`/`Deserialize` on all types (via serde, the only dependency)

#### agent-llm
- `LLMProvider` trait implementation
- OpenAI-compatible API client (reqwest-based)
- Local inference via `llama-cpp-2` and `candle`
- Structured output parsing (JSON schema validation, grammar-constrained generation)
- Streaming response handling (SSE parsing, `tokio_stream::Stream`)
- Token counting and cost tracking
- Model routing (edge/local/server tier selection)
- Fallback logic (local → remote → error)

#### agent-memory
- `MemoryStore` trait implementation
- `WorkingMemory` — In-memory LRU cache for current session context
- `LongTermMemory` — SQLite-backed persistent storage
- `VectorMemory` — Embedding-based semantic search (`sqlite-vec` or `pgvector`)
- `EpisodicMemory` — Task history, agent action logs
- `ProceduralMemory` — Learned workflows, tool usage patterns

#### agent-tools
- `Tool` trait implementation for each tool
- `ToolRegistry` — Central registry with permission checks
- `ToolPool` — Per-tool rate limiting and concurrency control
- Tool categories: filesystem, shell, browser, code, search, HTTP, data

#### agent-verifier
- `Verifier` trait implementation
- `DeterministicVerifier` — Schema validation, exit code checks, regex matching
- `LLMVerifier` — Critic agent evaluation with chain-of-thought
- `ToolVerifier` — Code compilation, test execution, linting
- `SelfConsistencyVerifier` — N-sample comparison with majority voting
- `ReflectionLoop` — Orchestrates retry cycles with escalating confidence thresholds

#### agent-runtime
- `Orchestrator` — Multi-agent coordination, workflow execution
- `TaskScheduler` — Priority queue, task lifecycle management
- `WorkerPool` — Bounded semaphore-controlled agent pool
- `SandboxManager` — Tiered sandbox execution (process/Docker/Firecracker)
- `Checkpointer` — State serialization and crash recovery
- `AgentEngine` — Top-level facade over the entire runtime
- `EventBus` — Broadcast channel fanning out to UI, logging, metrics

#### agent-api
- `axum`-based REST API
- `tokio-tungstenite` WebSocket endpoint for real-time event streaming
- JSON-RPC for agent commands
- Authentication and authorization middleware

#### agent-ui
- Shared Dioxus UI components
- Reactive dashboard components: TaskList, AgentStatusPanel, LiveLog, ApprovalDialog
- Platform-agnostic (same components rendered on desktop/web/mobile)

---

## 3. Core Agent Architecture

### Agent Trait — Full Definition

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Unique identifier for an agent instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// The role an agent plays in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// Analyzes tasks and produces execution plans
    Planner,
    /// Executes individual steps of a plan
    Executor,
    /// Verifies execution results against criteria
    Verifier,
    /// High-level review of complete workflows
    Critic,
    /// Supervisor — monitors and coordinates other agents
    Supervisor,
    /// Custom user-defined role
    Custom(String),
}

/// Core agent trait that all agents implement.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the unique identifier for this agent instance.
    fn id(&self) -> &AgentId;

    /// Returns the role of this agent.
    fn role(&self) -> AgentRole;

    /// Analyzes a task and generates initial thoughts/reasoning.
    async fn think(&self, task: &Task, context: &Context) -> Result<Thought, AgentError>;

    /// Converts thoughts into a structured execution plan.
    async fn plan(&self, thought: &Thought) -> Result<Plan, AgentError>;

    /// Executes a single step using available tools.
    async fn execute(&self, step: &Step, tools: &ToolRegistry) -> Result<StepResult, AgentError>;

    /// Verifies a step result against a set of criteria.
    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, AgentError>;

    /// Reflects on previous attempts to improve future execution.
    async fn reflect(&self, history: &[Turn]) -> Result<Reflection, AgentError>;
}
```

### Agent Lifecycle

```
                    ┌──────────┐
                    │  IDLE    │
                    └────┬─────┘
                         │ task assigned
                         ▼
                    ┌──────────┐
              ┌─────│  THINK   │─────┐
              │     └──────────┘     │
              │          │           │
              │     ┌────▼──────┐    │
              │     │  thought  │    │ (iteration)
              │     └────┬──────┘    │
              │          ▼           │
              │     ┌──────────┐     │
              └─────│  PLAN    │─────┘
                    └────┬─────┘
                         │ plan
                         ▼
                    ┌──────────┐
              ┌─────│ EXECUTE  │─────┐
              │     └────┬─────┘     │
              │          │           │
              │     ┌────▼──────┐    │
              │     │ step result│   │ (retry)
              │     └────┬──────┘    │
              │          ▼           │
              │     ┌──────────┐     │
              └─────│ VERIFY   │─────┘
                    └────┬─────┘
                         │ verdict
                    ┌────▼─────┐
                    │ verdict  │
                    │  check   │
                    └────┬─────┘
               ┌─────────┴──────────┐
               ▼                    ▼
          ┌──────────┐        ┌──────────┐
          │  PASS    │        │  FAIL    │
          └────┬─────┘        └────┬─────┘
               │                   │
          ┌────▼─────┐       ┌─────▼──────┐
          │ Finalize │       │ retries    │
          │  output  │       │ remaining? │
          └──────────┘       └─────┬──────┘
                          ┌────────┴────────┐
                          ▼                 ▼
                    ┌──────────┐      ┌──────────┐
                    │  REFLECT │      │ESCALATE  │
                    │  & RETRY │      │ to human │
                    └──────────┘      └──────────┘
```

### Agent Specializations

#### Planner Agent
- Input: High-level task description + context
- Output: Structured `Plan` with ordered/parallel steps
- Uses chain-of-thought with task decomposition
- Produces dependency graphs between steps
- Assigns success criteria per step

```rust
pub struct PlannerAgent {
    id: AgentId,
    llm: Arc<dyn LLMProvider>,
    max_steps: usize,
}

impl PlannerAgent {
    /// Decomposes a task into dependent sub-tasks.
    pub async fn decompose(&self, task: &Task) -> Result<Vec<SubTask>, AgentError> {
        // Identifies parallelizable work, sequential dependencies,
        // and verification checkpoints between steps
    }

    /// Assigns success criteria to each step.
    pub async fn assign_criteria(&self, plan: &mut Plan) -> Result<(), AgentError> {
        // Each step gets structured criteria that verifier can check
    }
}
```

#### Executor Agent
- Input: Single `Step` + context + retry history
- Output: `StepResult` with evidence
- Selects and invokes tools from the `ToolRegistry`
- Captures full tool output as evidence for verifier

```rust
pub struct ExecutorAgent {
    id: AgentId,
    llm: Arc<dyn LLMProvider>,
    max_tool_calls_per_step: usize,
    context_window_size: usize,
}

impl ExecutorAgent {
    /// Executes a step using available tools with reflection context.
    pub async fn execute_with_reflection(
        &self,
        step: &Step,
        tools: &ToolRegistry,
        previous_attempts: &[Turn],
    ) -> Result<StepResult, AgentError> {
        // Injects previous failure reflections into system prompt
        // Chooses tool sequence based on step requirements
        // Captures all tool I/O as evidence
    }
}
```

#### Verifier Agent
- Input: `StepResult` + `Criterion[]` + evidence
- Output: `Verdict` with `ConfidenceScore`
- Multi-gate verification pipeline

```rust
pub struct VerifierAgent {
    id: AgentId,
    llm: Arc<dyn LLMProvider>,
    confidence_threshold: f64,
    enable_self_consistency: bool,
    consistency_samples: usize,
}
```

---

## 4. Event Bus System

### Design

The event bus is a `tokio::sync::broadcast` channel that acts as the central nervous system of the platform. Every agent action, tool call, and state transition emits a typed event.

```rust
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

/// Capacity of the broadcast channel (number of events buffered).
const EVENT_BUS_CAPACITY: usize = 10_000;

/// Central event bus for the agent system.
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self { sender }
    }

    /// Subscribe to all agent events.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: AgentEvent) -> Result<(), BusError> {
        self.sender.send(event)?;
        Ok(())
    }
}
```

### Event Taxonomy

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    // ── Task Lifecycle ──
    TaskCreated(TaskId, Task),
    TaskEnqueued(TaskId, Priority),
    TaskStarted(TaskId),
    TaskPaused(TaskId, PauseReason),
    TaskCancelled(TaskId, CancelReason),
    TaskCompleted(TaskId, FinalOutput),
    TaskFailed(TaskId, AgentError),

    // ── Agent Lifecycle ──
    AgentSpawned(AgentId, AgentRole, TaskId),
    AgentBusy(AgentId),
    AgentIdle(AgentId),
    AgentDied(AgentId, AgentError),

    // ── Agent Reasoning ──
    ThoughtStarted(AgentId, ThoughtId),
    ThoughtComplete(AgentId, Thought),
    PlanCreated(AgentId, Plan),

    // ── Step Execution ──
    StepStarted(AgentId, StepId),
    StepProgress(AgentId, StepId, Progress),
    StepPaused(AgentId, StepId, PauseReason),
    StepCompleted(AgentId, StepId, StepResult),
    StepFailed(AgentId, StepId, AgentError),
    StepSkipped(AgentId, StepId, SkipReason),

    // ── Tool Execution ──
    ToolCalled(AgentId, ToolCall),
    ToolResult(AgentId, ToolCallId, ToolResult),
    ToolError(AgentId, ToolCallId, ToolError),
    ToolBlocked(AgentId, ToolCall, BlockReason),

    // ── Verification ──
    VerificationStarted(AgentId, StepId),
    VerificationGatePassed(AgentId, GateKind),
    VerificationGateFailed(AgentId, GateKind, FailureDetail),
    VerdictReached(AgentId, Verdict),
    ConfidenceLow(AgentId, ConfidenceScore, Threshold),

    // ── Retry / Reflection ──
    RetryScheduled(AgentId, usize, RetryPolicy, Duration),
    ReflectionGenerated(AgentId, Reflection),
    EscalationTriggered(AgentId, EscalationReason),

    // ── Memory ──
    MemoryRead(AgentId, MemoryQuery),
    MemoryWritten(AgentId, MemoryEntry),
    MemoryCacheHit(AgentId),
    MemoryCacheMiss(AgentId),

    // ── Human Oversight ──
    HumanApprovalRequested(AgentId, ApprovalRequest),
    HumanApprovalGranted(AgentId, ApprovalId),
    HumanApprovalDenied(AgentId, ApprovalId, String),
    HumanFeedbackReceived(AgentId, Feedback),

    // ── System ──
    SystemStart,
    SystemShutdown,
    SystemError(AgentError),
}
```

### Subscribers

| Subscriber | Purpose | Lag Tolerance |
|------------|---------|---------------|
| UI (Dioxus) | Real-time dashboard updates | Low |
| Logger (tracing) | Structured event log | Medium |
| Metrics (opentelemetry) | Prometheus metrics | Low |
| Checkpointer | State persistence triggers | Medium |
| Supervisor Agent | Monitors for anomalies | Low |
| WebSocket API | External client streaming | Low |

---

## 5. Task Orchestration

### Task Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub status: TaskStatus,
    pub max_retries: usize,
    pub timeout: Duration,
    pub criteria: Vec<Criterion>,
    pub context: Context,
    pub created_at: DateTime<Utc>,
    pub assigned_agents: Vec<AgentId>,
    pub parent_task: Option<TaskId>,
    pub subtasks: Vec<TaskId>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Queued,
    InProgress,
    Paused(PauseReason),
    AwaitingApproval(ApprovalRequest),
    Completed(FinalOutput),
    Failed(AgentError),
    Cancelled(CancelReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}
```

### Plan Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub task_id: TaskId,
    pub steps: Vec<Step>,
    pub dependency_graph: DependencyGraph,
    pub estimated_total_cost: TokenCost,
    pub created_by: AgentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub index: usize,
    pub description: String,
    pub tool_requirements: Vec<ToolRequirement>,
    pub criteria: Vec<Criterion>,
    pub max_retries: usize,
    pub timeout: Duration,
    pub dependencies: Vec<StepId>,
    pub parallel_group: Option<GroupId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub edges: Vec<(StepId, StepId)>, // (depends_on, dependent)
}
```

### Hierarchical Task Graph (HTG) Execution

```
Task: "Build a REST API endpoint for user registration"
│
├── [Plan]
│   ├── Step 1: Research framework & patterns
│   │   └── Subtask 1.1: Check existing codebase structure
│   │   └── Subtask 1.2: Read API documentation
│   │   └── Verifier: Confirm understanding is correct
│   │
│   ├── Step 2: Implement the endpoint (parallel group A)
│   │   ├── Step 2a: Write route handler
│   │   ├── Step 2b: Write input validation
│   │   └── Step 2c: Write database interaction
│   │
│   ├── Step 3: Verify implementation
│   │   ├── [Gate 1] Syntax check
│   │   ├── [Gate 2] Type check
│   │   ├── [Gate 3] Compile check
│   │   └── [Gate 4] Security audit
│   │
│   └── Step 4: Write tests (parallel group B)
│       ├── Step 4a: Write unit tests
│       └── Step 4b: Write integration tests
│
└── [Final Verification]
    ├── Run all tests
    ├── Lint check
    └── Critic agent review
```

### Execution Engine

```rust
pub struct TaskScheduler {
    queue: PriorityQueue<Task>,
    active_tasks: HashMap<TaskId, TaskHandle>,
    config: SchedulerConfig,
    checkpointer: Arc<Checkpointer>,
}

impl TaskScheduler {
    /// Enqueue a task for execution.
    pub async fn enqueue(&mut self, task: Task) -> Result<(), SchedulerError>;

    /// Dequeue and start executing the highest priority task.
    pub async fn process_next(&mut self) -> Result<(), SchedulerError>;

    /// Execute a single step with checkpointing.
    pub async fn execute_step_with_checkpoint(
        &self,
        task: &Task,
        step: &Step,
        agent: &ExecutorAgent,
    ) -> Result<StepResult, SchedulerError> {
        // 1. Load checkpoint (if exists)
        let checkpoint = self.checkpointer.load(&task.id, &step.id).await;

        // 2. Execute step (skip if already completed)
        let result = if checkpoint.is_some() {
            checkpoint.unwrap().result
        } else {
            let result = agent.execute(step, &self.tool_registry).await?;
            self.checkpointer.save(&task.id, &step.id, &result).await?;
            result
        };

        // 3. Verify
        let verifier = self.verifier_pool.acquire().await;
        let verdict = verifier.verify(&result, &step.criteria).await?;

        Ok(result)
    }
}
```

---

## 6. Self-Verification & Reflection System

### Architecture Overview

```
                     ┌────────────────────────────┐
                     │    Verification Pipeline    │
                     │  (executed sequentially)    │
                     └────────────────────────────┘
                                │
           ┌────────────────────┼────────────────────┐
           ▼                    ▼                    ▼
   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
   │ Deterministic │   │   LLM-Based   │   │  Tool-Based   │
   │   Verifier    │   │   Verifier    │   │   Verifier    │
   │               │   │               │   │               │
   │ • Schema      │   │ • Critic      │   │ • Compile     │
   │ • Exit codes  │   │   agent       │   │ • Test        │
   │ • Regex       │   │ • CoT eval    │   │ • Lint        │
   │ • Bounds      │   │ • Confidence  │   │ • Type check  │
   │ • Parse check │   │ • Evidence    │   │ • Security    │
   │               │   │   citation    │   │ • Format      │
   └───────┬───────┘   └───────┬───────┘   └───────┬───────┘
           │                   │                    │
           └───────────────────┼────────────────────┘
                               │
                               ▼
                    ┌──────────────────┐
                    │  Verdict +       │
                    │  ConfidenceScore │
                    │  + Evidence      │
                    └──────────────────┘
```

### Gate Implementation Details

#### Gate 1: Structural Validation

```rust
pub struct StructuralVerifier {
    schema_registry: HashMap<String, JsonSchema>,
    parsers: HashMap<String, Parser>,
}

impl StructuralVerifier {
    /// Validates that the output is well-formed and matches expected schema.
    pub async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> GateResult {
        for criterion in criteria {
            match criterion {
                Criterion::JsonSchema(schema_id) => {
                    let schema = self.schema_registry.get(schema_id).ok_or(VerificationError::UnknownSchema)?;
                    let parsed: serde_json::Value = serde_json::from_str(&result.output)
                        .map_err(|e| GateFailure::ParseError(e.to_string()))?;
                    schema.validate(&parsed)
                        .map_err(|e| GateFailure::SchemaViolation(e.to_string()))?;
                }
                Criterion::RequiredFields(fields) => {
                    for field in fields {
                        if !result.output.contains(field) {
                            return Err(GateFailure::MissingField(field.clone()));
                        }
                    }
                }
                Criterion::RegexPattern(pattern) => {
                    let re = Regex::new(pattern).map_err(|_| VerificationError::InvalidRegex)?;
                    if !re.is_match(&result.output) {
                        return Err(GateFailure::PatternMismatch(pattern.clone()));
                    }
                }
                _ => {}
            }
        }
        Ok(GatePass)
    }
}
```

#### Gate 2: Deterministic Checks

```rust
pub struct DeterministicVerifier {
    max_output_length: usize,
    max_tool_calls: usize,
}

impl DeterministicVerifier {
    /// Verifies tool execution integrity and output bounds.
    pub async fn verify(&self, result: &StepResult, step: &Step) -> GateResult {
        // Did the tools actually execute?
        if result.tool_results.is_empty() && !step.allows_empty_tool_result {
            return Err(GateFailure::NoToolExecution);
        }

        // Check exit codes
        for tr in &result.tool_results {
            if let Some(code) = tr.exit_code {
                if code != 0 && !step.allows_nonzero_exit {
                    return Err(GateFailure::NonZeroExit(code, tr.tool_name.clone()));
                }
            }
        }

        // Check output bounds
        if result.output.len() > self.max_output_length {
            return Err(GateFailure::OutputTooLong(result.output.len()));
        }

        // Check tool call count
        if result.tool_results.len() > self.max_tool_calls {
            return Err(GateFailure::TooManyToolCalls(result.tool_results.len()));
        }

        Ok(GatePass)
    }
}
```

#### Gate 3: LLM-Based Verification

```rust
pub struct LLMVerifier {
    llm: Arc<dyn LLMProvider>,
    critic_prompt_template: String,
}

impl LLMVerifier {
    /// Uses a critic LLM to evaluate execution quality against criteria.
    pub async fn verify(&self, result: &StepResult, criteria: &[Criterion], context: &Context) -> Result<Verdict, VerificationError> {
        let prompt = self.build_critic_prompt(result, criteria, context);

        let response = self.llm
            .generate_structured::<CriticEvaluation>(&prompt)
            .await?;

        let confidence = ConfidenceScore {
            structural: 1.0, // already passed gate 1
            llm_verification: response.confidence,
            tool_verification: 0.0, // filled by gate 4
            self_consistency: 0.0,  // filled by gate 5 if enabled
            overall: 0.0,
        };

        Ok(Verdict {
            passed: response.passed,
            confidence,
            evidence: response.evidence,
            issues: response.issues,
            suggestions: response.suggestions,
        })
    }

    fn build_critic_prompt(&self, result: &StepResult, criteria: &[Criterion], context: &Context) -> String {
        format!(
            r#"You are a critic evaluating an AI agent's work.

## Task Context
{}

## Execution Result
{}

## Evaluation Criteria
{}

## Instructions
Evaluate whether the result satisfies ALL criteria.
Provide:
1. A pass/fail decision
2. A confidence score (0.0-1.0) for your evaluation
3. Specific evidence from the result that supports your decision
4. Any issues found
5. Suggestions for improvement if failed

## Output Format
Respond in valid JSON with fields: passed, confidence, evidence, issues, suggestions"#,
            context, result.output, criteria.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\n")
        )
    }
}
```

#### Gate 4: Tool-Based Verification

```rust
pub struct ToolVerifier {
    tool_registry: Arc<ToolRegistry>,
    language_configs: HashMap<String, LanguageConfig>,
}

impl ToolVerifier {
    /// Executes verification tools (compile, test, lint) on the result.
    pub async fn verify(&self, result: &StepResult, language: &str) -> Result<GateResult, VerificationError> {
        let config = self.language_configs.get(language)
            .ok_or(VerificationError::UnsupportedLanguage(language.to_string()))?;

        // Compile check
        if let Some(compile_cmd) = &config.compile_command {
            let compile_result = self.tool_registry
                .execute("shell", &json!({ "command": compile_cmd, "workdir": result.workdir }))
                .await?;
            if !compile_result.success {
                return Err(GateFailure::CompilationFailed(compile_result.stderr));
            }
        }

        // Test execution
        if let Some(test_cmd) = &config.test_command {
            let test_result = self.tool_registry
                .execute("shell", &json!({ "command": test_cmd, "workdir": result.workdir }))
                .await?;
            if !test_result.success {
                return Err(GateFailure::TestsFailed(test_result.stdout, test_result.stderr));
            }
        }

        Ok(GatePass)
    }
}
```

#### Gate 5: Self-Consistency

```rust
pub struct SelfConsistencyVerifier {
    llm: Arc<dyn LLMProvider>,
    num_samples: usize,
    agreement_threshold: f64, // e.g., 0.8 = 80% agreement required
}

impl SelfConsistencyVerifier {
    /// Generates N independent solutions and checks for agreement.
    pub async fn verify(&self, task: &Task, context: &Context) -> Result<ConsistencyResult, VerificationError> {
        let mut samples = Vec::with_capacity(self.num_samples);
        let executor = ExecutorAgent::new(self.llm.clone());

        for i in 0..self.num_samples {
            // Use different temperature to get diverse samples
            let sample = executor.execute_with_temperature(task, context, 0.7 + (i as f64 * 0.1)).await?;
            samples.push(sample);
        }

        // Compare outputs for structural agreement
        let pairwise_agreements = self.compute_pairwise_agreement(&samples);
        let mean_agreement = pairwise_agreements.iter().sum::<f64>() / pairwise_agreements.len() as f64;

        Ok(ConsistencyResult {
            agreement_score: mean_agreement,
            samples,
            divergent_samples: self.find_divergent_samples(&samples, &pairwise_agreements),
            majority_output: self.majority_vote(&samples),
        })
    }

    fn compute_pairwise_agreement(&self, samples: &[StepResult]) -> Vec<f64> {
        // Computes semantic similarity between all sample pairs
        // Uses embedding cosine similarity + structural comparison
    }
}
```

### Reflection Loop — Full Implementation

```rust
pub struct ReflectionLoop {
    executor: Arc<ExecutorAgent>,
    verifier: Arc<VerifierAgent>,
    config: ReflectionConfig,
    failure_tracker: FailureTracker,
}

pub struct ReflectionConfig {
    pub max_cycles: usize,
    pub confidence_threshold: f64,
    pub escalation_threshold: f64, // confidence below this → human
    pub max_consecutive_same_failure: usize,
}

pub struct FailureTracker {
    pub pattern_counts: HashMap<String, usize>,
    pub consecutive_same: usize,
    pub last_failure_type: Option<String>,
}

impl ReflectionLoop {
    pub async fn run(
        &self,
        task: &Task,
        context: &Context,
    ) -> Result<FinalOutput, AgentError> {
        let mut history: Vec<Turn> = Vec::new();
        let mut cycle = 0;

        loop {
            // Phase 1: Execute
            cycle += 1;
            let output = self.executor
                .execute_with_reflection(task, context, &history)
                .await?;

            // Phase 2: Verify through all gates
            let verdict = self.run_verification_pipeline(&output, task).await?;

            // Phase 3: Check result
            if verdict.is_pass() || cycle >= self.config.max_cycles {
                if verdict.is_pass() {
                    return Ok(FinalOutput::Success(output, verdict, cycle));
                } else {
                    return Err(AgentError::MaxRetriesExceeded(output, verdict, cycle));
                }
            }

            // Phase 4: Reflect
            let reflection = self.generate_reflection(task, &output, &verdict, &history).await?;

            // Phase 5: Track failure patterns
            self.failure_tracker.record(&reflection);
            if self.failure_tracker.should_escalate(&self.config) {
                return Err(AgentError::Escalated(FinalOutput::Partial(output, verdict, cycle), reflection));
            }

            history.push(Turn { output, verdict, reflection });
        }
    }

    async fn run_verification_pipeline(
        &self,
        output: &StepResult,
        task: &Task,
    ) -> Result<Verdict, AgentError> {
        // Gate 1: Structural
        let gate1 = self.verifier.structural_verify(output, &task.criteria).await?;
        if gate1.is_fail() {
            return Ok(Verdict::fail_with_gate(GateKind::Structural, gate1));
        }

        // Gate 2: Deterministic
        let gate2 = self.verifier.deterministic_verify(output, task).await?;
        if gate2.is_fail() {
            return Ok(Verdict::fail_with_gate(GateKind::Deterministic, gate2));
        }

        // Gate 3: LLM-Based
        let gate3 = self.verifier.llm_verify(output, &task.criteria, task.context()).await?;
        if gate3.is_fail() && gate3.confidence < self.config.confidence_threshold {
            return Ok(gate3);
        }

        // Gate 4: Tool-Based (if applicable)
        let gate4 = if task.has_tool_verification() {
            self.verifier.tool_verify(output, task.language()).await?
        } else {
            GatePass::Skipped
        };
        if gate4.is_fail() {
            return Ok(Verdict::fail_with_gate(GateKind::Tool, gate4));
        }

        // Gate 5: Self-Consistency (only if confidence is borderline)
        if gate3.confidence < self.config.confidence_threshold * 1.2 {
            let gate5 = self.verifier.consistency_verify(task, task.context()).await?;
            if gate5.agreement_score < self.config.confidence_threshold {
                return Ok(Verdict::fail_with_gate(GateKind::Consistency, gate5));
            }
        }

        Ok(Verdict::pass(ConfidenceScore::from_gates(&[gate1, gate2, gate3, gate4])))
    }

    async fn generate_reflection(
        &self,
        task: &Task,
        output: &StepResult,
        verdict: &Verdict,
        history: &[Turn],
    ) -> Result<Reflection, AgentError> {
        let previous_reflections: Vec<&str> = history.iter()
            .map(|t| t.reflection.summary.as_str())
            .collect();

        let prompt = format!(
            r#"You attempted a task and it failed verification.

## Task
{}

## Your Output
{}

## Verification Failure Reason
{}

## Previous Attempts & Reflections
{}

## Instructions
Analyze why the verification failed. Provide:
1. Root cause analysis
2. What specifically needs to change in the next attempt
3. What should be kept the same
4. Any additional information needed
5. Confidence that the next attempt will succeed (0-1)

Be specific. "Try harder" is not acceptable."#,
            task.description,
            output.output,
            verdict.issues.join("\n"),
            previous_reflections.iter().enumerate()
                .map(|(i, r)| format!("Attempt {}: {}", i + 1, r))
                .collect::<Vec<_>>().join("\n")
        );

        self.verifier.llm()
            .generate_structured::<Reflection>(&prompt)
            .await
    }
}
```

---

## 7. Memory & Persistence Architecture

### Memory Store Trait

```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a memory entry with optional embedding.
    async fn store(&self, entry: MemoryEntry) -> Result<(), MemoryError>;

    /// Retrieve by exact key match.
    async fn retrieve(&self, key: &str) -> Result<Option<MemoryEntry>, MemoryError>;

    /// Semantic search over memory entries.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Search by time range.
    async fn search_by_time(&self, from: DateTime<Utc>, to: DateTime<Utc>, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Get recent entries (for context window).
    async fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Remove entries older than the specified duration.
    async fn prune(&self, older_than: Duration) -> Result<usize, MemoryError>;

    /// Clear all memory.
    async fn clear(&self) -> Result<(), MemoryError>;
}
```

### Memory Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     Memory Manager                            │
│  (routes queries to appropriate store based on type)          │
└──────────────────────────────────────────────────────────────┘
         │                  │                  │
         ▼                  ▼                  ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Working Memory   │ │ Long-Term       │ │ Vector Memory    │
│ (In-Memory LRU)  │ │ Memory (SQLite) │ │ (sqlite-vec /   │
│                  │ │                 │ │  pgvector)       │
│ • Session context│ │ • Task history  │ │ • Semantic       │
│ • Current task   │ │ • Agent logs    │ │   search         │
│ • Recent N turns │ │ • Checkpoints   │ │ • Embedding-     │
│ • Tool cache     │ │ • User prefs    │ │   based retrieval│
│                  │ │ • Learned       │ │ • Similarity     │
│ • Fast (ns/μs)   │ │   patterns      │ │   matching       │
│ • Volatile       │ │ • Persistent    │ │ • Persistent     │
│ • Bounded size   │ │ • SQL queries   │ │ • ANN search     │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

### Three-Tier Memory Retrieval

```rust
pub struct MemoryManager {
    working: WorkingMemory,
    long_term: LongTermMemory,
    vector: VectorMemory,
    config: MemoryConfig,
}

impl MemoryManager {
    /// Retrieve context with tiered fallback.
    pub async fn get_context(&self, query: &MemoryQuery) -> Result<Context, MemoryError> {
        // Tier 1: Working memory (fastest)
        if let Some(ctx) = self.working.get(&query.key).await? {
            tracing::debug!(key = ?query.key, "Working memory hit");
            return Ok(ctx);
        }

        // Tier 2: Long-term memory by key
        if let Some(entry) = self.long_term.retrieve(&query.key).await? {
            self.working.set(&query.key, entry.clone()).await?; // promote to working
            tracing::debug!(key = ?query.key, "Long-term memory hit");
            return Ok(entry.context);
        }

        // Tier 3: Semantic search (slowest, most flexible)
        let results = self.vector.search(&query.natural_query, query.limit).await?;
        if !results.is_empty() {
            let best = results.into_iter().next().unwrap();
            self.working.set(&query.key, best.clone()).await?;
            tracing::debug!(key = ?query.key, "Vector memory hit");
            return Ok(best.context);
        }

        Err(MemoryError::NotFound(query.key.clone()))
    }

    /// Store with automatic embedding.
    pub async fn store(&self, key: &str, context: Context, importance: Importance) -> Result<(), MemoryError> {
        // Always store in working memory
        self.working.set(key, context.clone()).await?;

        // Promote to long-term based on importance
        if importance >= Importance::Normal {
            self.long_term.store(MemoryEntry { key: key.into(), context: context.clone() }).await?;
        }

        // Vectorize if semantically searchable
        if importance >= Importance::Important {
            self.vector.store(MemoryEntry { key: key.into(), context }).await?;
        }

        Ok(())
    }
}
```

### Checkpoint System

```rust
pub struct Checkpointer {
    db: sqlx::SqlitePool,
}

impl Checkpointer {
    /// Save a checkpoint for a specific task step.
    pub async fn save(&self, task_id: &TaskId, step_id: &StepId, state: &CheckpointState) -> Result<(), CheckpointError> {
        let encoded = bincode::serialize(state)?;
        sqlx::query(
            "INSERT INTO checkpoints (task_id, step_id, state, created_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(task_id, step_id) DO UPDATE SET state = $3, created_at = $4"
        )
        .bind(task_id.to_string())
        .bind(step_id.to_string())
        .bind(encoded)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Load the latest checkpoint for a task.
    pub async fn load(&self, task_id: &TaskId) -> Result<Option<TaskCheckpoint>, CheckpointError> {
        let row = sqlx::query_as::<_, CheckpointRow>(
            "SELECT * FROM checkpoints WHERE task_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(task_id.to_string())
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(row) => {
                let state: CheckpointState = bincode::deserialize(&row.state)?;
                Ok(Some(TaskCheckpoint {
                    task_id: task_id.clone(),
                    step_id: row.step_id.into(),
                    state,
                    created_at: row.created_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// List all active (unfinished) checkpoints.
    pub async fn list_active(&self) -> Result<Vec<TaskId>, CheckpointError> {
        // Useful for crash recovery on startup
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT task_id FROM checkpoints
             WHERE task_id NOT IN (SELECT task_id FROM completed_tasks)"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(TaskId::from).collect())
    }
}
```

---

## 8. Tool System Architecture

### Tool Trait

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name for routing tool calls.
    fn name(&self) -> &str;

    /// Description of what the tool does (for LLM context).
    fn description(&self) -> &str;

    /// JSON Schema for the tool's input parameters.
    fn input_schema(&self) -> Value;

    /// Execute the tool with given parameters.
    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError>;

    /// Estimated cost of a single invocation (for budgeting).
    fn estimated_cost(&self) -> ToolCost;

    /// Whether this tool requires human approval.
    fn requires_approval(&self) -> bool;
}
```

### Tool Registry with Permissions

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    permissions: Permissions,
    rate_limiter: RateLimiter,
}

impl ToolRegistry {
    /// Register a new tool.
    pub fn register(&mut self, tool: Box<dyn Tool>) -> Result<(), RegistryError> {
        let name = tool.name().to_string();
        if self.tools.contains_key(&name) {
            return Err(RegistryError::DuplicateTool(name));
        }
        self.tools.insert(name, tool);
        Ok(())
    }

    /// Execute a tool with permission and rate-limit checks.
    pub async fn execute(&self, tool_name: &str, params: Value, agent_id: &AgentId) -> Result<ToolOutput, ToolError> {
        // 1. Check tool exists
        let tool = self.tools.get(tool_name)
            .ok_or(ToolError::NotFound(tool_name.to_string()))?;

        // 2. Permission check
        if !self.permissions.is_allowed(agent_id, tool_name, &params) {
            return Err(ToolError::PermissionDenied(tool_name.to_string()));
        }

        // 3. Rate limit check
        self.rate_limiter.check(tool_name).await?;

        // 4. Approval check
        if tool.requires_approval() {
            return Err(ToolError::RequiresApproval(tool_name.to_string()));
        }

        // 5. Execute
        let start = Instant::now();
        let result = tool.execute(params).await.map_err(|e| {
            tracing::error!(tool = tool_name, error = %e, "Tool execution failed");
            e
        })?;
        let duration = start.elapsed();

        tracing::info!(
            tool = tool_name,
            duration_ms = duration.as_millis() as u64,
            success = result.success,
            "Tool execution completed"
        );

        Ok(result)
    }
}
```

### Built-In Tool Implementations

#### Filesystem Tool

```rust
pub struct FilesystemTool {
    allowed_paths: Vec<PathBuf>,
    max_file_size: usize,
}

impl FilesystemTool {
    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        let resolved = std::path::Path::new(path).canonicalize()?;
        if !self.allowed_paths.iter().any(|p| resolved.starts_with(p)) {
            return Err(ToolError::PathNotAllowed(resolved));
        }
        Ok(resolved)
    }
}

#[async_trait]
impl Tool for FilesystemTool {
    fn name(&self) -> &str { "filesystem" }
    fn description(&self) -> &str { "Read, write, list, and manage files and directories" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["read", "write", "list", "delete", "move", "copy", "exists", "metadata"] },
                "path": { "type": "string" },
                "content": { "type": "string" },
                "recursive": { "type": "boolean" }
            },
            "required": ["action", "path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let action = params["action"].as_str().ok_or(ToolError::MissingParam("action"))?;
        let path = params["path"].as_str().ok_or(ToolError::MissingParam("path"))?;
        let resolved = self.resolve_path(path)?;

        match action {
            "read" => {
                let content = tokio::fs::read_to_string(&resolved).await?;
                Ok(ToolOutput::success(content))
            }
            "write" => {
                let content = params["content"].as_str().ok_or(ToolError::MissingParam("content"))?;
                if content.len() > self.max_file_size {
                    return Err(ToolError::FileTooLarge(content.len()));
                }
                tokio::fs::write(&resolved, content).await?;
                Ok(ToolOutput::success(format!("Written {} bytes", content.len())))
            }
            "list" => {
                let mut entries = tokio::fs::read_dir(&resolved).await?;
                let mut items = Vec::new();
                while let Some(entry) = entries.next_entry().await? {
                    items.push(entry.file_name().to_string_lossy().to_string());
                }
                Ok(ToolOutput::success(serde_json::to_string(&items)?))
            }
            "delete" => {
                if resolved.is_dir() {
                    tokio::fs::remove_dir_all(&resolved).await?;
                } else {
                    tokio::fs::remove_file(&resolved).await?;
                }
                Ok(ToolOutput::success(format!("Deleted {}", path)))
            }
            _ => Err(ToolError::UnknownAction(action.to_string()))
        }
    }
}
```

#### Shell Tool

```rust
pub struct ShellTool {
    allowed_commands: Vec<Regex>,
    blocked_commands: Vec<Regex>,
    max_execution_time: Duration,
    max_output_size: usize,
    sandbox_user: Option<String>,
}

impl ShellTool {
    fn is_command_allowed(&self, command: &str) -> bool {
        if self.blocked_commands.iter().any(|re| re.is_match(command)) {
            return false;
        }
        if self.allowed_commands.is_empty() {
            return true; // allow all if no allowlist
        }
        self.allowed_commands.iter().any(|re| re.is_match(command))
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str { "shell" }
    fn description(&self) -> &str { "Execute shell commands and return output" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "workdir": { "type": "string" },
                "env": { "type": "object" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let command = params["command"].as_str().ok_or(ToolError::MissingParam("command"))?;

        if !self.is_command_allowed(command) {
            return Err(ToolError::CommandNotAllowed(command.to_string()));
        }

        let output = tokio::process::Command::new("cmd")
            .arg("/C")
            .arg(command)
            .current_dir(params["workdir"].as_str().unwrap_or("."))
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ToolOutput {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
            truncated: stdout.len() > self.max_output_size,
        })
    }
}
```

#### Browser Tool

```rust
pub struct BrowserTool {
    browser: chromiumoxide::Browser,
}

impl BrowserTool {
    pub async fn new(headless: bool) -> Result<Self, ToolError> {
        let (browser, _) = chromiumoxide::Browser::builder()
            .with_head(headless)
            .build()
            .await?;
        Ok(Self { browser })
    }
}

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str { "browser" }
    fn description(&self) -> &str { "Navigate, interact, and extract data from web pages" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["navigate", "click", "type", "extract", "screenshot", "scroll", "wait", "evaluate"]
                },
                "url": { "type": "string" },
                "selector": { "type": "string" },
                "text": { "type": "string" },
                "script": { "type": "string" }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        let action = params["action"].as_str().ok_or(ToolError::MissingParam("action"))?;
        let page = self.browser.new_page("about:blank").await?;

        match action {
            "navigate" => {
                let url = params["url"].as_str().ok_or(ToolError::MissingParam("url"))?;
                page.goto(url).await?;
                Ok(ToolOutput::success(format!("Navigated to {}", url)))
            }
            "extract" => {
                let content = page.content().await?;
                Ok(ToolOutput::success(content))
            }
            "screenshot" => {
                let screenshot = page.screenshot().await?;
                Ok(ToolOutput::success_with_data(base64::encode(&screenshot)))
            }
            "click" => {
                let selector = params["selector"].as_str().ok_or(ToolError::MissingParam("selector"))?;
                page.find_element(selector).await?.click().await?;
                Ok(ToolOutput::success(format!("Clicked {}", selector)))
            }
            _ => Err(ToolError::UnknownAction(action.to_string()))
        }
    }
}
```

---

## 9. Cross-Platform Architecture

### Target Matrix

| Target | Renderer | Input | Build Command |
|--------|----------|-------|--------------|
| Windows | WebView2 (native) | Mouse, Keyboard, Touch | `cargo tauri build` |
| macOS | WKWebView (native) | Mouse, Keyboard, Touch | `cargo tauri build` |
| Linux | WebKitGTK (native) | Mouse, Keyboard, Touch | `cargo tauri build` |
| Android | WebView (embedded) | Touch | `cargo apk` |
| iOS | WKWebView (native) | Touch | `cargo bundle` |
| Web | Browser (WASM) | Mouse, Keyboard, Touch | `dx build --release` |

### Platform Abstraction Layer

```rust
// In agent-core
pub trait Platform: Send + Sync {
    fn platform_name(&self) -> &str;
    fn data_dir(&self) -> PathBuf;
    fn cache_dir(&self) -> PathBuf;
    fn config_dir(&self) -> PathBuf;
    fn os_type(&self) -> OSType;
    fn is_mobile(&self) -> bool;
    fn is_desktop(&self) -> bool;
    fn is_web(&self) -> bool;
}

// Platform-specific implementations
#[cfg(target_os = "windows")]
pub struct WindowsPlatform;

#[cfg(target_os = "linux")]
pub struct LinuxPlatform;

#[cfg(target_os = "macos")]
pub struct MacOSPlatform;

#[cfg(target_family = "wasm")]
pub struct WebPlatform;
```

### IPC Architecture

```
┌──────────────────────────────────────┐
│         UI Process (Dioxus)           │
│                                      │
│  ┌──────────────────────────────────┐│
│  │  Dioxus VirtualDOM + Components ││
│  └──────────────┬───────────────────┘│
│                 │                     │
│  ┌──────────────▼───────────────────┐│
│  │  use_resource → subscribe()      ││
│  │  dispatch() on user action       ││
│  └──────────────┬───────────────────┘│
└─────────────────┼────────────────────┘
                  │ WebSocket / IPC
                  │ (JSON-RPC over WS /
                  │  Unix domain socket /
                  │  Named pipe)
                  │
┌─────────────────▼────────────────────┐
│         Engine Process (Rust)         │
│                                      │
│  ┌──────────────────────────────────┐│
│  │  AgentEngine                     ││
│  │  • subscribe() → AgentEvent     ││
│  │  • dispatch() → UserCommand     ││
│  └──────────────────────────────────┘│
└──────────────────────────────────────┘
```

### UI Component Tree (Dioxus)

```rust
fn App(cx: Scope) -> Element {
    let engine = use_agent_engine(cx);
    let state = use_app_state(cx);

    cx.render(rsx! {
        div { class: "app-container",
            Sidebar {
                SessionList { sessions: state.sessions }
                AgentList { agents: state.agents }
            }
            MainContent {
                TaskDashboard {
                    TaskInput { on_submit: move |task| engine.dispatch(UserCommand::SubmitTask(task)) }
                    TaskList { tasks: state.active_tasks }
                    TaskDetail { task: state.selected_task }
                }
                AgentPanel {
                    AgentStatusCard { for each agent in state.agents }
                    AgentLog { entries: state.recent_events }
                }
                VerificationPanel {
                    VerdictDisplay { verdict: state.latest_verdict }
                    ConfidenceGauge { score: state.confidence_score }
                    ApprovalDialog {
                        show: state.pending_approval.is_some()
                        request: state.pending_approval
                        on_approve: move |id| engine.dispatch(UserCommand::ApproveAction(id))
                        on_reject: move |(id, reason)| engine.dispatch(UserCommand::RejectAction(id, reason))
                    }
                }
            }
            StatusBar {
                ConnectedIndicator { connected: state.connected }
                TaskCounter { count: state.active_tasks.len() }
                AgentCounter { count: state.agents.len() }
                TokenCounter { used: state.tokens_used, limit: state.token_limit }
            }
        }
    })
}
```

---

## 10. Concurrency Model

### Thread Pool Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      tokio Runtime                            │
│  (multi-threaded, work-stealing scheduler)                    │
│                                                               │
│  ┌──────────────────────────────┐  ┌──────────────────────┐  │
│  │    Agent Tasks (N workers)   │  │  I/O Bound Tasks     │  │
│  │                              │  │  (file, network,     │  │
│  │  ┌────┐ ┌────┐ ┌────┐       │  │   browser, etc.)     │  │
│  │  │ A1 │ │ A2 │ │ A3 │ ...   │  │                      │  │
│  │  └────┘ └────┘ └────┘       │  │  tokio::spawn for    │  │
│  │                              │  │  all async I/O       │  │
│  │  Bounded by Semaphore(N)     │  │                      │  │
│  └──────────────────────────────┘  └──────────────────────┘  │
│                                                               │
│  ┌──────────────────────────────┐  ┌──────────────────────┐  │
│  │    LLM Tasks (M concurrent)  │  │  Tool Tasks          │  │
│  │                              │  │  (per-tool          │  │
│  │  • Local models: 1 at a time │  │   semaphore)        │  │
│  │  • Remote APIs: N concurrent │  │                      │  │
│  │  • Queued if busy            │  │  • Shell: max 1     │  │
│  └──────────────────────────────┘  │  • Browser: max 3   │  │
│                                    │  • File: max 5      │  │
│  ┌──────────────────────────────┐  └──────────────────────┘  │
│  │    Memory Tasks              │                             │
│  │                              │                             │
│  │  • Single writer,            │                             │
│  │    multiple readers          │                             │
│  │  • ReadWriteLock             │                             │
│  └──────────────────────────────┘                             │
└─────────────────────────────────────────────────────────────┘
```

### Synchronization Primitives

```rust
use tokio::sync::{Semaphore, RwLock, mpsc, broadcast, oneshot};
use std::sync::Arc;

pub struct ConcurrencyConfig {
    pub max_concurrent_agents: usize,
    pub max_concurrent_llm_local: usize,  // typically 1
    pub max_concurrent_llm_remote: usize, // typically 8-16
    pub per_tool_limits: HashMap<String, usize>,
    pub event_bus_capacity: usize,
}

pub struct AgentPool {
    semaphore: Arc<Semaphore>,
    agents: Vec<Box<dyn Agent>>,
}

impl AgentPool {
    pub async fn acquire(&self) -> AgentHandle {
        let permit = self.semaphore.acquire().await;
        AgentHandle { permit }
    }
}

pub struct ToolPool {
    limiters: HashMap<String, Arc<Semaphore>>,
}

impl ToolPool {
    pub async fn acquire(&self, tool_name: &str) -> Result<ToolPermit, ToolError> {
        let semaphore = self.limiters.get(tool_name)
            .ok_or(ToolError::NotFound(tool_name.to_string()))?;
        let permit = semaphore.acquire().await;
        Ok(ToolPermit { tool_name: tool_name.to_string(), permit })
    }
}
```

### Graceful Shutdown

```rust
pub struct Shutdown {
    signal: oneshot::Sender<()>,
    completed: Arc<tokio::sync::Notify>,
}

impl AgentEngine {
    pub async fn shutdown(&self) -> Result<(), ShutdownError> {
        tracing::info!("Initiating graceful shutdown");

        // 1. Stop accepting new tasks
        self.orchestrator.pause_accepting().await;

        // 2. Drain active tasks (with timeout)
        let drain_timeout = Duration::from_secs(30);
        tokio::select! {
            _ = self.orchestrator.drain_active_tasks() => {
                tracing::info!("All active tasks completed");
            }
            _ = tokio::time::sleep(drain_timeout) => {
                tracing::warn!("Forcing shutdown with {} tasks still active", self.orchestrator.active_count());
            }
        }

        // 3. Save checkpoint for remaining tasks
        self.checkpointer.save_all(&self.orchestrator.active_tasks()).await?;

        // 4. Flush event bus
        while self.event_bus.receiver_count() > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 5. Close database connections
        self.db.close().await;

        tracing::info!("Shutdown complete");
        Ok(())
    }
}
```

---

## 11. Security Architecture

### Tiered Sandbox Model

```
┌──────────────────────────────────────────────────────────────────┐
│                     Sandbox Manager                               │
│                                                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
│  │    L0:      │  │    L1:      │  │    L2:      │  │   L3:   │ │
│  │  In-Process │  │  Process    │  │  Container  │  │ MicroVM │ │
│  │             │  │             │  │             │  │         │ │
│  │ • No spawn  │  │ • Restricted│  │ • Docker    │  │Firecrack│ │
│  │ • catch_un- │  │   user      │  │ • Per-task  │  │ • Full  │ │
│  │   wind      │  │ • Job obj   │  │ • Resource  │  │  isolat-│ │
│  │ • Pure Rust │  │ • cgroups   │  │   limits    │  │  ion    │ │
│  │ • Read-only │  │ • Timeout   │  │ • Network   │  │ • Multi-│ │
│  │   tools     │  │ • Output    │  │   policy    │  │  tenant │ │
│  │             │  │   cap       │  │ • Ephemeral │  │ • Untru-│ │
│  └─────────────┘  └─────────────┘  └─────────────┘  │  sted   │ │
│                                                      │  3rd    │ │
│  Latency: μs        Latency: ms       Latency: s     │  party  │ │
│  Isolation: None    Isolation: OS     Isolation: VM  └─────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

### Permission System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub default_policy: Policy,
    pub agent_permissions: HashMap<AgentId, Vec<PermissionEntry>>,
    pub role_permissions: HashMap<AgentRole, Vec<PermissionEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub tool: ToolPattern,
    pub actions: Vec<Action>,
    pub path_glob: Option<String>,
    pub network: NetworkPolicy,
    pub max_execution_time: Duration,
    pub max_calls_per_task: Option<usize>,
    pub requires_human_approval: bool,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Execute,
    Delete,
    Network,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicy {
    /// No network access
    DenyAll,
    /// Only these domains
    AllowList(Vec<String>),
    /// All except these domains
    BlockList(Vec<String>),
    /// Full network access
    AllowAll,
}

impl Permissions {
    pub fn is_allowed(&self, agent_id: &AgentId, tool_name: &str, params: &Value) -> bool {
        // 1. Check agent-specific permissions
        if let Some(entries) = self.agent_permissions.get(agent_id) {
            for entry in entries {
                if entry.tool.matches(tool_name) {
                    return self.evaluate_entry(entry, params);
                }
            }
        }

        // 2. Check role-based permissions
        if let Some(entries) = self.role_permissions.get(&agent_id.role()) {
            for entry in entries {
                if entry.tool.matches(tool_name) {
                    return self.evaluate_entry(entry, params);
                }
            }
        }

        // 3. Default policy
        matches!(self.default_policy, Policy::Allow)
    }

    fn evaluate_entry(&self, entry: &PermissionEntry, params: &Value) -> bool {
        // Check path restrictions
        if let Some(glob) = &entry.path_glob {
            if let Some(path) = params["path"].as_str() {
                if !glob_match(glob, path) {
                    return false;
                }
            }
        }

        // Check network policy
        if let Some(url) = params["url"].as_str() {
            if !entry.network.is_allowed(url) {
                return false;
            }
        }

        // Check execution time
        if entry.max_execution_time < Duration::from_secs(params["timeout"].as_u64().unwrap_or(0)) {
            return false;
        }

        true
    }
}
```

### Security Audit Trail

```rust
pub struct AuditLog {
    db: sqlx::SqlitePool,
}

impl AuditLog {
    pub async fn log_tool_call(&self, agent_id: &AgentId, call: &ToolCall, result: &Result<ToolOutput, ToolError>) {
        sqlx::query(
            "INSERT INTO audit_log (timestamp, agent_id, tool_name, params, success, error)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(Utc::now())
        .bind(agent_id.to_string())
        .bind(call.name.clone())
        .bind(serde_json::to_string(&call.params)?)
        .bind(result.is_ok())
        .bind(result.as_ref().err().map(|e| e.to_string()))
        .execute(&self.db)
        .await?;
    }

    pub async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>, AuditError> {
        // Supports filtering by agent, tool, time range, success/failure
    }
}
```

---

## 12. Plugin Architecture

### WASM-Based Plugin System

```rust
/// Plugin trait that external WASM modules implement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub tools: Vec<PluginToolDef>,
    pub permissions: Vec<String>,
    pub min_engine_version: String,
}

pub struct PluginManager {
    engine: wasmtime::Engine,
    store: wasmtime::Store<PluginData>,
    plugins: HashMap<String, PluginInstance>,
}

impl PluginManager {
    /// Load a plugin from a WASM binary.
    pub async fn load(&mut self, bytes: &[u8]) -> Result<PluginId, PluginError> {
        let module = wasmtime::Module::new(&self.engine, bytes)?;
        let mut store = wasmtime::Store::new(&self.engine, PluginData::new());

        // Initialize plugin
        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

        // Extract manifest
        let manifest: PluginManifest = instance
            .get_export(&mut store, "manifest")
            .and_then(|e| e.into_func())
            .map(|f| {
                // Call manifest function and parse JSON
            })
            .ok_or(PluginError::MissingManifest)??;

        // Register tools
        for tool_def in &manifest.tools {
            let tool = WasmTool::new(instance.clone(), tool_def.clone());
            self.tool_registry.register(Box::new(tool))?;
        }

        self.plugins.insert(manifest.name.clone(), PluginInstance { manifest, instance });
        Ok(PluginId(manifest.name))
    }
}

/// WASM-backed tool implementation.
pub struct WasmTool {
    instance: wasmtime::Instance,
    tool_def: PluginToolDef,
}

#[async_trait]
impl Tool for WasmTool {
    fn name(&self) -> &str { &self.tool_def.name }
    fn description(&self) -> &str { &self.tool_def.description }
    fn input_schema(&self) -> Value { self.tool_def.input_schema.clone() }

    async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
        // Call WASM export with serialized params
        // WASM runs in sandboxed environment
    }
}
```

---

## 13. Data Flow Diagrams

### Single Task Execution Flow

```
User
  │
  ├── SubmitTask("Build user registration endpoint")
  │
  ▼
TaskScheduler.enqueue(Task)
  │
  ▼
Orchestrator.start(Task)
  │
  ├── Event: TaskStarted
  │
  ├── [PlannerAgent]
  │   ├── think(task, context) → Thought
  │   ├── Event: ThoughtComplete
  │   ├── plan(thought) → Plan
  │   │   └── Plan: [Step 1: Research, Step 2: Implement, Step 3: Test, Step 4: Verify]
  │   └── Event: PlanCreated
  │
  ├── [ExecutorAgent]
  │   ├── execute(Step 1: Research)
  │   │   ├── Tool: filesystem.list → [existing files]
  │   │   ├── Tool: http.get("https://api.docs.dev") → docs
  │   │   └── Result: documentation gathered
  │   │
  │   ├── [VerifierAgent]
  │   │   ├── Gate 1: Schema check ✓ (valid JSON)
  │   │   ├── Gate 2: Deterministic ✓ (exit code 0)
  │   │   ├── Gate 3: LLM check → "Research is comprehensive" (confidence: 0.92)
  │   │   └── Verdict: PASS (0.92)
  │   │
  │   ├── execute(Step 2: Implement)
  │   │   ├── Tool: filesystem.write("src/routes/auth.rs", code)
  │   │   └── Result: code written
  │   │
  │   ├── [VerifierAgent]
  │   │   ├── Gate 1: Schema ✓
  │   │   ├── Gate 2: Deterministic ✓
  │   │   ├── Gate 3: LLM check → "Code looks correct" (confidence: 0.85)
  │   │   ├── Gate 4: Tool check
  │   │   │   ├── shell("cargo check") → Syntax OK
  │   │   │   ├── shell("cargo clippy") → Lint OK
  │   │   │   └── shell("cargo test") → All tests pass
  │   │   └── Verdict: PASS (0.95)
  │   │
  │   ├── execute(Step 3: Write tests)
  │   │   └── [similar verification flow]
  │   │
  │   └── execute(Step 4: Final verify)
  │       └── [CriticAgent review]
  │
  ├── [CriticAgent]
  │   ├── review(task, results) → "All criteria satisfied"
  │   └── Confidence: 0.98
  │
  ├── Event: TaskCompleted
  │
  └── FinalOutput { output, verdicts, attempts: 1 }
```

### Multi-Agent Collaboration Flow

```
PlannerAgent
  │  decompose(task)
  │
  ├── SubTask A ────────────────────────────────────────────────┐
  │   └── ExecutorAgent A1 ── VerifierAgent A1 ── (retry) ── ok │
  │                                                              │
  ├── SubTask B (parallel group) ───────────────────────────────┐│
  │   ├── ExecutorAgent B1 ── VerifierAgent B1 ── (retry) ── ok ││
  │   └── ExecutorAgent B2 ── VerifierAgent B2 ── (retry) ── ok ││
  │                                                              ││
  ├── SubTask C (depends on A + B) ────────────────────────────┐││
  │   └── ExecutorAgent C1 ── VerifierAgent C1 ── (retry) ── ok ││
  │                                                              ▼▼
  └── CriticAgent.review(all_results) ───── TaskCompleted ◄──────┘
```

### Error Recovery Flow

```
Step Execution
  │
  ├── Success → [Verifier]
  │                        │
  │                    [Gate Fail] ───→ [ReflectionLoop]
  │                                              │
  │                                         ┌─────┴──────┐
  │                                         │            │
  │                                    [retry < max]  [retry >= max]
  │                                         │            │
  │                                    [reflect +    [escalate
  │                                     re-execute]   to human]
  │                                         │            │
  │                                    [Verifier]    [Human reviews
  │                                         │          & provides
  │                                    [eventual     feedback or
  │                                     pass/fail]   override]
  │
  ├── ToolError → [RetryPolicy]
  │                    │
  │               ┌────┴────┐
  │               │         │
  │          [retry]  [alternative tool]
  │               │         │
  │          [execute]  [replan step]
  │
  ├── Timeout → [Checkpoint restore] → [retry with more time]
  │
  └── Crash → [On restart: load checkpoint] → [resume from last saved state]
```
