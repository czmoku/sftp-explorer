FROM rust:1.53.0-slim as builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY . /app
WORKDIR /app
RUN cargo build --release

FROM ubuntu:20.04
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=builder /app/target/release/sftp_http .
COPY ./Rocket.toml .
ENTRYPOINT ["/sftp_http"]