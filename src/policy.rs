//! Module contains [`Policy`] enum and its variants' metadata structs

use std::{
    cell::RefCell,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

/// Desribes rules when collected operations will be handled
pub enum Policy {
    /// Operations will be handled only on collection's drop
    OnDropOnly,
    /// Operatons will be handled each N method call
    OnCountOperations(OnCountOperationsMetadata),
    /// Operations will be handled at first method call that happened after given period since last handling
    LessOften(LessOftenMetadata),
}

pub struct OnCountOperationsMetadata {
    pub max_operations: usize,
    pub(crate) current_operations: RefCell<usize>,
}

impl OnCountOperationsMetadata {
    pub fn new(max_operations: NonZeroUsize) -> Self {
        Self {
            max_operations: max_operations.get(),
            current_operations: RefCell::new(0),
        }
    }
}

pub struct LessOftenMetadata {
    pub duration: Duration,
    pub(crate) last_collect: RefCell<Instant>,
}

impl LessOftenMetadata {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            last_collect: RefCell::new(Instant::now()),
        }
    }
}
