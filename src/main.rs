#![no_std]
#![no_main]

use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use fugit::ExtU32;
use panic_halt as _;
use rp_pico::{entry, hal};
use serde::{Deserialize, Serialize};
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[derive(Deserialize, Serialize)]
struct Config {
    load_cell_kg: bool,
}

#[entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);

    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("gogo")
        .product("cool pedals")
        .serial_number("G-CP000X")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut input_count_down = timer.count_down();
    input_count_down.start(1000.millis());

    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();

    let mut buffer = [0; 64];
    let mut write_buffer = [0; 64];

    loop {
        if input_count_down.wait().is_ok() {
            match serial.read(&mut buffer[..]) {
                Ok(_) => {
                    serial.write(&buffer).ok();
                    match serde_json_core::de::from_slice::<Config>(&buffer) {
                        Ok(config) => {
                            serde_json_core::ser::to_slice(&config, &mut write_buffer).unwrap();
                            serial.write(&write_buffer).ok()
                        }
                        Err(e) => serial.write(json_error_to_str(e).as_bytes()).ok(),
                    }
                }
                Err(_) => Some(0),
            };
        }

        if usb_dev.poll(&mut [&mut serial]) {}
    }
}

fn json_error_to_str(e: serde_json_core::de::Error) -> &'static str {
    match e {
        serde_json_core::de::Error::EofWhileParsingList => "EOF while parsing a list.",
        serde_json_core::de::Error::EofWhileParsingObject => "EOF while parsing an object.",
        serde_json_core::de::Error::EofWhileParsingString => "EOF while parsing a string.",
        serde_json_core::de::Error::EofWhileParsingValue => "EOF while parsing a JSON value.",
        serde_json_core::de::Error::ExpectedColon => "Expected this character to be a `':'`.",
        serde_json_core::de::Error::ExpectedListCommaOrEnd => {
            "Expected this character to be either a `','` or\
                     a \
                     `']'`."
        }
        serde_json_core::de::Error::ExpectedObjectCommaOrEnd => {
            "Expected this character to be either a `','` \
                     or a \
                     `'}'`."
        }
        serde_json_core::de::Error::ExpectedSomeIdent => {
            "Expected to parse either a `true`, `false`, or a \
                     `null`."
        }
        serde_json_core::de::Error::ExpectedSomeValue => {
            "Expected this character to start a JSON value."
        }
        serde_json_core::de::Error::InvalidNumber => "Invalid number.",
        serde_json_core::de::Error::InvalidType => "Invalid type",
        serde_json_core::de::Error::InvalidUnicodeCodePoint => "Invalid unicode code point.",
        serde_json_core::de::Error::KeyMustBeAString => "Object key is not a string.",
        serde_json_core::de::Error::TrailingCharacters => {
            "JSON has non-whitespace trailing characters after \
                     the \
                     value."
        }
        serde_json_core::de::Error::TrailingComma => {
            "JSON has a comma after the last value in an array or map."
        }
        serde_json_core::de::Error::CustomError => {
            "JSON does not match deserializer’s expected format."
        }
        _ => unimplemented!()
    }
}
