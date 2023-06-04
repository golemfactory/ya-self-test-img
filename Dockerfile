FROM rust:1.69.0 as builder
WORKDIR /data
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install --path . --target x86_64-unknown-linux-musl

FROM scratch
WORKDIR /data
COPY --from=builder /usr/local/cargo/bin/ya-self-test /ya-self-test
CMD [ "/ya-self-test" ]
