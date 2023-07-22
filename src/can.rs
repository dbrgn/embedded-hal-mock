//! CAN mock implementations.
//!
//! ## Usage
//!
//! ```
//! extern crate embedded_hal;
//! extern crate embedded_hal_mock;
//!
//! use embedded_can::blocking::Can;
//! use embedded_can::Frame;
//! use embedded_can::{StandardId, ExtendedId};
//! use embedded_hal_mock::can::{Mock as CanMock, Transaction as CanTransaction, MockFrame as CanMockFrame};
//!
//! // Create CAN frame
//!
//! let id: StandardId = StandardId::new(0x123).unwrap();
//! let frame1 = CanMockFrame::new(id, &[0x00, 0x00, 0x00, 0x00, 0x10]).unwrap();
//! let id: ExtendedId = ExtendedId::new(0x12345678).unwrap();
//! let frame2 = CanMockFrame::new(id, &[0x00, 0x00, 0x00, 0x00, 0x10]).unwrap();
//!
//! // Configure expectations
//! let expectations = [
//!     CanTransaction::transmit(frame1.clone()),
//!     CanTransaction::receive(frame2.clone()),
//! ];
//!
//! let mut can = CanMock::new(&expectations);
//!
//! // Transmitting
//! embedded_can::blocking::Can::transmit(&mut can, &frame1.clone()).unwrap();
//!
//! // Receiving
//! let frame = embedded_can::blocking::Can::receive(&mut can).unwrap();
//! assert_eq!(frame, frame2.clone());
//!
//! // Finalise expectations
//! can.done();
//! ```
//!
use embedded_can::{ErrorKind, Frame, Id};
use embedded_hal_nb::nb;

use crate::common::Generic;
use crate::error::MockError;

/// CAN Transaction modes
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Transmit transaction
    Transmit,
    /// Receive transaction
    Receive,
}

/// CAN Transaction type
///
/// Models an CAN transmit or receive transaction
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_frame: Option<MockFrame>,
    response_frame: Option<MockFrame>,
    /// An optional error return for a transaction.
    ///
    /// This is in addition to the mode to allow validation that the
    /// transaction mode is correct prior to returning the error.
    expected_err: Option<ErrorKind>,
}

impl Transaction {
    /// Create a Transmit transaction
    pub fn transmit(expected_frame: MockFrame) -> Transaction {
        Transaction {
            expected_mode: Mode::Transmit,
            expected_frame: Some(expected_frame),
            response_frame: None,
            expected_err: None,
        }
    }

    /// Create a Receive transaction
    pub fn receive(response_frame: MockFrame) -> Transaction {
        Transaction {
            expected_mode: Mode::Receive,
            expected_frame: None,
            response_frame: Some(response_frame),
            expected_err: None,
        }
    }

    /// Add an error return to a transaction
    ///
    /// This is used to mock failure behaviours.
    pub fn with_error(mut self, error: ErrorKind) -> Self {
        self.expected_err = Some(error);
        self
    }
}

/// Mock CAN Frame
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockFrame {
    id: Id,
    data: Vec<u8>,
}

impl Frame for MockFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let id = id.into();
        let data = data.to_vec();
        Some(Self { id, data })
    }

    fn new_remote(id: impl Into<Id>, _: usize) -> Option<Self> {
        let id = id.into();
        let data = [].to_vec();
        Some(Self { id, data })
    }

    fn is_extended(&self) -> bool {
        match self.id {
            Id::Standard(_) => false,
            Id::Extended(_) => true,
        }
    }

    fn is_remote_frame(&self) -> bool {
        self.data.is_empty()
    }

    fn id(&self) -> Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.data.len()
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Mock CAN implementation
pub type Mock = Generic<Transaction>;

impl embedded_can::blocking::Can for Mock {
    type Frame = MockFrame;
    type Error = MockError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), MockError> {
        let t = self.next().expect("no expectation for can::transmit call");
        assert_eq!(
            t.expected_mode,
            Mode::Transmit,
            "can::transmit unexpected mode"
        );
        assert_eq!(
            &t.expected_frame.unwrap(),
            frame,
            "can::transmit data does not match expectation"
        );
        Ok(())
    }

    fn receive(&mut self) -> Result<Self::Frame, MockError> {
        let t = self.next().expect("no expectation for can::receive call");
        assert_eq!(
            t.expected_mode,
            Mode::Receive,
            "can::receive unexpected mode"
        );
        Ok(t.response_frame.unwrap())
    }
}

impl embedded_can::nb::Can for Mock {
    type Frame = MockFrame;
    type Error = MockError;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, MockError> {
        let t = self.next().expect("no expectation for can::transmit call");
        assert_eq!(
            t.expected_mode,
            Mode::Transmit,
            "can::transmit unexpected mode"
        );
        assert_eq!(
            &t.expected_frame.unwrap(),
            frame,
            "can::transmit data does not match expectation"
        );
        Ok(None)
    }

    fn receive(&mut self) -> nb::Result<Self::Frame, MockError> {
        let t = self.next().expect("no expectation for can::receive call");
        assert_eq!(
            t.expected_mode,
            Mode::Receive,
            "can::receive unexpected mode"
        );
        Ok(t.response_frame.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use embedded_can::{ExtendedId, StandardId};

    #[test]
    fn test_can_mock_transmit_standard_id() {
        use embedded_can::blocking::Can;

        let id: StandardId = StandardId::new(0x123).unwrap();
        let frame = MockFrame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]).unwrap();
        let mut can = Mock::new(&[Transaction::transmit(frame.clone())]);

        let id: StandardId = StandardId::new(0x123).unwrap();
        let frame = MockFrame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]).unwrap();
        let _ = can.transmit(&frame);

        can.done();
    }

    #[test]
    fn test_can_mock_transmit_extended_id() {
        use embedded_can::blocking::Can;

        let id: ExtendedId = ExtendedId::new(0x12345678).unwrap();
        let frame = MockFrame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]).unwrap();
        let mut can = Mock::new(&[Transaction::transmit(frame.clone())]);

        let id: ExtendedId = ExtendedId::new(0x12345678).unwrap();
        let frame = MockFrame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]).unwrap();
        let _ = can.transmit(&frame);

        can.done();
    }

    #[test]
    fn test_can_mock_receive_standard_id() {
        use embedded_can::blocking::Can;

        let id: StandardId = StandardId::new(0x123).unwrap();
        let frame = MockFrame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]).unwrap();
        let mut can = Mock::new(&[Transaction::receive(frame.clone())]);

        let result = can.receive().unwrap();

        assert_eq!(result, frame.clone());

        can.done();
    }

    #[test]
    fn test_can_mock_receive_extended_id() {
        use embedded_can::blocking::Can;

        let id: ExtendedId = ExtendedId::new(0x12345678).unwrap();
        let frame = MockFrame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]).unwrap();
        let mut can = Mock::new(&[Transaction::receive(frame.clone())]);

        let result = can.receive().unwrap();

        assert_eq!(result, frame.clone());

        can.done();
    }
}
