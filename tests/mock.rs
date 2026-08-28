use embedded_can::Frame as _;
use usbd_gscan::errors::Error;
use usbd_gscan::host::CanBitTimingConst;
use usbd_gscan::host::CanState;
use usbd_gscan::host::DeviceBitTiming;
use usbd_gscan::host::DeviceBitTimingConst;
use usbd_gscan::host::DeviceBitTimingConstExtended;
use usbd_gscan::host::DeviceConfig;
use usbd_gscan::host::DeviceState;
use usbd_gscan::host::Feature;
use usbd_gscan::host::Frame;
use usbd_gscan::host::FrameFlag;
use usbd_gscan::Device;
use usbd_gscan::GsCan;
use zerocopy::IntoBytes;

const TIMING_NOMINAL: CanBitTimingConst = CanBitTimingConst {
    tseg1_min: 1,
    tseg1_max: 255,
    tseg2_min: 1,
    tseg2_max: 127,
    sjw_max: 127,
    brp_min: 1,
    brp_max: 511,
    brp_inc: 1,
};
const TIMING_DATA: CanBitTimingConst = CanBitTimingConst {
    tseg1_min: 1,
    tseg1_max: 31,
    tseg2_min: 1,
    tseg2_max: 15,
    sjw_max: 15,
    brp_min: 1,
    brp_max: 31,
    brp_inc: 1,
};

pub struct MockCanDevice {
    features: Feature,
    timestamp: u32,
}

impl Device for MockCanDevice {
    fn config(&self) -> DeviceConfig {
        DeviceConfig::new(2)
    }

    fn bit_timing(&self) -> DeviceBitTimingConst {
        DeviceBitTimingConst {
            features: self.features,
            fclk_can: 80_000_000,
            timing: TIMING_NOMINAL,
        }
    }

    fn bit_timing_ext(&self) -> DeviceBitTimingConstExtended {
        DeviceBitTimingConstExtended {
            features: self.features,
            fclk_can: 80_000_000,
            timing_nominal: TIMING_NOMINAL,
            timing_data: TIMING_DATA,
        }
    }

    fn configure_bit_timing(&mut self, _interface: u8, _timing: DeviceBitTiming) {}

    fn configure_bit_timing_data(&mut self, _interface: u8, _timing: DeviceBitTiming) {}

    fn reset(&mut self, _interface: u8) {}

    fn start(&mut self, _interface: u8, _features: Feature) {}

    fn state(&self, _interface: u8) -> DeviceState {
        DeviceState {
            state: CanState::Active,
            rx_errors: 0,
            tx_errors: 0,
        }
    }

    fn timestamp(&self) -> u32 {
        self.timestamp
    }

    fn receive(&mut self, _interface: u8, _frame: &usbd_gscan::host::Frame) {}
}

use usbd_class_tester::prelude::*;

struct TestCtx {
    features: Feature,
    timestamp: u32,
}

impl UsbDeviceCtx for TestCtx {
    type C<'c> = GsCan<'c, EmulatedUsbBus, MockCanDevice>;

    fn create_class<'a>(
        &mut self,
        alloc: &'a usb_device::bus::UsbBusAllocator<EmulatedUsbBus>,
    ) -> AnyResult<Self::C<'a>> {
        Ok(GsCan::new(
            &alloc,
            MockCanDevice {
                features: self.features,
                timestamp: self.timestamp,
            },
        ))
    }
}

#[test]
fn test_hardware_timestamp_classic_can() {
    TestCtx {
        features: Feature::HW_TIMESTAMP,
        timestamp: 0x12345678,
    }
    .with_usb(|mut cls, mut dev| {
        let frame = Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[1, 2, 3]).unwrap();
        cls.transmit(0, &frame, FrameFlag::empty());

        let data = dev.ep_read(&mut cls, 1, 64).unwrap();
        assert_eq!(data.len(), 24);
        assert_eq!(&data[20..24], &0x12345678_u32.to_ne_bytes());
    })
    .expect("with_usb");
}

