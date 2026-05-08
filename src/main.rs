use esp_idf_svc::sys;
use std::thread;
use std::time::Duration;

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

fn main() {
    sys::link_patches();

    // Spawn our robot logic on a Rust thread BEFORE handing
    // control to BTstack. This thread runs concurrently with
    // the BT stack via FreeRTOS scheduling.
    thread::spawn(|| {
        robot_loop();
    });

    // Hand control to BTstack — never returns.
    unsafe { bluepad32_platform_run() };
}

fn robot_loop() {
    loop {
        let state = unsafe { bluepad32_get_gamepad_state() };

        if state.connected {
            println!(
                "left=({}, {}) right=({}, {}) buttons={:#010b} dpad={}",
                state.axis_x, state.axis_y, state.axis_rx, state.axis_ry, state.buttons, state.dpad,
            );
        } else {
            println!("Waiting for controller...");
        }

        thread::sleep(Duration::from_millis(50)); // ~20hz
    }
}
