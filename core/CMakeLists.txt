cmake_minimum_required(VERSION 4.1)

project(core VERSION 0.0.1 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

list(APPEND CMAKE_PREFIX_PATH "/opt/grpc/install")

find_package(protobuf REQUIRED)
find_package(gRPC REQUIRED)

add_subdirectory(apps)