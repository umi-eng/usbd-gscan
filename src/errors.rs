use crate::host::{CanData, Frame, FrameFlag, IdFlag};
use bitflags::bitflags;
use zerocopy::FromZeroes;

/// Error message to host
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Error {
    /// Transmit timeout
    pub tx_timeout: bool,
    /// Arbitration lost with bit number in bit stream. Zero if unspecified
    pub lost_arbitration: Option<u8>,
    /// Controller error
    pub controller: Option<ControllerError>,
    /// Protocol error kind and location
    pub protocol: Option<(ProtocolErrorKind, ProtocolErrorLocation)>,
    /// Transceiver error
    pub transceiver: Option<TransceiverError>,
    /// No ack received
    pub no_ack: bool,
    /// Bus error state
    pub bus_error: bool,
    /// Controller restarted
    pub restarted: bool,
    /// Transmit/receive error counts
    pub tx_rx_error_count: Option<(u8, u8)>,
}

impl Error {
    pub(crate) fn to_err_frame(&self, interface: u8) -> crate::host::Frame {
        let mut mask = ErrorClass::empty();
        let mut data = [0; 8];

        mask.set(ErrorClass::TX_TIMEOUT, self.tx_timeout);
        if let Some(arb) = self.lost_arbitration {
            mask |= ErrorClass::LOST_ARBITRATION;
            data[0] = arb;
        }
        if let Some(err) = self.controller {
            mask |= ErrorClass::CONTROLLER;
            data[1] = err.bits();
        }
        if let Some((kind, loc)) = self.protocol {
            mask |= ErrorClass::PROTOCOL;
            data[2] = kind.bits();
            data[3] = loc as u8;
        }
        if let Some(err) = self.transceiver {
            mask |= ErrorClass::TRANSCEIVER;
            data[4] = err as u8;
        }
        mask.set(ErrorClass::NO_ACK, self.no_ack);
        mask.set(ErrorClass::BUS_ERROR, self.bus_error);
        mask.set(ErrorClass::RESTARTED, self.restarted);
        if let Some((tx, rx)) = self.tx_rx_error_count {
            mask |= ErrorClass::COUNTER;
            data[6] = tx;
            data[7] = rx;
        }

        let mut can_data = CanData::new_zeroed();
        can_data.classic_can.data = data;

        let mut frame = Frame::new_zeroed();
        frame.interface = interface;
        frame.echo_id = u32::MAX;
        frame.can_id = mask.bits() as u32 | IdFlag::ERROR.bits();
        frame.flags = FrameFlag::empty();
        frame.can_dlc = 8;
        frame.can_data = can_data;
        frame
    }
}

bitflags! {
    pub(crate) struct ErrorClass: u16 {
        const TX_TIMEOUT = 1 << 0;
        const LOST_ARBITRATION = 1 << 1;
        const CONTROLLER = 1 << 2;
        const PROTOCOL = 1 << 3;
        const TRANSCEIVER = 1 << 4;
        const NO_ACK = 1 << 5;
        const BUS_OFF = 1 << 6;
        const BUS_ERROR = 1 << 7;
        const RESTARTED = 1 << 8;
        const COUNTER = 1 << 9;
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct ControllerError(u8);

bitflags! {
    impl ControllerError: u8 {
        const RX_OVERFLOW = 1 << 0;
        const TX_OVERFLOW = 1 << 1;
        const RX_WARNING = 1 << 2;
        const TX_WARNING = 1 << 3;
        const RX_PASSIVE = 1 << 4;
        const TX_PASSIVE = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct ProtocolErrorKind(u8);

bitflags! {
    impl ProtocolErrorKind: u8 {
        const UNSPECIFIED = 0;
        const SINGLE_BIT = 1 << 0;
        const FRAME_FORMAT = 1 << 1;
        const BIT_STUFFING = 1 << 2;
        const BIT_DOMINANT = 1 << 3;
        const BIT_RECESSIVE = 1 << 4;
        const BUS_OVERLOAD = 1 << 5;
        const ACTIVE_ERROR = 1 << 6;
        const TRANSMIT = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub enum ProtocolErrorLocation {
    /// Unspecified
    Unspecified = 0x00,
    /// Start of frame
    Sof = 0x03,
    /// Bits 28 - 21
    Id28_21 = 0x02,
    /// Bits 20 - 18
    Id20_18 = 0x06,
    /// Substitude RTR
    Srtr = 0x04,
    /// Identifier extension
    Ide = 0x05,
    /// Bits 17 - 13
    Id17_13 = 0x07,
    /// Bits 12 - 5
    Id12_5 = 0x0F,
    /// Bits 4 - 0
    Id4_0 = 0x0E,
    /// RTR bit
    Rtr = 0x0C,
    /// Reserved bit 1
    Res1 = 0x0D,
    /// Reserved bit 0
    Res0 = 0x09,
    /// Data length code
    Dlc = 0x0B,
    /// Data section
    Data = 0x0A,
    /// CRC sequence
    CrcSeq = 0x08,
    /// CRC delimiter
    CrcDel = 0x18,
    /// ACK slot
    Ack = 0x19,
    /// ACK delimiter
    AckDel = 0x1B,
    /// End of frame
    Eof = 0x1A,
    /// Intermission
    Int = 0x12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub enum TransceiverError {
    /// Unspecified
    Unspecified = 0x00,
    /// CAN high no wire
    CanHighNoWire = 0x04,
    /// CAN high short to battery
    CanHighShortToBat = 0x05,
    /// CAN high short to VCC
    CanHighShortToVcc = 0x06,
    /// CAN high short to ground
    CanHighShortToGround = 0x07,
    /// CAN low no wire
    CanLowNoWire = 0x40,
    /// CAN low short to battery
    CanLowShortToBat = 0x50,
    /// CAN low short to VCC
    CanLowShortToVcc = 0x60,
    /// CAN low short to ground
    CanLowShortToGround = 0x70,
    /// CAN low short to CAN high
    CanLowShortToCanHigh = 0x80,
}

/// CAN error warning threshold
pub const THRESHOLD_WARNING: u16 = 96;
/// CAN error passive threshold
pub const THRESHOLD_PASSIVE: u16 = 128;
/// CAN error bus-off threshold
pub const THRESHOLD_BUS_OFF: u16 = 255;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_frame() {
        let error = Error::default();
        let frame = error.to_err_frame(0);

        assert_eq!(frame.can_dlc, 8);
        assert_eq!(frame.echo_id, u32::MAX); // received frame
        assert_eq!(IdFlag::from_bits_truncate(frame.can_id), IdFlag::ERROR);
        assert_eq!(
            unsafe { frame.can_data.classic_can }.data,
            [0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}
