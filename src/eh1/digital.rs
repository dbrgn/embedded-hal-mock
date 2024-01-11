//! Mock digital [`InputPin`], [`OutputPin`], and [`StatefulOutputPin`] implementations
//! Also mock calls to [`Wait`], assuming the `embedded-hal-async` feature is enabled.
//!
//! [`InputPin`]: https://docs.rs/embedded-hal/1/embedded_hal/digital/trait.InputPin.html
//! [`OutputPin`]: https://docs.rs/embedded-hal/1/embedded_hal/digital/trait.OutputPin.html
//! [`StatefulOutputPin`]: https://docs.rs/embedded-hal/1/embedded_hal/digital/trait.StatefulOutputPin.html
//! [`Wait`]: https://docs.rs/embedded-hal-async/1/embedded_hal_async/digital/trait.Wait.html
//!
//! ```
//! # use eh1 as embedded_hal;
//! use std::io::ErrorKind;
//!
//! use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
//! use embedded_hal_mock::eh1::{
//!     digital::{Mock as PinMock, State as PinState, Transaction as PinTransaction},
//!     MockError,
//! };
//!
//! let err = MockError::Io(ErrorKind::NotConnected);
//!
//! // Configure expectations
//! let expectations = [
//!     PinTransaction::get(PinState::High),
//!     PinTransaction::get(PinState::High),
//!     PinTransaction::set(PinState::Low),
//!     PinTransaction::set(PinState::High).with_error(err.clone()),
//!     PinTransaction::get_state(PinState::High),
//!     PinTransaction::toggle(),
//!     PinTransaction::get_state(PinState::Low),
//! ];
//!
//! // Create pin
//! let mut pin = PinMock::new(&expectations);
//!
//! // Run and test
//! assert_eq!(pin.is_high().unwrap(), true);
//! assert_eq!(pin.is_low().unwrap(), false);
//!
//! pin.set_low().unwrap();
//! pin.set_high().expect_err("expected error return");
//!
//! pin.is_set_high().unwrap();
//! pin.toggle().unwrap();
//! pin.is_set_low().unwrap();
//!
//! pin.done();
//!
//! // Update expectations
//! pin.update_expectations(&[]);
//! // ...
//! pin.done();
//! ```

use eh1 as embedded_hal;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

use crate::{common::Generic, eh1::error::MockError};

/// MockPin transaction
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Transaction {
    /// Kind is the transaction kind (and data) expected
    kind: TransactionKind,
    /// An optional error return value for a transaction. This is in addition
    /// to `kind` to allow validation that the transaction kind is correct
    /// prior to returning the error.
    err: Option<MockError>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Digital pin value enumeration
pub enum State {
    /// Digital low state
    Low,
    /// Digital high state
    High,
}

#[cfg(feature = "embedded-hal-async")]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Digital pin edge enumeration
pub enum Edge {
    /// Digital rising edge
    Rising,
    /// Digital falling edge
    Falling,
    /// Either digital rising or falling edge
    Any,
}

impl Transaction {
    /// Create a new pin transaction
    pub fn new(kind: TransactionKind) -> Transaction {
        Transaction { kind, err: None }
    }

    /// Create a new get transaction
    pub fn get(state: State) -> Transaction {
        Transaction::new(TransactionKind::Get(state))
    }

    /// Create a new get transaction
    pub fn set(state: State) -> Transaction {
        Transaction::new(TransactionKind::Set(state))
    }

    /// Create a new toggle transaction
    pub fn toggle() -> Transaction {
        Transaction::new(TransactionKind::Toggle)
    }

    /// Create a new get stateful pin state transaction
    pub fn get_state(state: State) -> Transaction {
        Transaction::new(TransactionKind::GetState(state))
    }

    /// Create a new wait_for_state transaction
    #[cfg(feature = "embedded-hal-async")]
    pub fn wait_for_state(state: State) -> Transaction {
        Transaction::new(TransactionKind::WaitForState(state))
    }

    /// Crate a new wait_for_edge transaction
    #[cfg(feature = "embedded-hal-async")]
    pub fn wait_for_edge(edge: Edge) -> Transaction {
        Transaction::new(TransactionKind::WaitForEdge(edge))
    }

    /// Add an error return to a transaction
    ///
    /// This is used to mock failure behaviours.
    ///
    /// Note that this can only be used for methods which actually return a
    /// [`Result`]; trying to invoke this for others will lead to an assertion
    /// error!
    pub fn with_error(mut self, error: MockError) -> Self {
        assert!(
            self.kind.supports_errors(),
            "the transaction kind supports errors"
        );
        self.err = Some(error);
        self
    }
}

/// MockPin transaction kind.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TransactionKind {
    /// Set the pin state
    Set(State),
    /// Get the pin state
    Get(State),
    /// Toggle the pin state
    Toggle,
    /// Get the set state of the stateful pin
    GetState(State),
    /// Wait for the given pin state
    #[cfg(feature = "embedded-hal-async")]
    WaitForState(State),
    /// Wait for the given pin edge
    #[cfg(feature = "embedded-hal-async")]
    WaitForEdge(Edge),
}

