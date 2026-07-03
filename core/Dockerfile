FROM ubuntu:24.04

# Установка зависимостей сборки
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    ninja-build \
    gdb \
    gcc-14 \
    g++-14 \
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

# Выставляем GCC 14 как дефолтный компилятор системы
RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-14 14 \
    --slave /usr/bin/g++ g++ /usr/bin/g++-14

# Создаем venv для Conan, чтобы избежать конфликтов с системным Python
RUN python3 -m venv /opt/conan-env
ENV PATH="/opt/conan-env/bin:$PATH"

# Обновляем pip и ставим Conan 2.x
RUN pip install --no-cache-dir -U pip setuptools && \
    pip install --no-cache-dir conan

# Создаем профиль Conan
RUN conan profile detect --force

WORKDIR /app
COPY . .

# ВАЖНО: Явно передаем дефолтный профиль и как host, и как build (-pr:b=default -pr:h=default)
# Это решает проблему сборки кросс-зависимостей вроде protoc в gRPC.
RUN conan install . --output-folder=build --build=missing \
    -pr:b=default -pr:h=default \
    -s build_type=Release
