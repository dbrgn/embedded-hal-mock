//! Common functionality used by the mock implementations.

use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex},
    thread,
};

/// Generic mock implementation.
///
/// ⚠️ **Do not use this directly as end user! This is only a building block
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
#[derive(Debug)]
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

        g.expect(expected);

        g
    }

    /// Set expectations on the interface
    ///
    /// This is a list of transactions to be executed in order. Note that
    /// setting this will overwrite any existing expectations.
    pub fn expect<E>(&mut self, expected: E)
    where
        E: IntoIterator<Item = &'a T>,
    {
        let v: VecDeque<T> = expected.into_iter().cloned().collect();

        let mut expected = self.expected.lock().unwrap();
        let mut done_called = self.done_called.lock().unwrap();
        *expected = v;
        done_called.reset();
    }

    /// Assert that all expectations on a given mock have been consumed.
    pub fn done(&mut self) {
        self.done_called.lock().unwrap().mark_as_called();

        let e = self.expected.lock().unwrap();
        assert!(e.is_empty(), "Not all expectations consumed");
    }
}

/// Clone allows a single mock to be duplicated for control and evaluation
impl<T> Clone for Generic<T>
where
    T: Clone + Debug + PartialEq,
{
    fn clone(&self) -> Self {
        Generic {
            expected: self.expected.clone(),
            done_called: self.done_called.clone(),
        }
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
    /// triggered.
    pub(crate) fn mark_as_called(&mut self) {
        assert!(!self.called, "The `.done()` method was called twice!");
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

            // Panic. This probably results in an abort:
            // https://doc.rust-lang.org/std/ops/trait.Drop.html#panics
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
        #[should_panic]
        fn panic_if_drop_not_called() {
            let expectations = [0u8, 1u8];
            let mut mock: Generic<u8> = Generic::new(&expectations);
            assert_eq!(mock.next(), Some(0u8));
            assert_eq!(mock.next(), Some(1u8));
            // Note: done() not called
        }

        #[test]
        #[should_panic]
        fn panic_if_drop_called_twice() {
            let expectations = [0u8, 1u8];
            let mut mock: Generic<u8> = Generic::new(&expectations);
            assert_eq!(mock.next(), Some(0u8));
            assert_eq!(mock.next(), Some(1u8));
            mock.done();
            mock.done(); // This will panic
        }
    }
}
