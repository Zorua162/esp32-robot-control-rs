use esp_idf_hal::{
    peripherals::Peripherals,
    rmt::{config::TxChannelConfig, encoder::CopyEncoder, TxChannelDriver},
    units::Hertz,
};

use esp_idf_svc::sys;
use std::thread;
use std::time::Duration;

mod neopixel;

use crate::neopixel::{set_neopixel_colour, RGB};

const RMT_RESOLUTION: Hertz = Hertz(10_000_000);

#[repr(C)]
pub struct GamepadState {
    pub axis_x: i32,
    pub axis_y: i32,
    pub axis_rx: i32,
    pub axis_ry: i32,
    pub brake: i32,
    pub throttle: i32,
    pub buttons: u16,
    pub dpad: u8,
    pub connected: bool,
}

extern "C" {
    fn bluepad32_platform_run();
    fn bluepad32_get_gamepad_state() -> GamepadState;
}

fn main() -> anyhow::Result<()> {
    sys::link_patches();

    // Spawn our robot logic on a Rust thread BEFORE handing
    // control to BTstack. This thread runs concurrently with
    // the BT stack via FreeRTOS scheduling.
    thread::spawn(|| {
        let _ = robot_loop();
    });

    // Hand control to BTstack — never returns.
    unsafe { bluepad32_platform_run() };

    Ok(())
}

fn robot_loop() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let led_pin = peripherals.pins.gpio8;

    let mut tx_channel = TxChannelDriver::new(
        led_pin,
        &TxChannelConfig {
            resolution: RMT_RESOLUTION,
            ..Default::default()
        },
    )?;

    let mut encoder = CopyEncoder::new()?;

    set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::red())?;

    loop {
        let state = unsafe { bluepad32_get_gamepad_state() };

        if state.connected {
            set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::green())?;
            println!(
                "left=({}, {}) right=({}, {}) buttons={:#010b} dpad={}",
                state.axis_x, state.axis_y, state.axis_rx, state.axis_ry, state.buttons, state.dpad,
            );

            match state.axis_x {
                -512..0 => set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::purple()),
                1..512 => set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::yellow()),
                _ => Ok(()),
            }?
        } else {
            println!("Waiting for controller...");
        }

        thread::sleep(Duration::from_millis(50)); // ~20hz
    }
}
