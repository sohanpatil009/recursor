use async_trait::async_trait;

use crate::error::MemoryError;
use crate::types::MemoryEntry;

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store(&self, key: &str, value: serde_json::Value) -> Result<(), MemoryError>;
    async fn retrieve(&self, key: &str) -> Result<Option<serde_json::Value>, MemoryError>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;
    async fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;
    async fn remove(&self, key: &str) -> Result<(), MemoryError>;
    async fn clear(&self) -> Result<(), MemoryError>;
}
