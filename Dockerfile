# POC Blue Glory agent (#68) — daily Sheets → CheckIn submit-guests
FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY src ./src
COPY assets ./assets
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/guest-checkin /usr/local/bin/guest-checkin
# Optional header image for SES HTML (same template as UNL mode)
COPY src/header_image.jpg /app/header_image.jpg
ENTRYPOINT ["guest-checkin"]
