FROM ubuntu:26.04

# Установка компиляторов, инструментов сборки и системных библиотек
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    ninja-build \
    pkg-config \
    gdb \
    gcc-16 \
    g++-16 \
    # Пакеты Protobuf и gRPC
    protobuf-compiler \
    libprotobuf-dev \
    libgrpc++-dev \
    protobuf-compiler-grpc \
    && rm -rf /var/lib/apt/lists/*

# Выставляем GCC 16 компилятором по умолчанию
RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-16 16 \
    --slave /usr/bin/g++ g++ /usr/bin/g++-16

WORKDIR /app
COPY . .
