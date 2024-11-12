//! Host interface messages.

use bitflags::bitflags;
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
    pub _reserved0: u8,
    pub _reserved1: u8,
    pub _reserved2: u8,
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
    pub tset2_max: u32,
    pub sjw_max: u32,
    pub brp_min: u32,
    pub brp_max: u32,
    pub brp_inc: u32,
}

/// Features flags that can be advertised by the device.
#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
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
    pub bit_timing: CanBitTimingConst,
}

/// Device extended bit timing and feature flags for CAN FD devices.
#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct DeviceBitTimingConstExtended {
    pub features: Feature,
    pub fclk_can: u32,
    pub bit_timing: CanBitTimingConst,
    pub data_timing: CanBitTimingConst,
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
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct ClassicCanTimestamp {
    pub data: [u8; 8],
    pub timestamp_us: u32,
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct CanFd {
    pub data: [u8; 64],
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct CanFdTimestamp {
    pub data: [u8; 64],
    pub timestamp_us: u32,
}

#[derive(Clone, Copy, FromZeroes, FromBytes)]
#[repr(C)]
pub union CanData {
    pub classic_can: ClassicCan,
    pub classic_can_timestamp: ClassicCanTimestamp,
    pub can_fd: CanFd,
    pub can_fd_timestamp: CanFdTimestamp,
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct FrameFlag(u8);

bitflags! {
    impl FrameFlag: u8 {
        const OVERFLOW = 1 << 0;
        const FD = 1 << 1;
        const BIT_RATE_SWITCH = 1 << 2;
        const ERROR_STATE_INDICATOR = 1 << 3;
    }
}

#[derive(Clone, Copy, FromZeroes, FromBytes)]
#[repr(C)]
pub struct Frame {
    pub echo_id: u32,
    pub can_id: u32,
    pub can_dlc: u8,
    pub channel: u8,
    pub flags: FrameFlag,
    pub _reserved0: u8,
    pub can_data: CanData,
}

#[derive(Debug, Clone, Copy, FromZeroes, FromBytes, AsBytes)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(C)]
pub struct CanIdFlags(u32);

bitflags! {
    impl CanIdFlags: u32 {
        const EXT_FRAME = 1 << 31;
        const REMOTE = 1 << 30;
        const ERROR = 1 << 29;
    }
}
