# CMake generated Testfile for 
# Source directory: /home/yguyon/git/CrabbyAvif/c_api_tests
# Build directory: /home/yguyon/git/CrabbyAvif/build
# 
# This file includes the relevant testing commands required for 
# testing this directory and lists subdirectories to be tested as well.
add_test(decoder_tests "/home/yguyon/git/CrabbyAvif/build/decoder_tests" "/home/yguyon/git/CrabbyAvif/tests/data/")
set_tests_properties(decoder_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;47;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;50;add_avif_gtest;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
add_test(encoder_tests "/home/yguyon/git/CrabbyAvif/build/encoder_tests" "/home/yguyon/git/CrabbyAvif/tests/data/")
set_tests_properties(encoder_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;47;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;51;add_avif_gtest;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
add_test(gainmap_tests "/home/yguyon/git/CrabbyAvif/build/gainmap_tests" "/home/yguyon/git/CrabbyAvif/tests/data/")
set_tests_properties(gainmap_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;47;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;52;add_avif_gtest;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
add_test(image_tests "/home/yguyon/git/CrabbyAvif/build/image_tests" "/home/yguyon/git/CrabbyAvif/tests/data/")
set_tests_properties(image_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;47;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;53;add_avif_gtest;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
add_test(incremental_tests "/home/yguyon/git/CrabbyAvif/build/incremental_tests" "/home/yguyon/git/CrabbyAvif/tests/data/")
set_tests_properties(incremental_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;47;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;54;add_avif_gtest;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
add_test(reformat_tests "/home/yguyon/git/CrabbyAvif/build/reformat_tests" "/home/yguyon/git/CrabbyAvif/tests/data/")
set_tests_properties(reformat_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;47;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;55;add_avif_gtest;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
add_test(conformance_tests "/home/yguyon/git/CrabbyAvif/build/conformance_tests" "/home/yguyon/git/CrabbyAvif/external/av1-avif/testFiles/")
set_tests_properties(conformance_tests PROPERTIES  _BACKTRACE_TRIPLES "/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;64;add_test;/home/yguyon/git/CrabbyAvif/c_api_tests/CMakeLists.txt;0;")
