FROM rust:1.83 AS builder

WORKDIR /app

COPY src/ /app/src
COPY ["Cargo.lock", "Cargo.toml", "/app/"]

RUN cargo build --release

FROM scratch

COPY --from=builder /app/target/release/mqtt-to-dawarich /

CMD [ "./mqtt-to-dawarich" ]
