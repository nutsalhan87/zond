//! Zond is crate with standard rust collections but with collecting statistics.
//!
//! Ok, maybe it contains only analogue of [`Vec`] - [`zvec::ZVec`]. And ok, `ZVec` contains only some part of `Vec` methods.
//! But I made this just for fun. I don't know anyone who would really need this.
//!
//! # Example
//!
//! Let's start from constructing collection.
//!
//! Constructors similar to their std analogues' constructors but have additional argument - struct [`Zond`] with two fields:
//! 1. `zond_handler` of type [`ZondHandler`]. \
//!  Trait object with single method that consumes two arguments: `id` as [`usize`] and `operations` as [`Operations`].
//!  All operations handling is hapeppening here: you can save them to file or database, send to your server or just print to console.
//! 2. `policy` of type [`Policy`]. \
//!  Desribes the rules about when collected operations will handled by `zond_handler`.
//! ```
//! # use std::{fmt::Debug, num::NonZeroUsize};
//! # use zond::{OperationType, ZondHandler, Operations, Zond, Policy, zvec::{ZVec, ZVecOperation}};
//! // So at first let's implement some ZondHandler. It will just print operations to stdout:
//! struct HandlerImpl;
//!
//! impl<T: OperationType + Debug> ZondHandler<T> for HandlerImpl {
//!     fn handle(&self, id: usize, operations: Operations<T>) {
//!         println!("{id} collected");
//!         operations
//!             .iter()
//!             .for_each(|v| println!("{:?}: {:?}", v.get_instant(), v.get_type()));
//!         println!();
//!     }
//! }
//!
//! # fn main() {
//! // Next let's construct Zond with HandlerImpl handler and such a policy that operations will be handled after each three method calls.
//! // It will handle operations for ZVec:
//! let zond: Zond<ZVecOperation<usize>> = Zond::new(
//!     HandlerImpl,
//!     Policy::on_count_operations(NonZeroUsize::new(3).unwrap()),
//! );
//!
//! // Next let's construct ZVec with zond variable:
//! let mut zvec: ZVec<usize> = ZVec::new(zond);
//!
//! // Finally let's execute some operations:
//! zvec.push(1);
//! zvec.push(2);
//! zvec.push(5);
//! zvec.push(5);
//! zvec.extend_from_within(1..);
//! zvec.dedup();
//! drop(zvec);
//! # }
//! ```
//!
//! The console output will look like this:
//! ```text
//! 0 collected
//! Instant { /* */ }: New
//! Instant { /* */ }: Push { value: 1 }
//! Instant { /* */ }: Push { value: 2 }
//!
//! 0 collected
//! Instant { /* */ }: Push { value: 5 }
//! Instant { /* */ }: Push { value: 5 }
//! Instant { /* */ }: ExtendFromWithin { src_start_bound: Included(1), src_end_bound: Unbounded }
//!
//! 0 collected
//! Instant { /* */ }: Dedup
//! ```
//!
//! As you can see, operations always being handled when dropping.

use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

pub use policy::Policy;
use policy::PolicyInner;

mod policy;
pub mod zvec;

static ID_GENERATOR: AtomicUsize = AtomicUsize::new(0);

/// Helper trait for constrainting generic types in other structs and traits. \
/// `OperationType` unites multiple collection-specific enums. But in fact only [`ZVec`](zvec::ZVec)'s [`ZVecOperation`](zvec::ZVecOperation).
pub trait OperationType {}

/// Describes one single operation with collection: time when it happened and operation type.
pub struct Operation<T: OperationType> {
    instant: Instant,
    operation_type: T,
}

impl<T: OperationType> Operation<T> {
    /// Constructs `Operation` with current time and given operation type.
    ///
    /// # Example
    /// ```
    /// # use zond::{Operation, zvec::ZVecOperation};
    /// # fn main() {
    /// let operation: Operation<ZVecOperation<usize>> = Operation::new(ZVecOperation::New);
    /// # }
    /// ```
    pub fn new(operation_type: T) -> Self {
        Self {
            instant: Instant::now(),
            operation_type,
        }
    }

