FROM rust:1.83 AS builder

WORKDIR /app

COPY src/ /app/src
COPY ["Cargo.lock", "Cargo.toml", "/app/"]

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:latest

COPY --from=builder /app/target/release/mqtt-to-dawarich /

CMD [ "/mqtt-to-dawarich" ]
