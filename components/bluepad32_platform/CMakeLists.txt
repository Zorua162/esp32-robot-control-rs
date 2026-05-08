idf_component_register(
    SRCS "bluepad32_platform.c"
    INCLUDE_DIRS "."
    REQUIRES bluepad32 btstack
)

# btstack keeps btstack_run_loop.h in src/ not include/
target_include_directories(__idf_bluepad32_platform PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../btstack/src
)