impl TransactionKind {
    fn is_get(&self) -> bool {
        match self {
            TransactionKind::Get(_) => true,
            _ => false,
        }
    }

    /// Specifies whether the actual API returns a [`Result`] (= supports errors) or not.
    fn supports_errors(&self) -> bool {
        true
    }
}

/// Mock Pin implementation
pub type Mock = Generic<Transaction>;

impl ErrorType for Mock {
    type Error = MockError;
}

/// Single digital push-pull output pin
impl OutputPin for Mock {
    /// Drives the pin low
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let Transaction { kind, err } = self.next().expect("no expectation for pin::set_low call");

        assert_eq!(
            kind,
            TransactionKind::Set(State::Low),
            "expected pin::set_low"
        );

        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Drives the pin high
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let Transaction { kind, err } = self.next().expect("no expectation for pin::set_high call");

        assert_eq!(
            kind,
            TransactionKind::Set(State::High),
            "expected pin::set_high"
        );

        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

impl InputPin for Mock {
    /// Is the input pin high?
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for pin::is_high call");

        assert!(kind.is_get(), "expected pin::get");

        if let Some(e) = err {
            Err(e)
        } else if let TransactionKind::Get(v) = kind {
            Ok(v == State::High)
        } else {
            unreachable!();
        }
    }

    /// Is the input pin low?
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for pin::is_low call");

        assert!(kind.is_get(), "expected pin::get");

        if let Some(e) = err {
            Err(e)
        } else if let TransactionKind::Get(v) = kind {
            Ok(v == State::Low)
        } else {
            unreachable!();
        }
    }
}

/// Single digital output pin that remembers its state and can be toggled between high and low states
impl StatefulOutputPin for Mock {
    /// Toggle the pin low to high or high to low
    fn toggle(&mut self) -> Result<(), Self::Error> {
        let Transaction { kind, err } = self.next().expect("no expectation for pin::toggle call");

        assert_eq!(kind, TransactionKind::Toggle, "expected pin::toggle");

        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Is the output pin set high?
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for pin::is_set_high call");

        assert!(
            matches!(kind, TransactionKind::GetState(_)),
            "expected pin::is_set_high"
        );

        if let Some(e) = err {
            Err(e)
        } else if let TransactionKind::GetState(v) = kind {
            Ok(v == State::High)
        } else {
            unreachable!();
        }
    }

    /// Is the output pin set low?
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for pin::is_set_low call");

        assert!(
            matches!(kind, TransactionKind::GetState(_)),
            "expected pin::is_set_low"
        );

        if let Some(e) = err {
            Err(e)
        } else if let TransactionKind::GetState(v) = kind {
            Ok(v == State::Low)
        } else {
            unreachable!();
        }
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::digital::Wait for Mock {
    /// Wait for the pin to go high
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s
            .next()
            .expect("no expectation for pin::wait_for_high call");

        assert!(
            matches!(kind, TransactionKind::WaitForState(State::High)),
            "got call to wait_for_high"
        );

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Wait for the pin to go low
    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } =
            s.next().expect("no expectation for pin::wait_for_low call");

        assert!(
            matches!(kind, TransactionKind::WaitForState(State::Low)),
            "got call to wait_for_low"
        );

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Wait for the pin to have a rising edge
    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s
            .next()
            .expect("no expectation for pin::wait_for_rising_edge call");

        assert!(
            matches!(kind, TransactionKind::WaitForEdge(Edge::Rising)),
            "got call to wait_for_rising_edge"
        );

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Wait for the pin to have a falling edge
    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s
            .next()
            .expect("no expectation for pin::wait_for_falling_edge call");

