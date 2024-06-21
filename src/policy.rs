//! Module contains [`Policy`] struct with its variants and its variants' metadata.

use std::{
    cell::RefCell,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

pub(crate) enum PolicyInner {
    // Operations will be handled only on collection's drop.
    OnDropOnly,
    // Operatons will be handled each N method call.
    OnCountOperations {
        max_operations: usize,
        current_operations: RefCell<usize>,
    },
    // Operations will be handled at first method call that happened after given period since last handling.
    LessOften {
        duration: Duration,
        last_collect: RefCell<Instant>,
    },
}

impl Clone for PolicyInner {
    fn clone(&self) -> Self {
        match self {
            Self::OnDropOnly => Self::OnDropOnly,
            Self::OnCountOperations { max_operations, .. } => Self::OnCountOperations {
                max_operations: *max_operations,
                current_operations: RefCell::new(0),
            },
            Self::LessOften { duration, .. } => Self::LessOften {
                duration: *duration,
                last_collect: RefCell::new(Instant::now()),
            },
        }
    }
}

/// Desribes rules when collected operations will be handled.
#[derive(Clone)]
pub struct Policy {
    pub(crate) inner: PolicyInner,
}

impl Policy {
    /// Operations will be handled only on collection's drop.
    pub fn on_drop_only() -> Self {
        Self {
            inner: PolicyInner::OnDropOnly,
        }
    }

    /// Operatons will be handled each N method call.
    pub fn on_count_operations(max_operations: NonZeroUsize) -> Self {
        Self {
            inner: PolicyInner::OnCountOperations {
                max_operations: max_operations.get(),
                current_operations: RefCell::new(0),
            },
        }
    }

    /// Operations will be handled at first method call that happened after given period since last handling.
    pub fn less_often(duration: Duration) -> Self {
        Self {
            inner: PolicyInner::LessOften {
                duration,
                last_collect: RefCell::new(Instant::now()),
            },
        }
    }
}
