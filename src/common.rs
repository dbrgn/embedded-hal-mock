//! Common functionality used by the mock implementations.

use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex},
};

/// Generic mock implementation.
///
/// ⚠️ **Do not use this directly as end user! This is only a building
/// block for creating mocks.**
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
        let mut e = self.expected.lock().unwrap();
        *e = v;
    }

    /// Assert that all expectations on a given mock have been consumed.
    pub fn done(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_mock() {
        let expectations = [0u8, 1u8];
        let mut mock: Generic<u8> = Generic::new(&expectations);

        assert_eq!(mock.next(), Some(0u8));
        assert_eq!(mock.next(), Some(1u8));
        assert_eq!(mock.next(), None);

        mock.done();
    }
}
