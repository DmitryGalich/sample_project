FROM ubuntu:26.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    ninja-build \
    gdb \
    gcc-16 \
    g++-16 \
    git \
    curl \
    zip \
    unzip \
    tar \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-16 16 \
    --slave /usr/bin/g++ g++ /usr/bin/g++-16

WORKDIR /app
COPY . .