FROM rust:1.78.0 as rust_builder
RUN rustup target add x86_64-unknown-linux-gnu

FROM rust_builder as builder
WORKDIR /data
COPY . .
RUN cargo install --path . --target x86_64-unknown-linux-gnu

FROM debian:stable-slim AS debian
VOLUME [ "/mnt/out" ]

WORKDIR /
COPY --from=builder /usr/local/cargo/bin/ya-self-test-gpu /ya-self-test

ENTRYPOINT [ "/ya-self-test" ]

