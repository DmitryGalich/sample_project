FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    software-properties-common \
    wget \
    curl \
    git \
    ninja-build \
    pkg-config \
    build-essential \
    python3 \
    python3-pip \
    unzip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# LLVM
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 20 && \
    rm llvm.sh

ENV CC=clang-20
ENV CXX=clang++-20

# CMake
RUN wget https://github.com/Kitware/CMake/releases/download/v4.1.0/cmake-4.1.0-linux-x86_64.sh && \
    chmod +x cmake-4.1.0-linux-x86_64.sh && \
    ./cmake-4.1.0-linux-x86_64.sh --skip-license --prefix=/usr/local && \
    rm cmake-4.1.0-linux-x86_64.sh

WORKDIR /opt

# protobuf
RUN git clone --depth 1 https://github.com/protocolbuffers/protobuf.git && \
    cd protobuf && \
    cmake -S . -B build \
        -GNinja \
        -Dprotobuf_BUILD_TESTS=OFF \
        -DCMAKE_BUILD_TYPE=Release && \
    cmake --build build -j && \
    cmake --install build

# grpc
RUN git clone --depth 1 --recurse-submodules \
    https://github.com/grpc/grpc.git && \
    cd grpc && \
    cmake -S . -B build \
        -GNinja \
        -DgRPC_BUILD_TESTS=OFF \
        -DgRPC_INSTALL=ON \
        -DgRPC_PROTOBUF_PROVIDER=package \
        -DCMAKE_BUILD_TYPE=Release && \
    cmake --build build -j && \
    cmake --install build

ENV CMAKE_PREFIX_PATH="/opt/grpc/install:${CMAKE_PREFIX_PATH}"
ENV PATH="/opt/grpc/install/bin:${PATH}"
ENV LD_LIBRARY_PATH="/opt/grpc/install/lib:${LD_LIBRARY_PATH}"




WORKDIR /workspace