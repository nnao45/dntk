# Setup Building Container
## Install Dependency Module
FROM rust:1.34.1-slim as builder
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=%%RUST-VERSION%%
RUN set -eux; \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        git

## Build Cache Dependency Library
RUN mkdir /dntk
COPY Cargo.toml Cargo.lock /dntk/
WORKDIR /dntk
RUN mkdir -p src/ && \
    touch src/lib.rs
RUN cargo build --release

## Build Base Library
COPY . .
RUN cargo build --release

# Setup Running Container
## Install Dependency Module
FROM debian:9.9-slim
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y \
        bc

## Copy App
COPY --from=builder /dntk/target/release/dntk .

## Run
CMD ["./dntk"]