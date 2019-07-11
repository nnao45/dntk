# Setup Building Container
FROM ekidd/rust-musl-builder:nightly-2019-04-25 as builder
ENV LINUX_TERM_LIB linux_musl.rs

## Build Cache Dependency Library
RUN mkdir /tmp/dntk
RUN sudo chown -R rust:rust .
WORKDIR /tmp/dntk
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/ && \
    touch src/lib.rs
RUN cargo build --release
## Build Base Library
COPY ./src/ ./src/
RUN sudo chown -R rust:rust .
RUN cargo build --release

# Setup Running Container
FROM alpine:3.9
LABEL maintainer "nnao45 <n4sekai5y@gmail.com>"

## Create The Using App User
RUN adduser --uid 1000 -D nnao45
## Install Dependency Module
RUN apk add --update --no-cache bc
## Copy The App
COPY --from=builder /tmp/dntk/target/x86_64-unknown-linux-musl/release/dntk /home/nnao45
## Setup The Using App User
USER 1000:1000
## Run
CMD ["/home/nnao45/dntk"]