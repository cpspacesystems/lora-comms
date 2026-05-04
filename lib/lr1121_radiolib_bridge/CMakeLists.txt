cmake_minimum_required(VERSION 3.13)
project(lora_bridge)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)
add_subdirectory("${CMAKE_SOURCE_DIR}/../RadioLib" RadioLib)

add_library(lora_bridge STATIC wrapper.cpp)

set_property(TARGET lora_bridge PROPERTY CXX_STANDARD 20)

# find lpgio
# find_path(LGPIO_INCLUDE_DIR lgpio.h "/usr/local/include" "/usr/include" "/opt/include" "${CMAKE_SOURCE_DIR}/../installed/include")
# find_library(LGPIO_LIBS liblgpio HINTS "/usr/local/lib" /usr/lib)
# if (NOT "${LGPIO_INCLUDE_DIR}")
    # message(NOTICE "lgpio.h not found, manually compiling lgpio!")
    execute_process(
        COMMAND bash "${CMAKE_SOURCE_DIR}/get_lgpio.bash"
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    )
    set(LGPIO_INCLUDE_DIR PATH "${CMAKE_BINARY_DIR}/lg")
    file(GLOB LGPIO_SOURCE "${CMAKE_BINARY_DIR}/lg/lg*.c")
    add_library(lgpio STATIC ${LGPIO_SOURCE})
    target_include_directories(lgpio PUBLIC ${LGPIO_INCLUDE_DIR})
    set(LGPIO_LIBS "lgpio")
# endif()

target_include_directories(lora_bridge 
    PUBLIC "${CMAKE_SOURCE_DIR}/../RadioLib/src"
    PUBLIC "${CMAKE_SOURCE_DIR}/../RadioLib/src/hal/RPi"
    PUBLIC "${LGPIO_INCLUDE_DIR}"
    PUBLIC "${CMAKE_SOURCE_DIR}"
)

target_link_libraries(lora_bridge 
    PUBLIC RadioLib
    PUBLIC ${LGPIO_LIBS}
)

# add_custom_command(TARGET lora_bridge POST_BUILD
# 	COMMAND ${CMAKE_SOURCE_DIR}/merge.sh
#         $<TARGET_FILE:lora_bridge>
#         $<TARGET_FILE:RadioLib>
#         ${CMAKE_CURRENT_BINARY_DIR}/liblora_full.a
#     COMMENT "Merging into liblora_full.a"
# )
