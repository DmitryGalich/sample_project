cmake_minimum_required(VERSION 4.1)

project(core VERSION 0.0.1 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(userver REQUIRED COMPONENTS core grpc)

userver_add_grpc_library(messenger_proto
    PROTOS "${CMAKE_CURRENT_SOURCE_DIR}/proto/transport.proto"
    INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/proto"
)

add_subdirectory(apps)
