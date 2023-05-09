//!HID pedal
use core::default::Default;
use defmt::{error, unwrap};
use fugit::ExtU32;
use packed_struct::prelude::*;
use usb_device::bus::UsbBus;
use usb_device::class_prelude::UsbBusAllocator;
use usbd_human_interface_device::usb_class::prelude::*;

#[rustfmt::skip]
pub const PEDAL_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)         5,   1
    0x09, 0x08, // Usage (Multi axies)                  9,   8
    0xa1, 0x01, // Collection (Application)             161, 1
    0x09, 0x01, //   Usage Page (Pointer)               9,   1
    0xa1, 0x00, //   Collection (Physical)              161, 0
    0x09, 0x31, //     Usage (Y)                        9,   49
    0x15, 0x81, //     Logical Minimum (-127)           21,  129
    0x25, 0x7f, //     Logical Maximum (127)            37,  127
    0x75, 0x08, //     Report Size (8)                  117, 8
    0x95, 0x02, //     Report count (1)                 149, 2,
    0x81, 0x02, //     Input (Data, Variable, Absolute) 129, 2,
    0xc0,       // End Collection                       192
    0xc0,       // End Collection                       192
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, PackedStruct)]
#[packed_struct(endian = "lsb", size_bytes = "1")]
pub struct PedalReport {
    #[packed_field]
    pub y: i8,
}

pub struct Pedal<'a, B: UsbBus> {
    interface: Interface<'a, B, InBytes8, OutNone, ReportSingle>,
}

impl<'a, B: UsbBus> Pedal<'a, B> {
    pub fn write_report(&mut self, report: &PedalReport) -> Result<(), UsbHidError> {
        let data = report.pack().map_err(|_| {
            error!("Error packing PedalReport");
            UsbHidError::SerializationError
        })?;
        self.interface
            .write_report(&data)
            .map(|_| ())
            .map_err(UsbHidError::from)
    }
}

impl<'a, B: UsbBus> DeviceClass<'a> for Pedal<'a, B> {
    type I = Interface<'a, B, InBytes8, OutNone, ReportSingle>;

    fn interface(&mut self) -> &mut Self::I {
        &mut self.interface
    }

    fn reset(&mut self) {}

    fn tick(&mut self) -> Result<(), UsbHidError> {
        Ok(())
    }
}

pub struct PedalConfig<'a> {
    interface: InterfaceConfig<'a, InBytes8, OutNone, ReportSingle>,
}

impl<'a> Default for PedalConfig<'a> {
    #[must_use]
    fn default() -> Self {
        Self::new(
            unwrap!(unwrap!(InterfaceBuilder::new(PEDAL_DESCRIPTOR))
                .boot_device(InterfaceProtocol::None)
                .description("Pedal")
                .in_endpoint(10.millis()))
            .without_out_endpoint()
            .build(),
        )
    }
}

impl<'a> PedalConfig<'a> {
    #[must_use]
    pub fn new(interface: InterfaceConfig<'a, InBytes8, OutNone, ReportSingle>) -> Self {
        Self { interface }
    }
}

impl<'a, B: UsbBus + 'a> UsbAllocatable<'a, B> for PedalConfig<'a> {
    type Allocated = Pedal<'a, B>;

    fn allocate(self, usb_alloc: &'a UsbBusAllocator<B>) -> Self::Allocated {
        Self::Allocated {
            interface: Interface::new(usb_alloc, self.interface),
        }
    }
}
