// Stubbed out to avoid legacy timer driver conflict with ESP-IDF v5.x
// Mouse quadrature is only needed for Unijoysticle platform, not custom platform.
#include "uni_mouse_quadrature.h"

void uni_mouse_quadrature_init(int cpu_id) {}

void uni_mouse_quadrature_setup_port(int port_idx,
                                     struct uni_mouse_quadrature_encoder_gpios h,
                                     struct uni_mouse_quadrature_encoder_gpios v) {}

void uni_mouse_quadrature_update(int port_idx, int32_t dx, int32_t dy) {}

void uni_mouse_quadrature_start(int port_idx) {}

void uni_mouse_quadrature_pause(int port_idx) {}

void uni_mouse_quadrature_deinit(void) {}

void uni_mouse_quadrature_set_scale_factor(float scale) {}

float uni_mouse_quadrature_get_scale_factor(void) { return 1.0f; }