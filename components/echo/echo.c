#include "echo.h"
#include "esp_log.h"

static const char* TAG = "echo";

void echo_print(void) {
    ESP_LOGI(TAG, "echo - updated c code");
}