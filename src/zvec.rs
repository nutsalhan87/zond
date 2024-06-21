//! [`Vec`]'s analogue with collecting statistics and all corresponding types, structs, traits, etc.

use std::{
    collections::TryReserveError,
    mem::MaybeUninit,
    ops::{Bound, Deref, RangeBounds},
    vec::{Drain, Splice},
};

use crate::{OperationType, Zond, ZondCollection};

/// Describes [`ZVec`]'s operation types or, in other words, called methods.
#[derive(Debug, Clone)]
pub enum ZVecOperation<T: Clone> {
    New,
    WithCapacity {
        capacity: usize,
    },
    FromRawParts {
        ptr: *mut T,
        length: usize,
        capacity: usize,
    },
    Capacity,
    Reserve {
        additional: usize,
    },
    ReserveExact {
        additional: usize,
    },
    TryReserve {
        additional: usize,
    },
    TryReserveExact {
        additional: usize,
    },
    ShrinkToFit,
    ShrinkTo {
        min_capacity: usize,
    },
    IntoBoxedSlice,
    Truncate {
        len: usize,
    },
    AsSlice,
    AsMutSlice,
    AsPtr,
    AsMutPtr,
    SetLen {
        new_len: usize,
    },
    SwapRemove {
        index: usize,
    },
    Insert {
        index: usize,
        element: T,
    },
    Remove {
        index: usize,
    },
    Retain,
    RetainMut,
    DedupByKey,
    DedupBy,
    Push {
        value: T,
    },
    Pop,
    Append {
        other: Vec<T>,
    },
    Drain {
        start_bound: Bound<usize>,
        end_bound: Bound<usize>,
    },
    Clear,
    Len,
    IsEmpty,
    SplitOff {
        at: usize,
    },
    ResizeWith {
        new_len: usize,
    },
    Leak,
    SpareCapacityMut,
    Resize {
        new_len: usize,
        value: T,
    },
    ExtendFromSlice {
        other: Vec<T>,
    },
    ExtendFromWithin {
        src_start_bound: Bound<usize>,
        src_end_bound: Bound<usize>,
    },
    Dedup,
    Splice {
        start_bound: Bound<usize>,
        end_bound: Bound<usize>,
    },
    Deref,
    IntoVec,
    FromVec {
        from: Vec<T>,
    },
}

impl<T: Clone> OperationType for ZVecOperation<T> {}

/// `ZVec` is a wrapper around [`Vec`] providing collecting statistics about operations.
///
/// *Attention*. Many `Vec`'s methods are avaliable via an implicit deref() call. So when you call them, only [`Deref`](ZVecOperation::Deref) saved.\
/// Later I'll implement wrapper around slice for collecting its operations.
pub struct ZVec<T: Clone> {
    inner: Vec<T>,
    zond_collection: ZondCollection<ZVecOperation<T>>,
}

impl<T: Clone> ZVec<T> {
    /// Creates `Zvec` from existing `Vec` instance.
    pub fn from_vec(from: Vec<T>, zond: Zond<ZVecOperation<T>>) -> Self {
        let zvec = Self {
            inner: from,
            zond_collection: ZondCollection::new(zond),
        };
        zvec.zond_collection.push_operation(ZVecOperation::FromVec {
            from: zvec.inner.clone(),
        });
        zvec
    }

    pub fn new(zond: Zond<ZVecOperation<T>>) -> Self {
        let zvec = Self {
            inner: Vec::new(),
            zond_collection: ZondCollection::new(zond),
        };
        zvec.zond_collection.push_operation(ZVecOperation::New);
        zvec
    }

    pub fn with_capacity(capacity: usize, zond: Zond<ZVecOperation<T>>) -> Self {
        let zvec = Self {
            inner: Vec::with_capacity(capacity),
            zond_collection: ZondCollection::new(zond),
        };
        zvec.zond_collection
            .push_operation(ZVecOperation::WithCapacity { capacity });
        zvec
    }

    pub unsafe fn from_raw_parts(
        ptr: *mut T,
        length: usize,
        capacity: usize,
        zond: Zond<ZVecOperation<T>>,
    ) -> Self {
        let zvec = Self {
            inner: Vec::from_raw_parts(ptr, length, capacity),
            zond_collection: ZondCollection::new(zond),
        };
        zvec.zond_collection
            .push_operation(ZVecOperation::FromRawParts {
                ptr,
                length,
                capacity,
            });
        zvec
    }

