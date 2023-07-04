//! Fast access to a thread-identifying value, [`ThrId`]. See its documentation
//! for details, as there are some caveats compared to [`std::thread::ThreadId`].
//!
//! # Usage
//!
//! The entry-point to this library is [`ThrId::get`].
//!
//! ```
//! let main_thread_id = thrid::ThrId::get();
//! let other_thread_id = std::thread::spawn(|| thrid::ThrId::get()).join().unwrap();
//!
//! assert_ne!(main_thread_id, other_thread_id);
//! ```

#![no_std] // But note that on some targets, we will pull in std anyway.

use core::num::NonZeroUsize;

mod imp;

/// A value that uniquely identifies a running thread.
///
/// This returns a thread-specific value that is not shared among any currently
/// running threads.
///
/// Typically, it will be a pointer to some low level thread-specific data
/// structure, such as the thread control block, but on some platforms it may be
/// implemented in a number of other ways.
///
/// # Caveats
///
/// There are a few caveats. The TL;DR is that the OS may recycle these values
/// (similar to system thread IDs), and the internal impl is subject to change.
///
/// - Similar to OS thread IDs (but unlike Rust's `std::thread::ThreadId`),
///   these may be reused by a new thread if some another thread terminates.
///   Concretely, if `tid0` and `tid1` are [`ThrId`]s from two different calls
///   to [`ThrId::get`], then:
///      - `tid0 != tid1` will be true if they are from different threads.
///
///      - `tid0 == tid1` will be true if *either* they are from the same
///        thread, or they are two unrelated threads which have never been alive
///        at the same time (this means that at least one of them is from a
///        thread which is no longer running).
///
/// - ThrId values only have meaning locally within a process. They are
///   meaningless in cross-process contexts.
///
/// - The internal implementation of [`ThrId::get()`] is subject to change, do
///   not rely on an `ThrId`'s value having any external meaning (you may want
///   to use the `thread-id` crate if you need this).
///
///     For example, while `ThrId` values may happen to hold the same value as
///     returned by `pthread_self()` or `GetCurrentThreadId()` value on certain
///     targets (ones where a more efficient implementaion is not yet
///     available), this must not be relied upon -- it is entirely subject to
///     change.
///
/// - As such, is generally wrong to compare values of `thrid::ThrId` from two
///   different versions of this crate. Part of the reason that the [`ThrId`]
///   wrapper exists is to help enforce that.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThrId(NonZeroUsize);

impl ThrId {
    /// Get the value for the current thread.
    ///
    /// This will be unique across all running threads, but note that the `ThrId`
    /// for a given thread may be reused after that thread terminates.
    #[inline(always)]
    pub fn get() -> Self {
        Self(imp::tid_impl())
    }

    /// Return the underlying integer value of the [`ThrId`]
    #[inline]
    pub fn value(self) -> NonZeroUsize {
        self.0
    }
}

impl AsRef<NonZeroUsize> for ThrId {
    #[inline]
    fn as_ref(&self) -> &NonZeroUsize {
        &self.0
    }
}

impl core::borrow::Borrow<NonZeroUsize> for ThrId {
    #[inline]
    fn borrow(&self) -> &NonZeroUsize {
        &self.0
    }
}

impl From<ThrId> for usize {
    #[inline]
    fn from(id: ThrId) -> usize {
        id.0.get()
    }
}

impl From<ThrId> for NonZeroUsize {
    #[inline]
    fn from(id: ThrId) -> NonZeroUsize {
        id.0
    }
}

impl PartialEq<NonZeroUsize> for ThrId {
    #[inline]
    fn eq(&self, rhs: &NonZeroUsize) -> bool {
        self.0 == *rhs
    }
}

impl PartialEq<ThrId> for NonZeroUsize {
    #[inline]
    fn eq(&self, rhs: &ThrId) -> bool {
        *self == rhs.0
    }
}

impl PartialEq<usize> for ThrId {
    #[inline]
    fn eq(&self, rhs: &usize) -> bool {
        self.0.get() == *rhs
    }
}

impl PartialEq<ThrId> for usize {
    #[inline]
    fn eq(&self, rhs: &ThrId) -> bool {
        *self == rhs.0.get()
    }
}
