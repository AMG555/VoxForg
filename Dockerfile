# Multi-stage Dockerfile for VoxForg Self-Contained Workstation
# Single container serves REST API, WebSocket streams, and embedded React UI.

# 1. UI Build Stage
FROM node:20-bullseye-slim AS ui-builder
WORKDIR /app/ui
COPY ui/package*.json ./
RUN npm ci --prefer-offline --no-audit
COPY ui/ ./
RUN npm run build

# 2. Rust Compile Stage
FROM rust:1.80-bullseye AS rust-builder
WORKDIR /usr/src/voxforg
COPY Cargo.toml ./
COPY crates ./crates

# Copy freshly compiled UI assets into voxforg-api embedded location
COPY --from=ui-builder /app/ui/dist ./crates/voxforg-api/ui_dist

# Build unified standalone binary
RUN cargo build --release --bin voxforg

# 3. Minimal Runtime Stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libasound2 \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy standalone binary
COPY --from=rust-builder /usr/src/voxforg/target/release/voxforg /usr/local/bin/voxforg

RUN mkdir -p /app/data /app/models

ENV VOXFORG_PORT=8080
ENV VOXFORG_HOST=0.0.0.0
ENV RUST_LOG=info

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["voxforg", "serve"]
CMD ["--port", "8080", "--host", "0.0.0.0", "--data-dir", "/app/data"]
