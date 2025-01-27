//! Host interface messages.

use bitflags::bitflags;
use embedded_can::{ExtendedId, Id, StandardId};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Tells the device the byte order of the host.
///
/// `byte_order` will contain `0x0000beef` for little endian and `0xefbe0000`
/// for big endian.
#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct HostConfig {
    pub byte_order: u32,
}

/// Device configuration.
///
/// `interface_count`
#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceConfig {
    _reserved0: u8,
    _reserved1: u8,
    _reserved2: u8,
    pub interface_count: u8,
    pub software_version: u32,
    pub hardware_version: u32,
}

impl DeviceConfig {
    /// Creates a new device config.
    ///
    /// The number of interfaces must be at least 1.
    pub fn new(interfaces: u8) -> Self {
        assert!(interfaces > 0);

        // API useses N-1 to represent the interface count.
        let interface_count = interfaces - 1;

        Self {
            _reserved0: 0,
            _reserved1: 0,
            _reserved2: 0,
            interface_count,
            software_version: 2, // to match candleLight firmware.
            hardware_version: 0,
        }
    }
}

/// Device mode.
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Mode {
    Reset = 0,
    Start = 1,
}

impl TryFrom<u32> for Mode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Reset as u32 => Ok(Self::Reset),
            x if x == Self::Start as u32 => Ok(Self::Start),
            _ => Err(()),
        }
    }
}

#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceMode {
    pub mode: u32,
    pub flags: Feature,
}

/// Same as Linux netlink can_state.
#[derive(Debug, Clone, Copy, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(u32)]
pub enum CanState {
    /// RX/TX error count < 96
    Active = 0,
    /// RX/TX error count < 128
    Warning = 1,
    /// RX/TX error count < 256
    Passive = 2,
    /// RX/TX error count >= 256
    BusOff = 3,
    /// Device is stopped
    Stopped = 4,
    /// Device is sleeping
    Sleeping = 5,
}

impl Into<u32> for CanState {
    fn into(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceState {
    pub state: CanState,
    pub rx_errors: u32,
    pub tx_errors: u32,
}

#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceBitTiming {
    pub prop_seg: u32,
    pub phase_seg1: u32,
    pub phase_seg2: u32,
    pub sjw: u32,
    pub brp: u32,
}

#[derive(Debug, Default, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct CanBitTimingConst {
    pub tseg1_min: u32,
    pub tseg1_max: u32,
    pub tseg2_min: u32,
    pub tseg2_max: u32,
    pub sjw_max: u32,
    pub brp_min: u32,
    pub brp_max: u32,
    pub brp_inc: u32,
}

/// Features flags that can be advertised by the device.
#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct Feature(u32);

bitflags! {
    impl Feature: u32 {
        const LISTEN_ONLY = 1 << 0;
        const LOOP_BACK = 1 << 1;
        const TRIPLE_SAMPLE = 1 << 2;
        const ONE_SHOT = 1 << 3;
        const HW_TIMESTAMP = 1 << 4;
        const IDENTIFY = 1 << 5;
        const USER_ID = 1 << 6;
        const PAD_PKTS_TO_MAX_PKT_SIZE = 1 << 7;
        const FD = 1 << 8;
        const REQ_USB_QUIRK_LPC546XX = 1 << 9;
        const BT_CONST_EXT = 1 << 10;
        const TERMINATION = 1 << 11;
        const BUS_ERROR_REPORTING = 1 << 12;
        const GET_STATE = 1 << 13;
    }
}

/// Device bit timing and feature flags.
#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceBitTimingConst {
    pub features: Feature,
    pub fclk_can: u32,
    pub timing: CanBitTimingConst,
}

/// Device extended bit timing and feature flags for CAN FD devices.
#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceBitTimingConstExtended {
    pub features: Feature,
    pub fclk_can: u32,
    pub timing_nominal: CanBitTimingConst,
    pub timing_data: CanBitTimingConst,
}

#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct IdentifyMode {
    pub mode: u32,
}

