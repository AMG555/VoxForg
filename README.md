<div align="center">

# VoxForg

**Enterprise-Grade, Self-Hostable, Multi-Engine Text-to-Speech Platform & Visual Pipeline Builder**

[![Status](https://img.shields.io/badge/Status-In_Active_Development-yellow.svg)](docs/architecture.md)
[![License](https://img.shields.io/badge/License-Apache_2.0_OR_MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/Frontend-React_19_%2B_TypeScript-61dafb.svg)](https://react.dev)
[![API](https://img.shields.io/badge/API-OpenAI_TTS_Compatible-green.svg)](docs/api.md)
[![Platform](https://img.shields.io/badge/Platform-Linux_%7C_macOS_%7C_Windows-lightgrey.svg)](docs/architecture.md)

*An open-source, modular speech synthesis workstation. Self-host high-fidelity neural voices, chain complex multi-speaker audio graphs, and run locally or scale horizontally.*

[Features](#key-features) • [Architecture](docs/architecture.md) • [Wireframes](docs/wireframes.md) • [API Spec](docs/api.md) • [Quick Start](#quick-start) • [Hardware Tiers](#hardware-profiler)

</div>

---

> [!NOTE]
> **Project Status: In Active Development (Pre-v1.0)**  
> Core engine architecture, hardware autodetect, DAG pipeline runner, and OpenAI-compatible API are functional. Engine backends, offline neural weights, and desktop packaging are actively progressing.

---

## Overview

**VoxForg** is a unified, hardware-adaptive speech synthesis server and visual workflow builder. Think of it as **"n8n for speech"**: a modular engine that allows developers and creators to design sophisticated audio pipelines—from chunking multi-page documents to assigning emotional prosody and dynamic voice-swapping across different TTS engines—all behind an OpenAI-compatible API.

VoxForg eliminates vendor lock-in by abstracting 10+ local and cloud speech engines behind a unified Rust core with zero-latency streaming.

### Key Capabilities

- **Unified Multi-Engine Orchestration**: Support for ultra-fast local engines (Piper, KittenTTS, Kokoro-82M), heavy neural synthesizers (Qwen3-TTS, ChatTTS, StyleTTS2), and zero-cost cloud relays (Edge-TTS) without code changes.
- **Hardware-Aware Adaptive Dispatch**: Automated hardware probe detecting CPU vector extensions (AVX2, AVX-512, NEON) and GPU accelerators (NVIDIA CUDA, Apple MPS, DirectML) to dynamically allocate optimal engine models.
- **Visual Pipeline Canvas**: Node-based directed acyclic graph (DAG) builder to visually compose multi-speaker scripts, insert pause markers, apply audio filters, and batch render podcast-length narratives.
- **Tri-Tier Storage Architecture**:
  - **Embedded / Desktop**: Zero-dependency SQLite with WAL mode.
  - **Self-Hosted Docker**: High-concurrency PostgreSQL with connection pooling.
  - **Cloud Multi-Tenant**: Supabase integration with Row-Level Security (RLS).
- **Enterprise Security Baseline**:
  - Strict server-side proxying; frontend never handles downstream API credentials.
  - Granular API key permissions (`tts:read`, `tts:write`, `pipeline:admin`).
  - Strict CORS validation, rate-limiting (token bucket), and RFC 7807 problem details.
- **Cross-Platform Delivery**:
  - Standalone single binary (`voxforg-cli`).
  - Official multi-arch Docker containers (`linux/amd64`, `linux/arm64`).
  - Native desktop application built on Tauri v2 (Windows `.msi`, macOS `.dmg`, Linux `.AppImage`).

---

## System Architecture

VoxForg is built as a high-performance modular workspace in Rust:

```
                          ┌────────────────────────┐
                          │   Client Application   │
                          │ (OpenAI SDK / Web / UI)│
                          └───────────┬────────────┘
                                      │ HTTP / WebSocket
                                      ▼
┌────────────────────────────────────────────────────────────────────────┐
│                              VOXFORG DAEMON                            │
│                                                                        │
│  ┌───────────────────────┐             ┌────────────────────────────┐  │
│  │     voxforg-api       │             │      voxforg-hardware      │  │
│  │ (Axum 0.8 / Security) │             │ (CUDA/MPS/AVX Auto-Detect) │  │
│  └───────────┬───────────┘             └─────────────┬──────────────┘  │
│              │                                       │                 │
│              ▼                                       ▼                 │
│  ┌───────────────────────┐             ┌────────────────────────────┐  │
│  │   voxforg-pipeline    │◄────────────┤      Engine Scheduler      │  │
│  │  (DAG Graph Runner)   │             │   (Tiered Auto-Fallback)   │  │
│  └───────────┬───────────┘             └─────────────┬──────────────┘  │
│              │                                       │                 │
│              ▼                                       ▼                 │
│  ┌───────────────────────┐             ┌────────────────────────────┐  │
│  │     voxforg-audio     │             │       voxforg-engine       │  │
│  │ (Normalizer/WAV/MP3)  │◄────────────┤  (Piper/Kokoro/Edge/Qwen)  │  │
│  └───────────┬───────────┘             └────────────────────────────┘  │
│              │                                                         │
│              ▼                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                           voxforg-core                           │  │
│  │       (DataStore Trait: SQLite / PostgreSQL / Supabase)          │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

For complete technical specifications, sequence diagrams, and lifecycle documentation, refer to:
- [Architecture Deep-Dive](docs/architecture.md)
- [UI/UX Wireframes & Component Specs](docs/wireframes.md)
- [REST & WebSocket API Reference](docs/api.md)
- [Architecture Decision Records (ADRs)](docs/decisions/)

---

## Quick Start

### 1. Run via Docker Compose (Recommended)

The fastest way to launch VoxForg with the Web UI and SQLite persistence:

```bash
# Clone the repository
git clone https://github.com/your-org/voxforg.git
cd voxforg

# Start the services
docker compose up -d
```

Open `http://localhost:3000` to access the visual pipeline builder. The API server listens at `http://localhost:8080`.

### 2. Run Single Binary (CLI)

Download the release binary for your platform or build from source:

```bash
# Build binary
cargo build --release --bin voxforg

# Verify hardware environment
./target/release/voxforg hardware probe

# Launch server
./target/release/voxforg serve --port 8080 --data-dir ./data
```

### 3. OpenAI Drop-In Replacement

VoxForg provides a drop-in replacement for the OpenAI Audio Speech API (`/v1/audio/speech`).

Using `curl`:

```bash
curl http://localhost:8080/v1/audio/speech \
  -H "Authorization: Bearer dev-key-123" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "edge-tts",
    "input": "VoxForg is now operational with hardware-accelerated speech synthesis.",
    "voice": "en-US-ChristopherNeural",
    "response_format": "wav",
    "speed": 1.0
  }' \
  --output test.wav
```

Using the official OpenAI Python SDK:

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="dev-key-123"
)

response = client.audio.speech.create(
    model="edge-tts",
    voice="en-US-AriaNeural",
    input="Streaming low-latency audio via VoxForg engine."
)

response.stream_to_file("speech.wav")
```

---

## Hardware Profiler

VoxForg inspects your host hardware on startup to tier model selection automatically:

| Tier | Minimum Specifications | Recommended Engines | Latency (TTFB) |
|---|---|---|---|
| **Tier 1: Minimal** | 2 CPU Cores, 2GB RAM | `edge-tts` (Cloud Relay), `espeak-ng` | ~120ms |
| **Tier 2: Standard** | 4 CPU Cores (AVX2), 4GB RAM | `piper` (medium), `kokoro-82m` (ONNX CPU) | ~85ms |
| **Tier 3: Pro** | 8 CPU Cores, 16GB RAM / 4GB VRAM | `kokoro-82m` (GPU), `kittentts`, `piper` (high) | ~35ms |
| **Tier 4: Enterprise** | NVIDIA GPU >= 12GB VRAM (CUDA) | `qwen3-tts`, `chat-tts`, `styletts2` | ~45ms |

To inspect system capability:

```bash
voxforg hardware probe
```

Output:
```
[INFO] Hardware Analysis:
  - CPU: AMD Ryzen 9 5900X (24 threads) [AVX2: YES, AVX-512: NO]
  - Memory: 32768 MB total (24120 MB free)
  - Accelerators Detected:
    * NVIDIA GeForce RTX 3080 (10240 MB VRAM) [CUDA 12.4]
  - Assigned Profile: TIER_4_ENTERPRISE
  - Default Local Engine: kokoro-82m-cuda
```

---

## Visual Pipeline Nodes

The visual workflow editor (`ui/`) supports real-time DAG composition:

- **Text Input / Ingestion**: Raw text, Markdown parser, SSML injector, Document chunker.
- **Voice Allocation**: Regex-based speaker detection (`Alice: ...`, `Bob: ...`), dynamic voice assigner.
- **Audio Processing**: Pitch shift, tempo scale, stereo panning, dynamic range compressor.
- **Assembly**: Concatenation with configurable crossfades, background music ducking, stem export.

---

## Project Structure

```
voxforg/
├── Cargo.toml                  # Virtual workspace manifest
├── README.md                   # Project overview & documentation index
├── docker-compose.yml          # Container configuration
├── docs/
│   ├── architecture.md         # Full architecture diagrams & specifications
│   ├── wireframes.md           # Visual UI specifications & layout blueprints
│   ├── api.md                  # REST & WebSocket API documentation
│   └── decisions/              # Architecture Decision Records (ADR-001 - 005)
├── crates/
│   ├── voxforg-core/           # Domain models, DataStore trait (SQLite/PostgreSQL)
│   ├── voxforg-hardware/       # CPU/GPU hardware detection & tier assignment
│   ├── voxforg-engine/         # TTS Engine trait, registry & implementations
│   ├── voxforg-audio/          # Audio manipulation, transcoding, normalization
│   ├── voxforg-pipeline/       # DAG execution runner & graph solver
│   ├── voxforg-api/            # Axum 0.8 REST server, security & streaming
│   └── voxforg-cli/            # Command line binary
└── ui/                         # React 19 + TypeScript + Tailwind UI
```

---

## Development & Testing

### Prerequisites

- Rust 1.80+ (`cargo`, `rustc`)
- Node.js 20+ & pnpm (for Web UI development)
- SQLite3 / PostgreSQL (optional for local database verification)

### Running Rust Workspace Tests

```bash
# Run all unit and integration tests across crates
cargo test --workspace

# Run security linting
cargo clippy --workspace -- -D warnings
```

---

## Security Policy

VoxForg treats audio generation security with enterprise rigor:
- **No Client Secrets**: API keys, storage bucket credentials, and master configurations are strictly server-side.
- **Input Sanitization**: Text and SSML input is validated against injection attacks and buffer overrun limits.
- **Path Traversal Protection**: Export directories, custom audio models, and voice assets are strictly scoped to designated storage roots.
- **Rate Limiting**: Configurable token bucket rate limiting prevents denial-of-service on local inference engines.

To report security vulnerabilities, please contact `security@voxforg.org`.

---

## License

This project is dual-licensed under:
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

You may choose either license at your option.