#[test]
fn test_hardware_timestamp_can_fd() {
    TestCtx {
        features: Feature::HW_TIMESTAMP | Feature::FD,
        timestamp: 0x87654321,
    }
    .with_usb(|mut cls, mut dev| {
        let mut frame =
            Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[0; 64]).unwrap();
        frame.flags = FrameFlag::FD;
        cls.transmit(0, &frame, FrameFlag::FD);

        let data = dev.ep_read(&mut cls, 1, 128).unwrap();
        assert_eq!(data.len(), 80);
        assert_eq!(&data[76..80], &0x87654321_u32.to_ne_bytes());
    })
    .expect("with_usb");
}

#[test]
fn test_hardware_timestamp_not_advertised() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0x12345678,
    }
    .with_usb(|mut cls, mut dev| {
        let frame = Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[1, 2, 3]).unwrap();
        cls.transmit(0, &frame, FrameFlag::empty());

        let data = dev.ep_read(&mut cls, 1, 64).unwrap();
        assert_eq!(data.len(), 20);
        assert_eq!(&data[20..], &[]);
    })
    .expect("with_usb");
}

#[test]
fn test_timestamp_request() {
    TestCtx {
        features: Feature::HW_TIMESTAMP,
        timestamp: 0x12345678,
    }
    .with_usb(|mut cls, mut dev| {
        let data = dev
            .control_read(
                &mut cls,
                CtrRequestType::to_host().interface().vendor(),
                6,
                0,
                0,
                4,
            )
            .unwrap();
        assert_eq!(data, 0x12345678_u32.to_le_bytes());
    })
    .expect("with_usb");
}

