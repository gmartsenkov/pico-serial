#![no_std]
#![no_main]

use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use fugit::ExtU32;
use panic_halt as _;
use rp_pico::{entry, hal};
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_human_interface_device::{device::joystick::JoystickReport, prelude::*};

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

    let mut joy = UsbHidClassBuilder::new()
        .add_device(usbd_human_interface_device::device::joystick::JoystickConfig::default())
        .build(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("gogo")
        .product("cool pedals")
        .serial_number("G-CP000X")
        .build();

    let mut input_count_down = timer.count_down();
    input_count_down.start(10.millis());

    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();

    loop {
        if input_count_down.wait().is_ok() {
            let report = JoystickReport {
                buttons: 0,
                x: 1,
                y: 1,
            };
            match joy.device().write_report(&report) {
                Err(UsbHidError::WouldBlock) => {}
                Ok(_) => {}
                Err(_) => {
                    core::panic!("Failed to write joystic report: {:?}", report)
                }
            }
        }

        if usb_dev.poll(&mut [&mut joy]) {}
    }
}
