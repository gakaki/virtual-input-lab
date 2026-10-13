cmake_minimum_required(VERSION 3.13)

# Resolve the Pico SDK: PICO_SDK_PATH (cache or environment) must point at a
# pico-sdk checkout. Copy pico_sdk_import.cmake from the SDK into this folder
# once: cp "$PICO_SDK_PATH/external/pico_sdk_import.cmake" .
if(NOT DEFINED PICO_SDK_PATH AND DEFINED ENV{PICO_SDK_PATH})
  set(PICO_SDK_PATH "$ENV{PICO_SDK_PATH}")
endif()

if(NOT EXISTS "${CMAKE_CURRENT_LIST_DIR}/pico_sdk_import.cmake")
  message(FATAL_ERROR
    "pico_sdk_import.cmake is missing. Copy it from the SDK:\n"
    "  cp \"$PICO_SDK_PATH/external/pico_sdk_import.cmake\" ${CMAKE_CURRENT_LIST_DIR}/")
endif()

include(${CMAKE_CURRENT_LIST_DIR}/pico_sdk_import.cmake)

project(rp2040_ds4 C CXX ASM)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)

pico_sdk_init()

add_executable(rp2040_ds4
  main.c
  usb_descriptors.c
)

target_include_directories(rp2040_ds4 PRIVATE
  ${CMAKE_CURRENT_LIST_DIR}
  ${CMAKE_CURRENT_LIST_DIR}/generated
)

target_link_libraries(rp2040_ds4 PRIVATE
  pico_stdlib
  tinyusb_device
)

# The firmware ships its own CDC interface, so the SDK USB stdio console must
# stay disabled. UART stdio stays available for debugging.
pico_enable_stdio_usb(rp2040_ds4 0)
pico_enable_stdio_uart(rp2040_ds4 1)

pico_add_extra_outputs(rp2040_ds4)
