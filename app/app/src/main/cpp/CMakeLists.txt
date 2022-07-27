cmake_minimum_required(VERSION 3.18.1)

project("quicklock")

# Define empty pthread library to fix relic build without modifying it.
# Android's pthread implementation is in libc.
add_library(pthread INTERFACE)

# Deps
set(BUILD_BLS_PYTHON_BINDINGS false)
set(BUILD_BLS_TESTS false)
set(BUILD_BLS_BENCHMARKS false)
add_subdirectory(external/bls)

add_library(quicklock SHARED
        quicklock.cpp)

find_library(log-lib log)

target_link_libraries(quicklock ${log-lib} bls)
