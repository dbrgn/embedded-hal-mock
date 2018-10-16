//! Common functionality used by the mock implementations.

use std::sync::{Mutex, Arc};

/// Generic Mock implementation
///
/// This supports the specification and evaluation of expectations to allow automated testing of hal drivers.
/// Mismatches between expectations will cause runtime assertions to assist in locating the source of the fault.
///
/// Note that the implementation uses an `Arc<Mutex<...>>` internally, so a
/// cloned instance of the mock can be used to check the expectations of the
/// original instance that has been moved into a driver.
#[derive(Debug)]
pub struct Generic<'a, T: 'a> {
    expected: Arc<Mutex<(usize, &'a[T])>>,
}

impl <'a, T>Generic<'a, T> {
    /// Create a new mock interface
    ///
    /// This creates a new generic mock interface with initial expectations
    pub fn new(e: &'a [T]) -> Self {
        Generic {
            expected: Arc::new(Mutex::new((0, e))),
        }
    }

    /// Set expectations on the interface
    ///
    /// This is a list of transactions to be executed in order
    /// Note that setting this will overwrite any existing expectations
    pub fn expect(&mut self, expected: &'a [T]) {
        let mut e = self.expected.lock().unwrap();
        e.0 = 0;
        e.1 = expected.into();
    }

    /// Assert that all expectations on a given Mock have been met
    pub fn done(&mut self) {
        let e = self.expected.lock().unwrap();
        assert_eq!(e.0, e.1.len());
    }
}

/// Clone allows a single mock to be duplicated for control and evaluation
impl <'a, T>Clone for Generic<'a, T> {
    fn clone(&self) -> Self {
        Generic{ expected: self.expected.clone() }
    }
}

/// Iterator impl for use in mock impls
impl <'a, T>Iterator for Generic<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut e = self.expected.lock().unwrap();
        e.0 += 1;
        e.1.get(e.0-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_mock() {
        let expectations = [0u8, 1u8];
        let mut mock = Generic::new(&expectations);

        assert_eq!(mock.next(), Some(&0u8));
        assert_eq!(mock.next(), Some(&1u8));
        assert_eq!(mock.next(), None);
    }
}
