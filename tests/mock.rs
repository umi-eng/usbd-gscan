use embedded_can::Frame as _;
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
