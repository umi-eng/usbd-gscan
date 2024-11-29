#![no_std]

pub mod host;
pub mod identifier;

use embedded_can::Frame as _;
use host::*;
use usb_device::class_prelude::*;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Interface class: vendor defined.
pub const INTERFACE_CLASS: u8 = 0xFF;

const REQ_HOST_FORMAT: u8 = 0;
const REQ_BIT_TIMING: u8 = 1;
const REQ_MODE: u8 = 2;
#[allow(unused)]
const REQ_BUS_ERROR: u8 = 3;
const REQ_BIT_TIMING_CONST: u8 = 4;
const REQ_DEVICE_CONFIG: u8 = 5;
#[allow(unused)]
const REQ_TIMESTAMP: u8 = 6;
#[allow(unused)]
const REQ_IDENTIFY: u8 = 7;
#[allow(unused)]
const REQ_GET_USER_ID: u8 = 8;
#[allow(unused)]
const REQ_SET_USER_ID: u8 = 9;
const REQ_BIT_TIMING_DATA: u8 = 10;
const REQ_BIT_TIMING_CONST_EXT: u8 = 11;
#[allow(unused)]
const REQ_SET_TERMINATION: u8 = 12;
#[allow(unused)]
const REQ_GET_TERMINATION: u8 = 13;
const REQ_GET_STATE: u8 = 14;

/// Geschwister Schneider USB device.
pub struct GsCan<'a, B: UsbBus, D: Device> {
    interface: InterfaceNumber,
    write_endpoint: EndpointIn<'a, B>,
    read_endpoint: EndpointOut<'a, B>,
    pub device: D,
    /// Stateful store for incomming can frames. Has `Some` when a frame is
    /// partially read.
    in_frame: Option<host::Frame>,
    echo_frame: Option<host::Frame>,
    transmit_frame: Option<host::Frame>,
}

impl<'a, B: UsbBus, D: Device> GsCan<'a, B, D> {
    /// Crate a new GsUsb device.
    pub fn new(alloc: &'a UsbBusAllocator<B>, device: D) -> Self {
        // hack to get the out endpoint number right.
        let _: EndpointOut<'a, B> = alloc.bulk(0);

        Self {
            interface: alloc.interface(),
            write_endpoint: alloc.bulk(64),
            read_endpoint: alloc.bulk(64),
            device,
            in_frame: None,
            echo_frame: None,
            transmit_frame: None,
        }
    }

    /// Send a CAN frame to the host.
    pub fn transmit(&mut self, interface: u16, frame: &impl embedded_can::Frame) {
        let mut frame = if frame.is_remote_frame() {
            host::Frame::new_remote(frame.id(), frame.dlc()).unwrap()
        } else {
            host::Frame::new(frame.id(), frame.data()).unwrap()
        };

        frame.echo_id = u32::MAX; // set as receive frame
        frame.interface = interface as u8;

        if let Err(_err) = self.write_endpoint.write(&frame.as_bytes()[..64]) {
            #[cfg(feature = "defmt-03")]
            defmt::error!("{}", _err);
        }

        if frame.flags.intersects(FrameFlag::FD) {
            self.transmit_frame = Some(frame);
        }
    }
}

