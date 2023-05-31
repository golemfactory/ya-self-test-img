FROM rust:1.69.0 as builder
WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install --path . --target x86_64-unknown-linux-musl

# use scratch base image
FROM alpine
COPY --from=builder /usr/local/cargo/bin/ya-self-test /usr/local/bin/ya-self-test
