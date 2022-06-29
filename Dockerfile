FROM rust:1.61 AS builder
WORKDIR /usr/src/auth-uservice

# Install musl dependencies
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

# Build binary
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

# Deployment
FROM scratch
WORKDIR /usr/src/auth-uservice

COPY --from=builder /usr/src/auth-uservice/target/x86_64-unknown-linux-musl/release/auth-uservice .

EXPOSE 8000/tcp

ENTRYPOINT ["./auth-uservice"]