    /// Get time when operation happened.
    pub fn get_instant(&self) -> &Instant {
        &self.instant
    }

    /// Get operation type.
    pub fn get_type(&self) -> &T {
        &self.operation_type
    }
}

/// Just type alias for more convenient types declaring in other places.
pub type Operations<T> = Vec<Operation<T>>;

/// Provides function that handle all operations with collection.
///
/// # Example
/// ```no_run
/// # use std::fmt::Debug;
/// # use zond::{ZondHandler, Operations, OperationType};
/// struct HandlerImpl;
///
/// impl<T: OperationType + Debug> ZondHandler<T> for HandlerImpl {
///     fn handle(&self, id: usize, operations: Operations<T>) {
///         for operation in operations {
///             println!("{:?}", operation.get_type());
///         }
///     }
/// }
/// ```
pub trait ZondHandler<T: OperationType> {
    /// `id` is used to distinguish between different collection instances' operations.
    ///
    /// `operations` is just operations.
    fn handle(&self, id: usize, operations: Operations<T>);
}

/// Struct that controls how and when to handle operations.
#[derive(Clone)]
pub struct Zond<T: OperationType> {
    zond_handler: Arc<dyn ZondHandler<T>>,
    policy: Policy,
}

impl<T: OperationType> Zond<T> {
    /// Constructs a new `Zond<T>`
    ///
    /// # Example
    /// ```
    /// # use std::fmt::Debug;
    /// # use zond::{ZondHandler, Operations, OperationType, Zond, Policy, zvec::ZVecOperation};
    /// struct HandlerImpl;
    ///
    /// impl<T: OperationType + Debug> ZondHandler<T> for HandlerImpl {
    ///     fn handle(&self, id: usize, operations: Operations<T>) {
    ///         for operation in operations {
    ///             println!("{:?}", operation.get_type());
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let zond: Zond<ZVecOperation<usize>> = Zond::new(HandlerImpl, Policy::on_drop_only());        
    /// }
    /// ```
    pub fn new(zond_handler: impl ZondHandler<T> + 'static, policy: Policy) -> Self {
        Self {
            zond_handler: Arc::new(zond_handler),
            policy,
        }
    }
}

// Crucial part of the crate. This struct contains all other structs, trait object and enums that take part in storing and handling operations. \
// Must be aggregated in structs that implement some collection's functionality.
pub(crate) struct ZondCollection<T: OperationType> {
    id: usize,
    operations: RefCell<Operations<T>>,
    zond: Zond<T>,
}

impl<T: OperationType> ZondCollection<T> {
    pub(crate) fn new(zond: Zond<T>) -> Self {
        Self {
            id: ID_GENERATOR.fetch_add(1, Ordering::Relaxed),
            operations: RefCell::default(),
            zond,
        }
    }

    // Force handle collected operations.
    pub(crate) fn handle(&self) {
        let operations = self.operations.replace(Vec::new());
        self.zond.zond_handler.handle(self.id, operations);
    }

    // Check handling policy and, if accordingly to them operations should be handled, handle operations.
    pub(crate) fn try_handle(&self) {
        match &self.zond.policy.inner {
            PolicyInner::OnCountOperations {
                current_operations,
                max_operations,
            } => {
                let mut current_operations = current_operations.borrow_mut();
                if max_operations - 1 == *current_operations {
                    *current_operations = 0;
                    self.handle()
                } else {
                    *current_operations += 1;
                }
            }
            PolicyInner::LessOften {
                duration,
                last_collect,
            } => {
                let mut last_collect = last_collect.borrow_mut();
                let now = Instant::now();
                if now.duration_since(*last_collect) > *duration {
                    *last_collect = now;
                    self.handle()
                }
            }
            PolicyInner::OnDropOnly => (),
        }
    }

    // Push single operation to store and handle all of them if they should be handled.
    pub(crate) fn push_operation(&self, operation: T) {
        self.operations.borrow_mut().push(Operation::new(operation));
        self.try_handle();
    }
}

impl<T: OperationType> Drop for ZondCollection<T> {
    fn drop(&mut self) {
        self.handle();
    }
}
