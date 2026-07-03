cmake_minimum_required(VERSION 4.2.3)

project(core VERSION 0.0.1 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 26)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(PkgConfig REQUIRED)
pkg_check_modules(PROTOBUF REQUIRED protobuf)
pkg_check_modules(GRPC REQUIRED grpc++)

find_program(PROTOC_PATH protoc REQUIRED)
find_program(GRPC_CPP_PLUGIN_PATH grpc_cpp_plugin REQUIRED)

add_subdirectory(apps)