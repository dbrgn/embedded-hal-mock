//! Common functionality used by the mock implementations.

use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex},
    thread,
};

/// Generic mock implementation.
///
/// ⚠️ **Do not create this directly as end user! This is only a building block
/// for creating mocks.**
///
/// This type supports the specification and evaluation of expectations to
/// allow automated testing of hal drivers. Mismatches between expectations
/// will cause runtime assertions to assist in locating the source of the
/// fault.
///
/// Note that the implementation uses an `Arc<Mutex<...>>` internally, so a
/// cloned instance of the mock can be used to check the expectations of the
/// original instance that has been moved into a driver.
#[derive(Debug, Clone)]
pub struct Generic<T: Clone + Debug + PartialEq> {
    expected: Arc<Mutex<VecDeque<T>>>,
    done_called: Arc<Mutex<DoneCallDetector>>,
}

impl<'a, T: 'a> Generic<T>
where
    T: Clone + Debug + PartialEq,
{
    /// Create a new mock interface
    ///
    /// This creates a new generic mock interface with initial expectations
    pub fn new<E>(expected: E) -> Generic<T>
    where
        E: IntoIterator<Item = &'a T>,
    {
        let mut g = Generic {
            expected: Arc::new(Mutex::new(VecDeque::new())),
            done_called: Arc::new(Mutex::new(DoneCallDetector::new())),
        };

        g.update_expectations(expected);

        g
    }

    /// Update expectations on the interface
    ///
    /// When this method is called, first it is ensured that existing
    /// expectations are all consumed by calling [`done()`](#method.done)
    /// internally (if not called already). Afterwards, the new expectations
    /// are set.
    pub fn update_expectations<E>(&mut self, expected: E)
    where
        E: IntoIterator<Item = &'a T>,
    {
        // Ensure that existing expectations are consumed
        self.done_impl(false);

        // Collect new expectations into vector
        let new_expectations: VecDeque<T> = expected.into_iter().cloned().collect();

        // Lock internal state
        let mut expected = self.expected.lock().unwrap();
        let mut done_called = self.done_called.lock().unwrap();

        // Update expectations
        *expected = new_expectations;

        // Reset done call detector
        done_called.reset();
    }

    /// Deprecated alias of `update_expectations`.
    #[deprecated(
        since = "0.10.0",
        note = "The method 'expect' was renamed to 'update_expectations'"
    )]
    pub fn expect<E>(&mut self, expected: E)
    where
        E: IntoIterator<Item = &'a T>,
    {
        self.update_expectations(expected)
    }

    /// Assert that all expectations on a given mock have been consumed.
    pub fn done(&mut self) {
        self.done_impl(true);
    }

    fn done_impl(&mut self, panic_if_already_done: bool) {
        self.done_called
            .lock()
            .unwrap()
            .mark_as_called(panic_if_already_done);
        let e = self.expected.lock().unwrap();
        assert!(e.is_empty(), "Not all expectations consumed");
    }
}

/// Iterator impl for use in mock impls
impl<T> Iterator for Generic<T>
where
    T: Clone + Debug + PartialEq,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.expected.lock().unwrap().pop_front()
    }
}

/// Struct used to detect whether or not the `.done()` method was called.
#[derive(Debug)]
pub(crate) struct DoneCallDetector {
    called: bool,
}

impl DoneCallDetector {
    pub(crate) fn new() -> Self {
        Self { called: false }
    }

    /// Mark the `.done()` method as called.
    ///
    /// Note: When calling this method twice, an assertion failure will be
    /// triggered if `panic_if_already_done` is true.
    pub(crate) fn mark_as_called(&mut self, panic_if_already_done: bool) {
        if panic_if_already_done {
            assert!(!self.called, "The `.done()` method was called twice!");
        }
        self.called = true;
    }

    /// Reset the detector.
    pub(crate) fn reset(&mut self) {
        self.called = false;
    }
}

impl Drop for DoneCallDetector {
    fn drop(&mut self) {
        // Ensure that the `.done()` method was called on the mock before
        // dropping.
        if !self.called && !thread::panicking() {
            let msg = "WARNING: A mock (from embedded-hal-mock) was dropped \
                       without calling the `.done()` method. \
                       See https://github.com/dbrgn/embedded-hal-mock/issues/34 \
                       for more details.";

            // Note: We cannot use the print macros here, since they get
            // captured by the Cargo test runner. Instead, write to stderr
            // directly.
            use std::io::Write;
            let mut stderr = std::io::stderr();
            stderr.write_all(b"\x1b[31m").ok();
            stderr.write_all(msg.as_bytes()).ok();
            stderr.write_all(b"\x1b[m\n").ok();
            stderr.flush().ok();

            // Panic!
            //
            // (Note: Inside a `Drop` implementation, panic should only be used
            // if not already panicking:
            // https://doc.rust-lang.org/std/ops/trait.Drop.html#panics
            // This is ensured by checking `!thread::panicking()`.)
            panic!("{}", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod generic_mock {
        use super::*;

        #[test]
        fn success() {
            let expectations = [0u8, 1u8];
            let mut mock: Generic<u8> = Generic::new(&expectations);

            assert_eq!(mock.next(), Some(0u8));
            assert_eq!(mock.next(), Some(1u8));
            assert_eq!(mock.next(), None);
            assert_eq!(mock.next(), None);

            mock.done();
        }

        #[test]
        #[should_panic(
            expected = "WARNING: A mock (from embedded-hal-mock) was dropped without calling the `.done()` method. See https://github.com/dbrgn/embedded-hal-mock/issues/34 for more details."
        )]
        fn panic_if_drop_not_called() {
            let expectations = [0u8, 1u8];
            let mut mock: Generic<u8> = Generic::new(&expectations);
            assert_eq!(mock.next(), Some(0u8));
            assert_eq!(mock.next(), Some(1u8));
        }

        #[test]
        #[should_panic(expected = "The `.done()` method was called twice!")]
        fn panic_if_drop_called_twice() {
            let expectations = [0u8, 1u8];
            let mut mock: Generic<u8> = Generic::new(&expectations);
            assert_eq!(mock.next(), Some(0u8));
            assert_eq!(mock.next(), Some(1u8));
            mock.done();
            mock.done();
        }
    }
}
