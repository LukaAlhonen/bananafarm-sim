FROM rust:1.91-alpine AS builder

ARG TARGETARCH
WORKDIR /usr/src/app

# RUN apt update && apt install -y pkg-config build-essential musl-tools openssl libssl-dev
RUN apk add openssl-dev musl-dev openssl-libs-static pkgconfig

COPY Cargo.toml Cargo.lock ./

# need to copy all workspaces so cargo does not freak out
COPY integration ./integration
COPY influxdb3client ./influxdb3client
COPY subscriber ./subscriber
COPY publisher ./publisher

RUN rustup target add aarch64-unknown-linux-musl
RUN rustup target add x86_64-unknown-linux-musl

RUN case "$TARGETARCH" in \
    "arm64")  cargo build -p subscriber --release --target aarch64-unknown-linux-musl ;; \
    "amd64")  cargo build -p subscriber --release --target x86_64-unknown-linux-musl ;; \
    esac

RUN case "$TARGETARCH" in \
    "arm64")  cp /usr/src/app/target/aarch64-unknown-linux-musl/release/subscriber /usr/src/app/target/release/subscriber ;; \
    "amd64")  cp /usr/src/app/target/x86_64-unknown-linux-musl/release/subscriber /usr/src/app/target/release/subscriber ;; \
    esac

FROM alpine:latest

COPY --from=builder /usr/src/app/target/release/subscriber /subscriber

ENTRYPOINT [ "/subscriber" ]