#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceTerminationState {
    pub state: u32,
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct ClassicCan {
    pub data: [u8; 8],
    _padding: [u8; 60],
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct ClassicCanTimestamp {
    pub data: [u8; 8],
    pub timestamp_us: u32,
    _padding: [u8; 56],
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct CanFd {
    pub data: [u8; 64],
    _padding: [u8; 4],
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct CanFdTimestamp {
    pub data: [u8; 64],
    pub timestamp_us: u32,
}

#[derive(Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub union CanData {
    pub classic_can: ClassicCan,
    pub classic_can_timestamp: ClassicCanTimestamp,
    pub can_fd: CanFd,
    pub can_fd_timestamp: CanFdTimestamp,
}

/// Frame flags.
#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct FrameFlag(u8);

bitflags! {
    impl FrameFlag: u8 {
        /// Receive/transmit overflow.
        const OVERFLOW = 1 << 0;
        /// FD type frame.
        const FD = 1 << 1;
        /// FD bit rate switching in use.
        const BIT_RATE_SWITCH = 1 << 2;
        /// FD error state indicator.
        const ERROR_STATE_INDICATOR = 1 << 3;
    }
}

#[derive(Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub struct Frame {
    pub echo_id: u32,
    pub can_id: u32,
    pub can_dlc: u8,
    pub interface: u8,
    pub flags: FrameFlag,
    _reserved0: u8,
    pub can_data: CanData,
}

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let mut frame = Frame::new_zeroed();

        frame.can_dlc = fd_len_to_dlc(data.len())?;

        match id.into() {
            Id::Standard(id) => frame.can_id = id.as_raw() as u32,
            Id::Extended(id) => frame.can_id = id.as_raw() | IdFlag::EXTENDED.bits(),
        }

        unsafe { frame.can_data.can_fd.data[..data.len()].copy_from_slice(data) };

        Some(frame)
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        let mut frame = Frame::new_zeroed();

        match id.into() {
            Id::Standard(id) => frame.can_id = id.as_raw() as u32,
            Id::Extended(id) => frame.can_id = id.as_raw() | IdFlag::EXTENDED.bits(),
        }

        frame.can_dlc = dlc as u8;

        Some(frame)
    }

    fn id(&self) -> Id {
        let masked = self.can_id & 0x1FFFFFFF;
        if self.is_extended() {
            Id::Extended(ExtendedId::new(masked).unwrap())
        } else {
            Id::Standard(StandardId::new(masked as u16).unwrap())
        }
    }

    fn is_extended(&self) -> bool {
        (self.can_id & IdFlag::EXTENDED.bits()) != 0
    }

    fn is_remote_frame(&self) -> bool {
        (self.can_id & IdFlag::REMOTE.bits()) != 0
    }

    fn dlc(&self) -> usize {
        self.can_dlc as usize
    }

    fn data(&self) -> &[u8] {
        // safety: underlying type is initialised with zeros and length is given by dlc.
        let len = fd_dlc_to_len(self.dlc()).unwrap();
        if self.flags.intersects(FrameFlag::FD) {
            unsafe { &self.can_data.can_fd.data[..len] }
        } else {
            unsafe { &self.can_data.classic_can.data[..len] }
        }
    }
}

/// Identifier flags.
#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct IdFlag(u32);

bitflags! {
    impl IdFlag: u32 {
        /// Extended frame.
        const EXTENDED = 0x80000000;
        /// Remote frame.
        const REMOTE = 0x40000000;
        /// Error frame.
        const ERROR = 0x20000000;
    }
}

/// Get the data length for a given DLC.
#[allow(unused)]
fn fd_dlc_to_len(dlc: usize) -> Option<usize> {
    match dlc {
        0..=8 => Some(dlc as usize),
        9 => Some(12),
        10 => Some(16),
        11 => Some(20),
        12 => Some(24),
        13 => Some(32),
        14 => Some(48),
        15 => Some(64),
        _ => None,
    }
}

/// Get the DLC for a given data length.
fn fd_len_to_dlc(len: usize) -> Option<u8> {
    match len {
        0..=8 => Some(len as u8),
        12 => Some(9),
        16 => Some(10),
        20 => Some(11),
        24 => Some(12),
        32 => Some(13),
        48 => Some(14),
        64 => Some(15),
        _ => panic!("Invalid len: {}", len),
    }
}
