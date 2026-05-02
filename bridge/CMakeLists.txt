cmake_minimum_required(VERSION 3.13)
project(lora_bridge)

add_subdirectory(.. RadioLib)

add_library(lora_bridge STATIC wrapper.cpp)

set_property(TARGET lora_bridge PROPERTY CXX_STANDARD 20)

target_include_directories(lora_bridge PUBLIC
    ../src
    ../src/hal/RPi
    .
)

target_link_libraries(lora_bridge PRIVATE RadioLib)

add_custom_command(TARGET lora_bridge POST_BUILD
	COMMAND ${CMAKE_SOURCE_DIR}/merge.sh
        $<TARGET_FILE:lora_bridge>
        $<TARGET_FILE:RadioLib>
        ${CMAKE_CURRENT_BINARY_DIR}/liblora_full.a
    COMMENT "Merging into liblora_full.a"
)
