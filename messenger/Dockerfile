FROM rust:1.97-slim

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    git \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

WORKDIR /workspace/messenger

# CMD ["sleep", "infinity"]
