use async_trait::async_trait;

use crate::error::VerificationError;
use crate::types::{Criterion, StepResult, Verdict};

#[async_trait]
pub trait VerificationGate: Send + Sync {
    fn name(&self) -> &str;
    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, VerificationError>;
}

#[async_trait]
pub trait Verifier: Send + Sync {
    async fn verify(&self, result: &StepResult, criteria: &[Criterion]) -> Result<Verdict, VerificationError>;
}