        assert!(
            matches!(kind, TransactionKind::WaitForEdge(Edge::Falling)),
            "got call to wait_for_falling_edge"
        );

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Wait for the pin to have either a rising or falling edge
    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s
            .next()
            .expect("no expectation for pin::wait_for_any_edge call");

        assert!(
            matches!(kind, TransactionKind::WaitForEdge(Edge::Any)),
            "got call to wait_for_any_edge"
        );

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;

    use eh1 as embedded_hal;
    use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};

    use super::{
        super::error::MockError,
        TransactionKind::{Get, GetState, Set, Toggle},
        *,
    };

    #[test]
    fn test_input_pin() {
        let expectations = [
            Transaction::new(Get(State::High)),
            Transaction::new(Get(State::High)),
            Transaction::new(Get(State::Low)),
            Transaction::new(Get(State::Low)),
            Transaction::new(Get(State::High)).with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        assert_eq!(pin.is_high().unwrap(), true);
        assert_eq!(pin.is_low().unwrap(), false);
        assert_eq!(pin.is_high().unwrap(), false);
        assert_eq!(pin.is_low().unwrap(), true);

        pin.is_low().expect_err("expected error return");

        pin.done();
    }

    #[test]
    fn test_output_pin() {
        let expectations = [
            Transaction::new(Set(State::High)),
            Transaction::new(Set(State::Low)),
            Transaction::new(Set(State::High)).with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        pin.set_high().unwrap();
        pin.set_low().unwrap();

        pin.set_high().expect_err("expected error return");

        pin.done();
    }

    #[test]
    fn test_stateful_output_pin() {
        let expectations = [
            Transaction::new(GetState(State::Low)),
            Transaction::get_state(State::Low),
            Transaction::new(Toggle),
            Transaction::get_state(State::High),
            Transaction::get_state(State::High),
            Transaction::toggle(),
            Transaction::get_state(State::Low).with_error(MockError::Io(ErrorKind::NotConnected)),
            Transaction::toggle().with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        assert!(pin.is_set_low().unwrap());
        assert!(!pin.is_set_high().unwrap());
        pin.toggle().unwrap();
        assert!(pin.is_set_high().unwrap());
        assert!(!pin.is_set_low().unwrap());
        pin.toggle().unwrap();

        pin.is_set_low()
            .expect_err("expected an error when getting state");
        pin.toggle()
            .expect_err("expected an error when toggling state");

        pin.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_can_wait_for_state() {
        use embedded_hal_async::digital::Wait;

        let expectations = [
            Transaction::new(TransactionKind::WaitForState(State::High)),
            Transaction::new(TransactionKind::WaitForState(State::Low)),
            Transaction::new(TransactionKind::WaitForState(State::High))
                .with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        pin.wait_for_high().await.unwrap();
        pin.wait_for_low().await.unwrap();

        pin.wait_for_high()
            .await
            .expect_err("expected error return");

        pin.done();
    }

    #[tokio::test]
    #[should_panic(expected = "got call to wait_for_high")]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_wait_for_wrong_state() {
        use embedded_hal_async::digital::Wait;

        let expectations = [Transaction::wait_for_state(State::Low)];
        let mut pin = Mock::new(&expectations);

        pin.wait_for_high().await.unwrap();

        pin.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_can_wait_for_edge() {
        use embedded_hal_async::digital::Wait;

        let expectations = [
            Transaction::new(TransactionKind::WaitForEdge(Edge::Rising)),
            Transaction::new(TransactionKind::WaitForEdge(Edge::Falling)),
            Transaction::new(TransactionKind::WaitForEdge(Edge::Any)),
            Transaction::new(TransactionKind::WaitForEdge(Edge::Rising))
                .with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        pin.wait_for_rising_edge().await.unwrap();
        pin.wait_for_falling_edge().await.unwrap();
        pin.wait_for_any_edge().await.unwrap();

        pin.wait_for_rising_edge()
            .await
            .expect_err("expected error return");

        pin.done();
    }

    #[tokio::test]
    #[should_panic(expected = "got call to wait_for_rising_edge")]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_wait_for_wrong_edge() {
        use embedded_hal_async::digital::Wait;

        let expectations = [Transaction::wait_for_edge(Edge::Falling)];
        let mut pin = Mock::new(&expectations);

        pin.wait_for_rising_edge().await.unwrap();

        pin.done();
    }
}
