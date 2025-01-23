use usb_device::{bus::UsbBus, LangID};
use usbd_gscan::{
    host::{
        CanBitTimingConst, CanState, DeviceBitTiming, DeviceBitTimingConst,
        DeviceBitTimingConstExtended, DeviceConfig, DeviceState, Feature,
    },
    Device, GsCan,
};

const TIMING_NOMINAL: CanBitTimingConst = CanBitTimingConst {
    tseg1_min: 1,
    tseg1_max: 255,
    tseg2_min: 1,
    tset2_max: 127,
    sjw_max: 127,
    brp_min: 1,
    brp_max: 511,
    brp_inc: 1,
};
const TIMING_DATA: CanBitTimingConst = CanBitTimingConst {
    tseg1_min: 1,
    tseg1_max: 31,
    tseg2_min: 1,
    tset2_max: 15,
    sjw_max: 15,
    brp_min: 1,
    brp_max: 31,
    brp_inc: 1,
};

pub struct MockCanDevice {}

impl Device for MockCanDevice {
    fn config(&self) -> DeviceConfig {
        DeviceConfig::new(2)
    }

    fn bit_timing(&self) -> DeviceBitTimingConst {
        DeviceBitTimingConst {
            features: Feature::all(),
            fclk_can: 80_000_000,
            timing: TIMING_NOMINAL,
        }
    }

    fn bit_timing_ext(&self) -> DeviceBitTimingConstExtended {
        DeviceBitTimingConstExtended {
            features: Feature::all(),
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

    fn receive(&mut self, _interface: u8, _frame: &usbd_gscan::host::Frame) {}
}

use usbd_class_tester::prelude::*;

struct TestCtx {}

impl UsbDeviceCtx for TestCtx {
    type C<'c> = GsCan<'c, EmulatedUsbBus, MockCanDevice>;

    fn create_class<'a>(
        &mut self,
        alloc: &'a usb_device::bus::UsbBusAllocator<EmulatedUsbBus>,
    ) -> AnyResult<Self::C<'a>> {
        Ok(GsCan::new(&alloc, MockCanDevice {}))
    }
}

#[test]
fn test_host_format() {
    TestCtx {}
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