    pub fn capacity(&self) -> usize {
        self.zond_collection.push_operation(ZVecOperation::Capacity);
        self.inner.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.zond_collection
            .push_operation(ZVecOperation::Reserve { additional });
        self.inner.reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.zond_collection
            .push_operation(ZVecOperation::ReserveExact { additional });
        self.inner.reserve_exact(additional)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.zond_collection
            .push_operation(ZVecOperation::TryReserve { additional });
        self.inner.try_reserve(additional)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.zond_collection
            .push_operation(ZVecOperation::TryReserveExact { additional });
        self.inner.try_reserve_exact(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.zond_collection
            .push_operation(ZVecOperation::ShrinkToFit);
        self.inner.shrink_to_fit()
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.zond_collection
            .push_operation(ZVecOperation::ShrinkTo { min_capacity });
        self.inner.shrink_to(min_capacity)
    }

    pub fn into_boxed_slice(self) -> Box<[T]> {
        let ZVec {
            inner,
            zond_collection,
        } = self;
        zond_collection.push_operation(ZVecOperation::IntoBoxedSlice);
        inner.into_boxed_slice()
    }

    pub fn truncate(&mut self, len: usize) {
        self.zond_collection
            .push_operation(ZVecOperation::Truncate { len });
        self.inner.truncate(len)
    }

    pub fn as_slice(&self) -> &[T] {
        self.zond_collection.push_operation(ZVecOperation::AsSlice);
        self.inner.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.zond_collection
            .push_operation(ZVecOperation::AsMutSlice);
        self.inner.as_mut_slice()
    }

    pub fn as_ptr(&self) -> *const T {
        self.zond_collection.push_operation(ZVecOperation::AsPtr);
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.zond_collection.push_operation(ZVecOperation::AsMutPtr);
        self.inner.as_mut_ptr()
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.zond_collection
            .push_operation(ZVecOperation::SetLen { new_len });
        self.inner.set_len(new_len)
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        self.zond_collection
            .push_operation(ZVecOperation::SwapRemove { index });
        self.inner.swap_remove(index)
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.zond_collection.push_operation(ZVecOperation::Insert {
            index,
            element: element.clone(),
        });
        self.inner.insert(index, element)
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.zond_collection
            .push_operation(ZVecOperation::Remove { index });
        self.inner.remove(index)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.zond_collection.push_operation(ZVecOperation::Retain);
        self.inner.retain(f)
    }

    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.zond_collection
            .push_operation(ZVecOperation::RetainMut);
        self.inner.retain_mut(f)
    }

    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.zond_collection
            .push_operation(ZVecOperation::DedupByKey);
        self.inner.dedup_by_key(key)
    }

    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.zond_collection.push_operation(ZVecOperation::DedupBy);
        self.inner.dedup_by(same_bucket)
    }

    pub fn push(&mut self, value: T) {
        self.zond_collection.push_operation(ZVecOperation::Push {
            value: value.clone(),
        });
        self.inner.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.zond_collection.push_operation(ZVecOperation::Pop);
        self.inner.pop()
    }

    pub fn append(&mut self, other: &mut Vec<T>) {
        self.zond_collection.push_operation(ZVecOperation::Append {
            other: other.clone(),
        });
        self.inner.append(other)
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        self.zond_collection.push_operation(ZVecOperation::Drain {
            start_bound: range.start_bound().cloned(),
            end_bound: range.end_bound().cloned(),
        });
        self.inner.drain(range)
    }

    pub fn clear(&mut self) {
        self.zond_collection.push_operation(ZVecOperation::Clear);
        self.inner.clear()
    }

    pub fn len(&self) -> usize {
        self.zond_collection.push_operation(ZVecOperation::Len);
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.zond_collection.push_operation(ZVecOperation::IsEmpty);
        self.inner.is_empty()
    }

    pub fn split_off(&mut self, at: usize) -> Vec<T> {
        self.zond_collection
            .push_operation(ZVecOperation::SplitOff { at });
        self.inner.split_off(at)
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.zond_collection
            .push_operation(ZVecOperation::ResizeWith { new_len });
        self.inner.resize_with(new_len, f)
    }

    pub fn leak<'a>(self) -> &'a mut [T] {
        let ZVec {
            inner,
            zond_collection,
        } = self;
        zond_collection.push_operation(ZVecOperation::Leak);
        inner.leak()
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.zond_collection
            .push_operation(ZVecOperation::SpareCapacityMut);
        self.inner.spare_capacity_mut()
    }

    pub fn resize(&mut self, new_len: usize, value: T) {
        self.zond_collection.push_operation(ZVecOperation::Resize {
            new_len,
            value: value.clone(),
        });
        self.inner.resize(new_len, value)
    }

    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.zond_collection
            .push_operation(ZVecOperation::ExtendFromSlice {
                other: other.to_vec(),
            });
        self.inner.extend_from_slice(other)
    }

    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
    {
        self.zond_collection
            .push_operation(ZVecOperation::ExtendFromWithin {
                src_start_bound: src.start_bound().cloned(),
                src_end_bound: src.end_bound().cloned(),
            });
        self.inner.extend_from_within(src)
    }

    pub fn splice<I, R>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> Splice<'_, <I as IntoIterator>::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        self.zond_collection.push_operation(ZVecOperation::Splice {
            start_bound: range.start_bound().cloned(),
            end_bound: range.end_bound().cloned(),
        });
        self.inner.splice(range, replace_with)
    }
}

impl<T> ZVec<T>
where
    T: Clone + PartialEq,
{
    pub fn dedup(&mut self) {
        self.zond_collection.push_operation(ZVecOperation::Dedup);
        self.inner.dedup()
    }
}

impl<T: Clone> Deref for ZVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.zond_collection.push_operation(ZVecOperation::Deref);
        self.inner.deref()
    }
}

impl<T: Clone> From<ZVec<T>> for Vec<T> {
    fn from(zvec: ZVec<T>) -> Vec<T> {
        let ZVec {
            inner,
            zond_collection,
        } = zvec;
        zond_collection.push_operation(ZVecOperation::IntoVec);
        inner
    }
}
