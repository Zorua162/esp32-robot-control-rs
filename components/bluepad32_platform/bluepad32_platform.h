#pragma once
#include <stdint.h>
#include <stdbool.h>

// Simple gamepad state readable from Rust
typedef struct {
    int32_t axis_x;
    int32_t axis_y;
    int32_t axis_rx;
    int32_t axis_ry;
    int32_t brake;
    int32_t throttle;
    uint16_t buttons;
    uint8_t dpad;
    bool connected;
} gamepad_state_t;

// Called from Rust main to initialise the whole BT + bluepad32 stack
void bluepad32_platform_run(void);

// Called from Rust to poll the latest gamepad state (gamepad 0)
gamepad_state_t bluepad32_get_gamepad_state(void);