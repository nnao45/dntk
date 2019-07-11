# Setup Building Container
FROM ekidd/rust-musl-builder:nightly-2019-04-25 as builder
ENV LINUX_TERM_LIB linux_musl.rs

## Build Cache Dependency Library
RUN mkdir /app
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/root/.cargo \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release -Z unstable-options --out-dir /output

# Setup Running Container
FROM alpine:3.9
LABEL maintainer "nnao45 <n4sekai5y@gmail.com>"

## Create The Using App User
RUN adduser --uid 1000 -D nnao45
## Install Dependency Module
RUN apk add --update --no-cache bc
## Copy The App
COPY --from=builder /output/dntk /home/nnao45
## Setup The Using App User
USER 1000:1000
## Run
CMD ["/home/nnao45/dntk"]