#[test]
fn test_timestamp_request_without_feature() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0x12345678,
    }
    .with_usb(|mut cls, mut dev| {
        assert!(dev
            .control_read(
                &mut cls,
                CtrRequestType::to_host().interface().vendor(),
                6,
                0,
                0,
                4,
            )
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_control_in_device_information_requests() {
    TestCtx {
        features: Feature::FD | Feature::BT_CONST_EXT | Feature::GET_STATE,
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let request_type = CtrRequestType::to_host().interface().vendor();

        let timing = dev
            .control_read(&mut cls, request_type, 4, 0, 0, 64)
            .unwrap();
        assert_eq!(timing.len(), core::mem::size_of::<DeviceBitTimingConst>());
        assert_eq!(
            &timing[0..4],
            &(Feature::FD | Feature::BT_CONST_EXT | Feature::GET_STATE)
                .bits()
                .to_le_bytes()
        );
        assert_eq!(&timing[4..8], &80_000_000_u32.to_le_bytes());

        let config = dev
            .control_read(&mut cls, request_type, 5, 0, 0, 64)
            .unwrap();
        assert_eq!(config.len(), core::mem::size_of::<DeviceConfig>());
        assert_eq!(config[3], 1);
        assert_eq!(&config[4..8], &2_u32.to_le_bytes());
        assert_eq!(&config[8..12], &0_u32.to_le_bytes());

        let timing_ext = dev
            .control_read(&mut cls, request_type, 11, 0, 0, 128)
            .unwrap();
        assert_eq!(
            timing_ext.len(),
            core::mem::size_of::<DeviceBitTimingConstExtended>()
        );

        let state = dev
            .control_read(&mut cls, request_type, 14, 0, 0, 64)
            .unwrap();
        assert_eq!(state.len(), core::mem::size_of::<DeviceState>());
        assert_eq!(&state[0..4], &0_u32.to_le_bytes());
        assert_eq!(&state[4..12], &[0; 8]);
    })
    .expect("with_usb");
}

#[test]
fn test_invalid_get_state_interface_is_rejected() {
    TestCtx {
        features: Feature::GET_STATE,
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        assert!(dev
            .control_read(
                &mut cls,
                CtrRequestType::to_host().interface().vendor(),
                14,
                3,
                0,
                64,
            )
            .is_err());
        assert!(dev
            .control_read(
                &mut cls,
                CtrRequestType::to_host().interface().vendor(),
                99,
                0,
                0,
                4,
            )
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_host_format() {
    TestCtx {
        features: Feature::all(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        dev.control_write(
            &mut cls,
            CtrRequestType::to_device().class().vendor(),
            0,
            0,
            0,
            4,
            &0x0000beef_u32.to_le_bytes(),
        )
        .unwrap();

        let st = dev.interface_get_status(&mut cls, 0).expect("Status");
        assert_eq!(st, 0);
    })
    .expect("with_usb")
}

#[test]
fn test_invalid_host_format_requests_are_rejected() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let request_type = CtrRequestType::to_device().class().vendor();
        assert!(dev
            .control_write(&mut cls, request_type, 0, 0, 0, 0, &[])
            .is_err());
        assert!(dev
            .control_write(&mut cls, request_type, 0, 0, 0, 4, &[0; 4])
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_valid_timing_and_mode_requests() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let request_type = CtrRequestType::to_device().class().vendor();
        let timing = [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0];

        dev.control_write(&mut cls, request_type, 1, 0, 0, 20, &timing)
            .unwrap();
        dev.control_write(&mut cls, request_type, 10, 0, 0, 20, &timing)
            .unwrap();

        let reset = [0, 0, 0, 0, 0, 0, 0, 0];
        dev.control_write(&mut cls, request_type, 2, 0, 0, 8, &reset)
            .unwrap();

        let start = [1, 0, 0, 0, 0x80, 0x01, 0, 0];
        dev.control_write(&mut cls, request_type, 2, 0, 0, 8, &start)
            .unwrap();
    })
    .expect("with_usb");
}

#[test]
fn test_invalid_timing_and_mode_requests_are_rejected() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let request_type = CtrRequestType::to_device().class().vendor();
        assert!(dev
            .control_write(&mut cls, request_type, 1, 0, 0, 4, &[0; 4])
            .is_err());
        assert!(dev
            .control_write(&mut cls, request_type, 10, 0, 0, 4, &[0; 4])
            .is_err());

        let invalid_mode = [2, 0, 0, 0, 0, 0, 0, 0];
        assert!(dev
            .control_write(&mut cls, request_type, 2, 0, 0, 8, &invalid_mode)
            .is_err());
        assert!(dev
            .control_write(&mut cls, request_type, 99, 0, 0, 0, &[])
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_invalid_interface_request_is_rejected() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        assert!(dev
            .control_write(
                &mut cls,
                CtrRequestType::to_device().class().vendor(),
                2,
                3,
                0,
                8,
                &[0; 8],
            )
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_invalid_host_byte_order_is_rejected() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        assert!(dev
            .control_write(
                &mut cls,
                CtrRequestType::to_device().class().vendor(),
                0,
                0,
                0,
                4,
                &0xefbe0000_u32.to_le_bytes(),
            )
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_malformed_mode_request_is_rejected() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        assert!(dev
            .control_write(
                &mut cls,
                CtrRequestType::to_device().class().vendor(),
                2,
                0,
                0,
                4,
                &[0; 4],
            )
            .is_err());
    })
    .expect("with_usb");
}

#[test]
fn test_transmit_error_emits_error_frame() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        cls.transmit_error(
            0,
            Error {
                tx_timeout: true,
                no_ack: true,
                ..Error::default()
            },
        );

        let data = dev.ep_read(&mut cls, 1, 64).unwrap();
        assert_eq!(data.len(), 20);
        assert_eq!(
            u32::from_ne_bytes(data[4..8].try_into().unwrap()) & 0x20000000,
            0x20000000
        );
        assert_eq!(data[12], 0);
    })
    .expect("with_usb");
}

#[test]
fn test_classic_frame_can_be_padded_to_endpoint_size() {
    TestCtx {
        features: Feature::PAD_PKTS_TO_MAX_PKT_SIZE,
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        dev.control_write(
            &mut cls,
            CtrRequestType::to_device().class().vendor(),
            2,
            0,
            0,
            8,
            &[1, 0, 0, 0, 0x80, 0, 0, 0],
        )
        .unwrap();
        let frame = Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[1, 2, 3]).unwrap();
        cls.transmit(0, &frame, FrameFlag::empty());

        let data = dev.ep_read(&mut cls, 1, 64).unwrap();
        assert_eq!(data.len(), 64);
        assert_eq!(&data[4..8], &frame.can_id.to_le_bytes());
        assert_eq!(&data[12..15], &[1, 2, 3]);
        assert_eq!(&data[20..], &[0; 44]);
    })
    .expect("with_usb");
}

