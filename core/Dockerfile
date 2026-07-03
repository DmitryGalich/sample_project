FROM ubuntu:26.04

# Установка системных зависимостей, Python и Pip
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
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-16 16 \
    --slave /usr/bin/g++ g++ /usr/bin/g++-16

RUN pip install --break-system-packages conan

RUN conan profile detect --force && \
    echo "compiler.cppstd=gnu26" >> ~/.conan2/profiles/default

WORKDIR /app
COPY . .

RUN conan install . --output-folder=build --build=missing
