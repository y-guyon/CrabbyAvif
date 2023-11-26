cmake_minimum_required(VERSION 3.10)

project(rust_libavif_c_api_tests)

set(CMAKE_C_COMPILER "clang")
set(CMAKE_CXX_COMPILER "clang++")

set(GTEST_INCLUDE_DIR "/Users/vigneshv/code/libavif/ext/googletest/googletest/include")
set(RUST_LIBAVIF_INCLUDE_DIR "/Users/vigneshv/code/rust_pg/rust-libavif/c_build/usr/include/")

set(GTEST_LIBRARIES "/Users/vigneshv/code/libavif/ext/googletest/build/lib/libgtest.a")
set(GTEST_MAIN_LIBRARIES "/Users/vigneshv/code/libavif/ext/googletest/build/lib/libgtest_main.a")
set(RUST_LIBAVIF_LIBRARIES "/Users/vigneshv/code/rust_pg/rust-libavif/c_build/usr/lib/librust_libavif.a")

macro(add_avif_gtest TEST_NAME)
    add_executable(${TEST_NAME} ${TEST_NAME}.cc)
    target_include_directories(${TEST_NAME} PRIVATE ${GTEST_INCLUDE_DIR})
    target_include_directories(${TEST_NAME} PRIVATE ${RUST_LIBAVIF_INCLUDE_DIR})
    target_link_libraries(${TEST_NAME} PRIVATE ${GTEST_LIBRARIES})
    target_link_libraries(${TEST_NAME} PRIVATE ${GTEST_MAIN_LIBRARIES})
    target_link_libraries(${TEST_NAME} PRIVATE ${RUST_LIBAVIF_LIBRARIES})
    add_test(NAME ${TEST_NAME} COMMAND ${TEST_NAME})
endmacro()

add_avif_gtest(avifalphanoispetest)
add_avif_gtest(avifanimationtest)
add_avif_gtest(avifdecodetest)