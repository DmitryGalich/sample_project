FROM rust:1.88

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

RUN cargo install sqlx-cli --version 0.7.4 --no-default-features --features native-tls,postgres

WORKDIR /workspace/users