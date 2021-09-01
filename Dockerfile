FROM rust:1.53.0-slim as builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /app
COPY backend .
RUN cargo build --release

FROM node:16-alpine as vue-builder
WORKDIR app
COPY frontend/package.json .
RUN yarn install
COPY frontend .
RUN yarn build

FROM ubuntu:20.04
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=builder /app/target/release/sftp_explorer .
COPY --from=vue-builder /app/dist ./static
COPY backend/Rocket.toml .
ENV API_PREFIX=/api
ENV STATIC_PREFIX=/ui
ENV BE_PREFIX=/api
ENTRYPOINT ["/sftp_explorer"]