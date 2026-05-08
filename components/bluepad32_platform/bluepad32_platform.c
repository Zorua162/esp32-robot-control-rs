#include "bluepad32_platform.h"

#include <string.h>
#include <btstack_port_esp32.h>
#include <btstack_run_loop.h>
#include <btstack_stdio_esp32.h>
#include <uni.h>
#include "sdkconfig.h"

// Shared gamepad state updated by the BT callback, read by Rust
static gamepad_state_t s_gamepad_state = {0};

// --- Platform callbacks ---

static void platform_init(int argc, const char** argv) {
    (void)argc; (void)argv;
}

static void platform_on_init_complete(void) {
    uni_bt_start_scanning_and_autoconnect_unsafe();
    uni_bt_allow_incoming_connections(true);
    uni_bt_del_keys_unsafe();
}

static uni_error_t platform_on_device_discovered(bd_addr_t addr, const char* name,
                                                   uint16_t cod, uint8_t rssi) {
    (void)addr; (void)name; (void)cod; (void)rssi;
    return UNI_ERROR_SUCCESS;
}

static void platform_on_device_connected(uni_hid_device_t* d) {
    (void)d;
}

static void platform_on_device_disconnected(uni_hid_device_t* d) {
    (void)d;
    s_gamepad_state.connected = false;
}

static uni_error_t platform_on_device_ready(uni_hid_device_t* d) {
    (void)d;
    s_gamepad_state.connected = true;
    return UNI_ERROR_SUCCESS;
}

static void platform_on_controller_data(uni_hid_device_t* d, uni_controller_t* ctl) {
    (void)d;
    if (ctl->klass != UNI_CONTROLLER_CLASS_GAMEPAD) return;

    uni_gamepad_t* gp = &ctl->gamepad;
    s_gamepad_state.axis_x   = gp->axis_x;
    s_gamepad_state.axis_y   = gp->axis_y;
    s_gamepad_state.axis_rx  = gp->axis_rx;
    s_gamepad_state.axis_ry  = gp->axis_ry;
    s_gamepad_state.brake    = gp->brake;
    s_gamepad_state.throttle = gp->throttle;
    s_gamepad_state.buttons  = gp->buttons;
    s_gamepad_state.dpad     = gp->dpad;
}

static const uni_property_t* platform_get_property(uni_property_idx_t idx) {
    (void)idx;
    return NULL;
}

static void platform_on_oob_event(uni_platform_oob_event_t event, void* data) {
    (void)event; (void)data;
}

static struct uni_platform s_platform = {
    .name                  = "rust_robot",
    .init                  = platform_init,
    .on_init_complete      = platform_on_init_complete,
    .on_device_discovered  = platform_on_device_discovered,
    .on_device_connected   = platform_on_device_connected,
    .on_device_disconnected= platform_on_device_disconnected,
    .on_device_ready       = platform_on_device_ready,
    .on_controller_data    = platform_on_controller_data,
    .get_property          = platform_get_property,
    .on_oob_event          = platform_on_oob_event,
};

// --- Public API ---

gamepad_state_t bluepad32_get_gamepad_state(void) {
    return s_gamepad_state;
}

void bluepad32_platform_run(void) {
#ifdef CONFIG_ESP_CONSOLE_UART
    btstack_stdio_init();
#endif
    btstack_init();
    uni_platform_set_custom(&s_platform);
    uni_init(0, NULL);
    btstack_run_loop_execute(); // does not return
}