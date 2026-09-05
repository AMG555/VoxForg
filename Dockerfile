# Multi-stage Dockerfile for VoxForg Workstation Daemon

FROM rust:1.80-bullseye AS builder

WORKDIR /usr/src/voxforg

# Copy manifests
COPY Cargo.toml ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin voxforg

# Runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/voxforg/target/release/voxforg /usr/local/bin/voxforg

RUN mkdir -p /app/data /app/models

ENV VOXFORG_PORT=8080
ENV VOXFORG_HOST=0.0.0.0
ENV RUST_LOG=info

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["voxforg", "serve"]
CMD ["--port", "8080", "--host", "0.0.0.0", "--data-dir", "/app/data"]
