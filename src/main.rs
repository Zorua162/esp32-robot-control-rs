use esp_idf_hal::sys::EspError;
use esp_idf_hal::{
    gpio::{Output, PinDriver, Pins},
    peripherals::Peripherals,
    rmt::{config::TxChannelConfig, encoder::CopyEncoder, TxChannelDriver},
    units::Hertz,
};

use esp_idf_svc::hal::gpio::{Gpio1, Gpio2, Gpio3, Gpio4};
use esp_idf_svc::sys;
use std::thread;
use std::time::Duration;

mod neopixel;

use crate::neopixel::{set_neopixel_colour, RGB};

const RMT_RESOLUTION: Hertz = Hertz(10_000_000);

const DEAD_ZONE: i32 = 100;

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

pub struct MotorPins<'d> {
    pub in1: PinDriver<'d, Output>,
    pub in2: PinDriver<'d, Output>,
    pub in3: PinDriver<'d, Output>,
    pub in4: PinDriver<'d, Output>,
}

impl<'d> MotorPins<'d> {
    pub fn new(
        gpio1: Gpio1<'d>,
        gpio2: Gpio2<'d>,
        gpio3: Gpio3<'d>,
        gpio4: Gpio4<'d>,
    ) -> Result<Self, EspError> {
        Ok(Self {
            // GPIO1 - in1
            // GPIO2 - in2
            // GPIO3 - in3
            // GPIO4 - in4
            in1: PinDriver::output(gpio1)?,
            in2: PinDriver::output(gpio2)?,
            in3: PinDriver::output(gpio3)?,
            in4: PinDriver::output(gpio4)?,
        })
    }
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
    let Pins {
        gpio1,
        gpio2,
        gpio3,
        gpio4,
        gpio21,
        ..
    } = peripherals.pins;

    let led_pin = gpio21;

    let mut tx_channel = TxChannelDriver::new(
        led_pin,
        &TxChannelConfig {
            resolution: RMT_RESOLUTION,
            ..Default::default()
        },
    )?;

    let mut encoder = CopyEncoder::new()?;

    set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::red())?;
    // Allow time for BT to start and show if a crash happens

    thread::sleep(Duration::from_millis(500));
    set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::blue())?;

    let mut motor = MotorPins::new(gpio1, gpio2, gpio3, gpio4)?;

    loop {
        let state = unsafe { bluepad32_get_gamepad_state() };

        if state.connected {
            set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::green())?;
            println!(
                "left=({}, {}) right=({}, {}) buttons={:#010b} dpad={}",
                state.axis_x, state.axis_y, state.axis_rx, state.axis_ry, state.buttons, state.dpad,
            );

            do_movement(&state, &mut motor)?;

            match state.axis_x {
                -512..1 => set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::purple()),
                1..512 => set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::yellow()),
                _ => Ok(()),
            }?
        } else {
            // Slow pulse blue = waiting for controller
            set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::green())?;
            thread::sleep(Duration::from_millis(400));
            set_neopixel_colour(&mut tx_channel, &mut encoder, RGB::off())?;
            thread::sleep(Duration::from_millis(100));
        }

        thread::sleep(Duration::from_millis(50)); // ~20hz
    }
}

fn do_movement(state: &GamepadState, motor: &mut MotorPins) -> Result<(), EspError> {
    // Figure out where the controller is pointing:

    // If up or down is greater than left or right then its forward or backwards
    // Otherwise we need to turn

    if i32::abs(state.axis_y) > i32::abs(state.axis_y) {
        do_forward_backward(state, motor)?;
    } else {
        do_turn(state, motor)?;
    };

    Ok(())
}

fn do_forward_backward(state: &GamepadState, motor: &mut MotorPins) -> Result<(), EspError> {
    match state.axis_y {
        n if n > -512 && n < -DEAD_ZONE => {
            // Backwards
            motor.in1.set_low()?;
            motor.in2.set_high()?;

            motor.in3.set_low()?;
            motor.in4.set_high()?;
        }
        n if n > -DEAD_ZONE && n < DEAD_ZONE => {
            // STOP
            motor.in1.set_high()?;
            motor.in2.set_high()?;

            motor.in3.set_high()?;
            motor.in4.set_high()?;
        }
        n if n > DEAD_ZONE && n < 512 => {
            // Forward!
            motor.in1.set_high()?;
            motor.in2.set_low()?;

            motor.in3.set_high()?;
            motor.in4.set_low()?;
        }
        _ => (),
    };
    Ok(())
}

fn do_turn(state: &GamepadState, motor: &mut MotorPins) -> Result<(), EspError> {
    match state.axis_y {
        n if n > -512 && n < -DEAD_ZONE => {
            // Left?
            motor.in1.set_high()?;
            motor.in2.set_low()?;

            motor.in3.set_low()?;
            motor.in4.set_high()?;
        }
        n if n > -DEAD_ZONE && n < DEAD_ZONE => {
            // STOP
            motor.in1.set_high()?;
            motor.in2.set_high()?;

            motor.in3.set_high()?;
            motor.in4.set_high()?;
        }
        n if n > DEAD_ZONE && n < 512 => {
            // Forward!
            motor.in1.set_low()?;
            motor.in2.set_high()?;

            motor.in3.set_high()?;
            motor.in4.set_low()?;
        }
        _ => (),
    };
    Ok(())
}
