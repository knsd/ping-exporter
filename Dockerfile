# Original: https://github.com/knsd/ping-exporter

# AMD64 Build
FROM --platform=linux/amd64 ekidd/rust-musl-builder:1.36.0 as builder
COPY Cargo.toml Makefile ./
COPY src/ ./src/
RUN make
RUN strip /home/rust/src/target/x86_64-unknown-linux-musl/release/ping-exporter
FROM alpine:latest
WORKDIR /
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/ping-exporter ./
ENTRYPOINT ["/ping-exporter"]

# From: https://github.com/rust-cross/rust-musl-cross

# ARM32v7 build
FROM --platform=linux/arm32v7 messense/rust-musl-cross:armv7-musleabihf as builder
COPY Cargo.toml Makefile ./
COPY src/ ./src/
RUN make
RUN musl-strip /home/rust/src/target/armv7-unknown-linux-musleabihf/release/ping-exporter
FROM alpine:latest
WORKDIR /
COPY --from=builder /home/rust/src/target/armv7-unknown-linux-musleabihf/release/ping-exporter ./
ENTRYPOINT ["/ping-exporter"]

# ARM64 build
FROM --platform=linux/arm64 messense/rust-musl-cross:aarch64-musl as builder
COPY Cargo.toml Makefile ./
COPY src/ ./src/
RUN make
RUN musl-strip /home/rust/src/target/aarch64-unknown-linux-musl/release/ping-exporter
FROM alpine:latest
WORKDIR /
COPY --from=builder /home/rust/src/target/aarch64-unknown-linux-musl/release/ping-exporter ./
ENTRYPOINT ["/ping-exporter"]