#[test]
fn test_queued_classic_frames_are_sent_in_order() {
    TestCtx {
        features: Feature::empty(),
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let first = Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[1]).unwrap();
        let second = Frame::new(embedded_can::StandardId::new(0x456).unwrap(), &[2]).unwrap();
        cls.transmit(0, &first, FrameFlag::empty());
        cls.transmit(0, &second, FrameFlag::empty());

        let first_data = dev.ep_read(&mut cls, 1, 64).unwrap();
        assert_eq!(&first_data[4..8], &first.can_id.to_le_bytes());
        assert_eq!(first_data[12], 1);
        let second_data = dev.ep_read(&mut cls, 1, 64).unwrap();
        assert_eq!(&second_data[4..8], &second.can_id.to_le_bytes());
        assert_eq!(second_data[12], 2);
    })
    .expect("with_usb");
}

#[test]
fn test_short_classic_frame_is_processed_without_fd_continuation() {
    TestCtx {
        features: Feature::FD,
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let frame = Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[1, 2, 3]).unwrap();

        // A short classic frame must not be treated as an FD continuation
        // merely because FD is supported.
        let frame_len = frame.len(false);
        assert_eq!(frame_len, 20);
        // `ep_raw` drives the class callback synchronously. Supplying an IN
        // buffer also lets the harness consume the echoed frame.
        let mut echoed = [0u8; 64];
        let result = dev
            .ep_raw(
                &mut cls,
                1,
                None,
                Some(&frame.as_bytes()[..frame_len]),
                &mut echoed,
            )
            .unwrap();
        assert_eq!(result.read, Some(frame_len));
        assert_eq!(&echoed[..frame_len], &frame.as_bytes()[..frame_len]);
    })
    .expect("with_usb");
}

#[test]
fn test_fd_frame_accepts_short_continuation_with_timestamp_enabled() {
    TestCtx {
        features: Feature::FD | Feature::HW_TIMESTAMP,
        timestamp: 0x12345678,
    }
    .with_usb(|mut cls, mut dev| {
        let mut frame =
            Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[0; 64]).unwrap();
        frame.flags = FrameFlag::FD;

        // Linux can omit the host-to-device timestamp, leaving a 12-byte
        // continuation even when hardware timestamps are enabled.
        dev.ep_write(&mut cls, 1, &frame.as_bytes()[..64]).unwrap();
        let mut echoed = [0u8; 128];
        let result = dev
            .ep_raw(
                &mut cls,
                1,
                None,
                Some(&frame.as_bytes()[64..76]),
                &mut echoed,
            )
            .unwrap();

        assert_eq!(result.read, Some(80));
        assert_eq!(&echoed[..76], &frame.as_bytes()[..76]);
        assert_eq!(&echoed[76..80], &0x12345678_u32.to_ne_bytes());
    })
    .expect("with_usb");
}

#[test]
fn test_invalid_fd_continuation_is_dropped_and_state_recovers() {
    TestCtx {
        features: Feature::FD,
        timestamp: 0,
    }
    .with_usb(|mut cls, mut dev| {
        let mut fd_frame =
            Frame::new(embedded_can::StandardId::new(0x123).unwrap(), &[0; 64]).unwrap();
        fd_frame.flags = FrameFlag::FD;

        // The first 64-byte packet starts FD reassembly.
        dev.ep_write(&mut cls, 1, &fd_frame.as_bytes()[..64])
            .unwrap();
        // A valid continuation is 16 bytes; this malformed packet must not
        // panic or leave the reassembly state stuck.
        dev.ep_write(&mut cls, 1, &[0; 8]).unwrap();

        let classic =
            Frame::new(embedded_can::StandardId::new(0x456).unwrap(), &[4, 5, 6]).unwrap();
        let classic_len = classic.len(false);
        let mut packet = [0u8; 64];
        packet[..classic_len].copy_from_slice(&classic.as_bytes()[..classic_len]);
        let mut echoed = [0u8; 64];
        let result = dev
            .ep_raw(&mut cls, 1, None, Some(&packet), &mut echoed)
            .unwrap();
        assert_eq!(result.read, Some(classic_len));
        assert_eq!(&echoed[..classic_len], &classic.as_bytes()[..classic_len]);
    })
    .expect("with_usb");
}
