use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver, Pins};
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{self, EspError}; // gives you 50.Hz()

use std::thread;
use std::time::Duration;

mod neopixel;
mod servo;

use crate::neopixel::{NeoPixelControl, RGB};
use crate::servo::Servo;

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
    in1: PinDriver<'d, Output>,
    in2: PinDriver<'d, Output>,
    in3: PinDriver<'d, Output>,
    in4: PinDriver<'d, Output>,
}

impl<'d> MotorPins<'d> {
    pub fn new(
        gpio1: impl OutputPin + 'd,
        gpio2: impl OutputPin + 'd,
        gpio3: impl OutputPin + 'd,
        gpio4: impl OutputPin + 'd,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Drive {
    Stop,
    Forward,
    Backward,
    Left,
    Right,
}

impl MotorPins<'_> {
    fn apply(&mut self, drive: Drive) -> Result<(), EspError> {
        let (a1, a2, b1, b2) = match drive {
            Drive::Forward => (true, false, true, false),
            Drive::Backward => (false, true, false, true),
            Drive::Left => (true, false, false, true),
            Drive::Right => (false, true, true, false),
            Drive::Stop => (true, true, true, true), // brake
        };
        self.in1.set_level(a1.into())?;
        self.in2.set_level(a2.into())?;
        self.in3.set_level(b1.into())?;
        self.in4.set_level(b2.into())
    }
}

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    EspLogger::initialize_default();

    // Spawn our robot logic on a Rust thread BEFORE handing
    // control to BTstack. This thread runs concurrently with
    // the BT stack via FreeRTOS scheduling.
    thread::Builder::new().stack_size(8192).spawn(|| {
        if let Err(e) = robot_loop() {
            log::error!("robot_loop exited: {e:?}");
        }
    })?;

    // Hand control to BTstack — never returns.
    unsafe { bluepad32_platform_run() };

    Ok(())
}

fn robot_loop() -> anyhow::Result<()> {
    let Peripherals { pins, ledc, .. } = Peripherals::take()?;
    let Pins {
        gpio1,
        gpio2,
        gpio3,
        gpio4,
        gpio10,
        gpio21,
        ..
    } = pins;

    let led_pin = gpio21;

    let mut pixel: NeoPixelControl = NeoPixelControl::new(led_pin)?;

    let timer = LedcTimerDriver::new(
        ledc.timer0,
        &TimerConfig::new()
            .frequency(50.Hz().into())
            .resolution(Resolution::Bits14),
    )?;
    let mut servo = Servo::new(LedcDriver::new(ledc.channel0, &timer, gpio10)?)?;

    // Allow time for BT to start and show if a crash happens
    thread::sleep(Duration::from_millis(500));
    pixel.set_colour(RGB::blue())?;

    let mut motor = MotorPins::new(gpio1, gpio2, gpio3, gpio4)?;

    loop {
        let state = unsafe { bluepad32_get_gamepad_state() };

        if state.connected {
            log::debug!(
                "left=({}, {}) right=({}, {}) buttons={:#010b} dpad={}",
                state.axis_x,
                state.axis_y,
                state.axis_rx,
                state.axis_ry,
                state.buttons,
                state.dpad,
            );

            let drive = choose_drive(&state);
            motor.apply(drive)?;
            pixel.set_colour(match drive {
                Drive::Forward | Drive::Backward => RGB::yellow(),
                Drive::Left | Drive::Right => RGB::purple(),
                Drive::Stop => RGB::green(),
            })?;

            let angle = ((state.axis_rx.clamp(-512, 512) + 512) * 180 / 1024) as u32;
            servo.set_angle(angle)?;
        } else {
            motor.apply(Drive::Stop)?;
            // Slow pulse blue = waiting for controller
            pixel.set_colour(RGB::blue())?;
            thread::sleep(Duration::from_millis(400));
            pixel.set_colour(RGB::off())?;
            thread::sleep(Duration::from_millis(100));
        }

        thread::sleep(Duration::from_millis(50)); // ~20hz
    }
}

fn choose_drive(state: &GamepadState) -> Drive {
    let (x, y) = (state.axis_x, state.axis_y);
    if y.abs() >= x.abs() {
        match y {
            n if n < -DEAD_ZONE => Drive::Backward,
            n if n > DEAD_ZONE => Drive::Forward,
            _ => Drive::Stop,
        }
    } else {
        match x {
            n if n < -DEAD_ZONE => Drive::Left,
            n if n > DEAD_ZONE => Drive::Right,
            _ => Drive::Stop,
        }
    }
}
