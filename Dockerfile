FROM ekidd/rust-musl-builder:1.51.0 as builder

COPY Cargo.toml Makefile ./
COPY src/ ./src/

RUN make
RUN strip /home/rust/src/target/x86_64-unknown-linux-musl/release/ping-exporter

FROM alpine:latest

WORKDIR /
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/ping-exporter ./

ENTRYPOINT ["/ping-exporter"]
