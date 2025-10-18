# Setup Building Container
FROM rust:1.83-alpine as builder
ENV LINUX_TERM_LIB linux_musl.rs

## Install build dependencies
RUN apk add --no-cache musl-dev

## Build Cache Dependency Library
RUN mkdir /tmp/dntk
WORKDIR /tmp/dntk
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/ && \
    touch src/lib.rs
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
## Build Base Library
COPY ./src/ ./src/
COPY ./build.rs ./
RUN cargo build --release --target x86_64-unknown-linux-musl

# Setup Running Container
FROM alpine:3.9
LABEL maintainer "nnao45 <n4sekai5y@gmail.com>"

## Create The Using App User
RUN adduser --uid 1000 -D nnao45
## Copy The App (bc is no longer required!)
COPY --from=builder /tmp/dntk/target/x86_64-unknown-linux-musl/release/dntk /home/nnao45
## Setup The Using App User
USER 1000:1000
## Run
CMD ["/home/nnao45/dntk"]