FROM rust:1.88.0-bookworm AS build-env

WORKDIR /app/src

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update -y && apt-get install -y curl

WORKDIR /app
COPY --from=build-env /app/src/target/release .
COPY --from=build-env /app/src/settings.toml /app/settings.toml

ENTRYPOINT ["/app/boxer-validator-nginx"]