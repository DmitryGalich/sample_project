FROM ubuntu:26.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    gdb \
    git \
    pkg-config \
    libssl-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /tmp
RUN git clone --recurse-submodules -b v1.66.0 --depth 1 --shallow-submodules https://github.com/grpc/grpc.git grpc-repo \
    && cd grpc-repo \
    && mkdir build && cd build \
    && cmake -DgRPC_INSTALL=ON \
             -DgRPC_BUILD_TESTS=OFF \
             -DCMAKE_BUILD_TYPE=Release \
             -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
             .. \
    && make -j$(nproc) \
    && make install 

WORKDIR /workspace
