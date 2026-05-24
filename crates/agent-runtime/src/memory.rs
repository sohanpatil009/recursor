use agent_core::error::MemoryError;
use agent_core::memory::MemoryStore;
use agent_core::types::{CheckpointState, MemoryEntry, Task, TaskId, TaskStatus};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct SqliteMemoryStore {
    pool: SqlitePool,
}

impl SqliteMemoryStore {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", path.as_ref().display()))
            .await
            .map_err(|e| MemoryError::StorageError(format!("Failed to connect: {}", e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_history (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                priority TEXT NOT NULL,
                status TEXT NOT NULL,
                max_retries INTEGER NOT NULL,
                timeout_seconds INTEGER NOT NULL,
                working_directory TEXT NOT NULL,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                parent_task TEXT
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Migration failed: {}", e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                attempt INTEGER NOT NULL,
                output TEXT NOT NULL DEFAULT '',
                verdict TEXT,
                reflection TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES task_history(id)
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Migration failed: {}", e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory_entries (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                importance REAL NOT NULL DEFAULT 0.5
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Migration failed: {}", e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                task_id TEXT PRIMARY KEY,
                step_index INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES task_history(id)
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Migration failed: {}", e)))?;

        Ok(Self { pool })
    }

    pub async fn save_task(&self, task: &Task) -> Result<(), MemoryError> {
        let status_str = format!("{:?}", task.status);
        sqlx::query(
            "INSERT OR REPLACE INTO task_history
            (id, title, description, priority, status, max_retries, timeout_seconds, working_directory, created_at, tags, parent_task)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&task.id.0)
        .bind(&task.title)
        .bind(&task.description)
        .bind(format!("{:?}", task.priority))
        .bind(&status_str)
        .bind(task.max_retries as i64)
        .bind(task.timeout_seconds as i64)
        .bind(&task.context.working_directory)
        .bind(task.created_at.to_rfc3339())
        .bind(serde_json::to_string(&task.tags).unwrap_or_default())
        .bind(task.parent_task.as_ref().map(|t| t.0.clone()))
        .execute(&self.pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Save task failed: {}", e)))?;
        Ok(())
    }

    pub async fn load_task(&self, id: &TaskId) -> Result<Option<Task>, MemoryError> {
        type TaskRow = (
            String,
            String,
            String,
            String,
            String,
            i64,
            i64,
            String,
            String,
            String,
            Option<String>,
        );
        let row: Option<TaskRow> =
            sqlx::query_as(
                "SELECT id, title, description, priority, status, max_retries, timeout_seconds, working_directory, created_at, tags, parent_task FROM task_history WHERE id = ?"
            )
            .bind(&id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Load task failed: {}", e)))?;

        match row {
            Some((
                _id,
                title,
                description,
                _priority,
                _status,
                max_retries,
                timeout_seconds,
                wd,
                created_at,
                tags,
                parent_task,
            )) => {
                let tags: Vec<String> = serde_json::from_str(&tags).unwrap_or_default();
                Ok(Some(Task {
                    id: id.clone(),
                    title,
                    description,
                    priority: agent_core::types::Priority::Normal,
                    status: TaskStatus::Pending,
                    max_retries: max_retries as usize,
                    timeout_seconds: timeout_seconds as u64,
                    criteria: vec![],
                    context: agent_core::types::Context::new(&wd),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    parent_task: parent_task.map(TaskId),
                    subtasks: vec![],
                    tags,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn save_checkpoint(&self, state: &CheckpointState) -> Result<(), MemoryError> {
        let json = serde_json::to_string(state)
            .map_err(|e| MemoryError::StorageError(format!("Serialize checkpoint failed: {}", e)))?;
        sqlx::query(
            "INSERT OR REPLACE INTO checkpoints (task_id, step_index, state_json, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&state.task_id.0)
        .bind(state.step_index as i64)
        .bind(&json)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Save checkpoint failed: {}", e)))?;
        Ok(())
    }

    pub async fn load_checkpoint(&self, task_id: &TaskId) -> Result<Option<CheckpointState>, MemoryError> {
        let row: Option<(String, i64, String)> =
            sqlx::query_as("SELECT task_id, step_index, state_json FROM checkpoints WHERE task_id = ?")
                .bind(&task_id.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| MemoryError::StorageError(format!("Load checkpoint failed: {}", e)))?;

        match row {
            Some((_tid, _step, json)) => {
                let state: CheckpointState = serde_json::from_str(&json)
                    .map_err(|e| MemoryError::StorageError(format!("Deserialize checkpoint failed: {}", e)))?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_checkpoint(&self, task_id: &TaskId) -> Result<(), MemoryError> {
        sqlx::query("DELETE FROM checkpoints WHERE task_id = ?")
            .bind(&task_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Delete checkpoint failed: {}", e)))?;
        Ok(())
    }

    pub async fn log_step(
        &self,
        task_id: &TaskId,
        step_index: usize,
        attempt: usize,
        output: &str,
        verdict: Option<&agent_core::types::Verdict>,
        reflection: Option<&agent_core::types::Reflection>,
    ) -> Result<(), MemoryError> {
        let verdict_json = verdict.map(|v| serde_json::to_string(v).unwrap_or_default());
        let reflection_json = reflection.map(|r| serde_json::to_string(r).unwrap_or_default());
        sqlx::query(
            "INSERT INTO task_logs (task_id, step_index, attempt, output, verdict, reflection, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&task_id.0)
        .bind(step_index as i64)
        .bind(attempt as i64)
        .bind(output)
        .bind(verdict_json)
        .bind(reflection_json)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Log step failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn store(&self, key: &str, value: Value) -> Result<(), MemoryError> {
        let json =
            serde_json::to_string(&value).map_err(|e| MemoryError::StorageError(format!("Serialize failed: {}", e)))?;
        sqlx::query("INSERT OR REPLACE INTO memory_entries (key, value, created_at, importance) VALUES (?, ?, ?, ?)")
            .bind(key)
            .bind(&json)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(0.5f64)
            .execute(&self.pool)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Store failed: {}", e)))?;
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Value>, MemoryError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM memory_entries WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Retrieve failed: {}", e)))?;
        match row {
            Some((json,)) => {
                let value: Value = serde_json::from_str(&json)
                    .map_err(|e| MemoryError::StorageError(format!("Deserialize failed: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        let pattern = format!("%{}%", query);
        let rows: Vec<(String, String, String, f64)> = sqlx::query_as(
            "SELECT key, value, created_at, importance FROM memory_entries WHERE key LIKE ? OR value LIKE ? ORDER BY importance DESC LIMIT ?"
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Search failed: {}", e)))?;

        let mut entries = Vec::new();
        for (key, value_json, created_at, importance) in rows {
            let value: Value = serde_json::from_str(&value_json).unwrap_or(Value::Null);
            entries.push(MemoryEntry {
                key,
                value,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map(|d| d.to_utc())
                    .unwrap_or_else(|_| chrono::Utc::now()),
                importance,
            });
        }
        Ok(entries)
    }

    async fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        let rows: Vec<(String, String, String, f64)> = sqlx::query_as(
            "SELECT key, value, created_at, importance FROM memory_entries ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| MemoryError::StorageError(format!("Recent failed: {}", e)))?;

        let mut entries = Vec::new();
        for (key, value_json, created_at, importance) in rows {
            let value: Value = serde_json::from_str(&value_json).unwrap_or(Value::Null);
            entries.push(MemoryEntry {
                key,
                value,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map(|d| d.to_utc())
                    .unwrap_or_else(|_| chrono::Utc::now()),
                importance,
            });
        }
        Ok(entries)
    }

    async fn remove(&self, key: &str) -> Result<(), MemoryError> {
        sqlx::query("DELETE FROM memory_entries WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Delete failed: {}", e)))?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), MemoryError> {
        sqlx::query("DELETE FROM memory_entries")
            .execute(&self.pool)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Clear failed: {}", e)))?;
        Ok(())
    }
}