impl<B: UsbBus, D: Device> UsbClass<B> for GsCan<'_, B, D> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.interface(self.interface, INTERFACE_CLASS, 0xFF, 0xFF)?;
        writer.endpoint(&self.write_endpoint)?;
        writer.endpoint(&self.read_endpoint)?;

        Ok(())
    }

    // Handle control requests to the host.
    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = *xfer.request();

        if req.request_type != control::RequestType::Vendor {
            return;
        }

        match req.request {
            REQ_BIT_TIMING_CONST => {
                xfer.accept_with(self.device.bit_timing().as_bytes())
                    .unwrap();
            }
            REQ_DEVICE_CONFIG => {
                xfer.accept_with(self.device.config().as_bytes()).unwrap();
            }
            REQ_BIT_TIMING_CONST_EXT => {
                xfer.accept_with(self.device.bit_timing_ext().as_bytes())
                    .unwrap();
            }
            REQ_GET_STATE => {
                let interface = req.value as u8;
                xfer.accept_with(self.device.state(interface).as_bytes())
                    .unwrap();
            }
            _ => {
                #[cfg(feature = "defmt-03")]
                defmt::warn!("Unimplemented request kind: {}", req.request);
            }
        }
    }

    // Handle control requests from the host
    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = *xfer.request();

        if req.request_type != control::RequestType::Vendor {
            return;
        }

        match req.request {
            REQ_HOST_FORMAT => {
                let config = HostConfig::ref_from(xfer.data()).unwrap();
                assert_eq!(
                    config.byte_order, 0x0000beef,
                    "Byte order check mismatch. Big endian not currently supported.",
                );
                xfer.accept().unwrap();
            }
            REQ_BIT_TIMING => {
                let timing = DeviceBitTiming::read_from(xfer.data()).unwrap();
                let interface = req.value as u8;
                self.device.configure_bit_timing(interface, timing);
                xfer.accept().unwrap();
            }
            REQ_MODE => {
                let device_mode = DeviceMode::ref_from(xfer.data()).unwrap();
                let interface = req.value as u8;
                let mode = host::Mode::try_from(device_mode.mode).unwrap();
                match mode {
                    host::Mode::Reset => self.device.reset(interface),
                    host::Mode::Start => self.device.start(interface),
                }
                xfer.accept().unwrap();
            }
            REQ_BIT_TIMING_DATA => {
                let timing = DeviceBitTiming::read_from(xfer.data()).unwrap();
                let interface = req.value as u8;
                self.device.configure_bit_timing_data(interface, timing);
                xfer.accept().unwrap();
            }
            _ => {
                #[cfg(feature = "defmt-03")]
                defmt::warn!("Unimplemented request kind: {}", req.request);
                xfer.reject().ok();
            }
        }
    }

    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        // filter endpoint address.
        if addr.index() != 1 {
            return;
        }

        if let Some(frame) = self.echo_frame.take() {
            // only write to byte 76. we don't support timestamps yet.
            if let Err(_err) = self.write_endpoint.write(&frame.as_bytes()[64..76]) {
                #[cfg(feature = "defmt-03")]
                defmt::error!("{}", _err);
            }
        }

        if let Some(frame) = self.transmit_frame.take() {
            // only write to byte 76. we don't support timestamps yet.
            if let Err(_err) = self.write_endpoint.write(&frame.as_bytes()[64..76]) {
                #[cfg(feature = "defmt-03")]
                defmt::error!("{}", _err);
            }
        }
    }

    fn endpoint_out(&mut self, addr: EndpointAddress) {
        // filter endpoint address.
        if addr.index() != 2 {
            return;
        }

        let mut frame = match self.in_frame {
            None => {
                let mut frame = host::Frame::new_zeroed();
                self.read_endpoint
                    .read(&mut frame.as_bytes_mut()[..64])
                    .unwrap();

                if frame.flags.intersects(FrameFlag::FD) {
                    self.in_frame = Some(frame);
                    return;
                }

                frame
            }
            Some(mut frame) => {
                self.read_endpoint
                    .read(&mut frame.as_bytes_mut()[64..])
                    .unwrap();

                self.in_frame = None;

                frame
            }
        };

        if self.device.receive(frame.interface, frame).is_ok() {
            frame.echo_id = 0; // tx complete
            if let Err(_err) = self.write_endpoint.write(&frame.as_bytes()[..64]) {
                #[cfg(feature = "defmt-03")]
                defmt::error!("{}", _err);
            }

            if frame.flags.intersects(FrameFlag::FD) {
                self.echo_frame = Some(frame);
            }
        }
    }
}

pub trait Device {
    /// Returns the device configuration.
    ///
    /// Requested after reset by the host.
    fn config(&self) -> DeviceConfig;

    /// Returns the bit timing options.
    fn bit_timing(&self) -> DeviceBitTimingConst;

    /// Returns the extended bit timing options.
    fn bit_timing_ext(&self) -> DeviceBitTimingConstExtended;

    /// Called to configure the timing of the CAN interface.
    fn configure_bit_timing(&mut self, interface: u8, timing: DeviceBitTiming);

    /// Called to configure the timing of the CAN interface.
    fn configure_bit_timing_data(&mut self, interface: u8, timing: DeviceBitTiming);

    /// Called when the host requests an interface is reset.
    fn reset(&mut self, interface: u8);

    /// Called when the host requests an interface is started.
    fn start(&mut self, interface: u8);

    /// Returns the device state including TX and RX error counters.
    fn state(&self, interface: u8) -> DeviceState;

    /// Called when a frame is received from the host.
    // todo: error kinds.
    fn receive(&mut self, interface: u8, frame: host::Frame) -> Result<(), ()>;
}
