FROM rust:1.88 AS builder

WORKDIR /app

COPY Cargo.toml .

RUN mkdir src \
    && echo "fn main() {}" > src/main.rs

RUN cargo build --release

COPY src ./src

RUN touch src/main.rs \
    && cargo build --release


FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd \
    --system \
    --uid 10001 \
    --create-home \
    app

COPY --from=builder \
    /app/target/release/auth-backend \
    /usr/local/bin/auth-backend

USER app

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/auth-backend"]