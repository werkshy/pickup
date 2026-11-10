# syntax=docker/dockerfile:1

FROM rust:1-trixie AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
      libasound2-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
ARG PICKUP_VERSION=""
ENV PICKUP_VERSION=${PICKUP_VERSION}
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY askama.toml ./
COPY src ./src
COPY frontend ./frontend
RUN cargo build --release --locked && strip target/release/pickup

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
      libasound2t64 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 1000 pickup \
    && useradd --uid 1000 --gid pickup --no-create-home --shell /usr/sbin/nologin pickup \
    && mkdir -p /music /cache \
    && chown pickup:pickup /cache
COPY --from=builder /app/target/release/pickup /usr/local/bin/pickup
ENV MUSIC_DIR=/music \
    CACHE_DIR=/cache \
    PORT=9090
EXPOSE 9090
USER pickup
ENTRYPOINT ["/usr/local/bin/pickup"]
CMD ["serve"]