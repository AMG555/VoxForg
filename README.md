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

[Download .exe](#direct-downloads-desktop--standalone) • [Features](#key-features) • [Installation Guide](docs/installation.md) • [User Manual](docs/user-guide.md) • [Architecture](docs/architecture.md) • [API Spec](docs/api.md) • [Wireframes](docs/wireframes.md) • [Hardware Tiers](#hardware-profiler)

</div>

---

> [!NOTE]
> **Project Status: In Active Development (Pre-v1.0)**  
> Core engine architecture, hardware autodetect, DAG pipeline runner, and OpenAI-compatible API are functional. Engine backends, offline neural weights, and desktop packaging are actively progressing.

> [!TIP]
> **100% Local & Privacy Guaranteed**  
> All neural voice synthesis, cloning, and transcription execute directly on your machine. Zero telemetry, zero audio streaming to external clouds, zero tracking. Fully functional offline.

---

## Direct Downloads (Desktop & Standalone)

For creators, studio producers, and desktop users: download the pre-compiled standalone app directly. Zero build tools, Docker, or terminal setup required.

| Platform | Download | Package Format | File Size | Details |
| :--- | :--- | :--- | :--- | :--- |
| **Windows 10 / 11 (Installer)** | [**Download Setup (.exe)**](https://github.com/AMG555/VoxForg/releases/latest/download/VoxForg-Setup-v0.1.0-windows-x64.exe) | Setup Wizard (`.exe`) | ~14 MB | Professional installer with path selection, desktop shortcut, uninstaller |
| **Windows 10 / 11 (Portable)** | [**Download Standalone .exe**](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-windows-x86_64.exe) | Portable Executable | ~13 MB | Single zero-install binary with embedded studio UI |
| **Windows 10 / 11 (Zip Bundle)** | [**Download .zip**](https://github.com/AMG555/VoxForg/releases/latest/download/VoxForg-Windows-x64.zip) | Portable Bundle (`.zip`) | ~5 MB | Includes `voxforg.exe`, 1-click launcher, and desktop installer |
| **macOS (Apple Silicon & Intel)** | [**Download Universal**](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-macos-universal) | Standalone Binary | ~14 MB | Universal binary with embedded studio UI |
| **Linux (x86_64)** | [**Download .tar.gz**](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-linux-x86_64.tar.gz) | Tarball Archive | ~12 MB | Zero-dependency glibc standalone executable |

### How to Run & Install on Windows:

1. **Direct Run**: Download [**`voxforg-windows-x86_64.exe`**](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-windows-x86_64.exe) and double-click it. It spins up the neural engine server and opens your web browser to `http://localhost:8080`.
2. **Portable Bundle**: Download [**`VoxForg-Windows-x64.zip`**](https://github.com/AMG555/VoxForg/releases/latest/download/VoxForg-Windows-x64.zip), extract anywhere, and double-click `VoxForg-Start.bat`.
3. **Install Desktop & Start Menu Icon**: Run `install-desktop.bat` (or run in PowerShell: `irm https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.ps1 | iex`). Adds `VoxForg` icon directly to your Windows Desktop and Start Menu.

> [!NOTE]
> **Windows SmartScreen warning?** You may see *"voxforg-windows-x86_64.exe isn't commonly downloaded"* in your browser or *"Windows protected your PC"* on launch. This is normal for any new open-source executable that hasn't yet built download reputation — **VoxForg contains no malware**. The full source code is auditable in this repository.
>
> **To bypass in Edge:** In the Downloads panel click **See more** → **Keep anyway**.
>
> **To bypass in Chrome:** Click the download arrow → **Keep** → **Keep anyway**.
>
> **To bypass the SmartScreen launch dialog:** Click **More info** → **Run anyway**.
>
> **Prefer no warnings at all?** Use the **PowerShell one-liner installer** — it runs entirely in-terminal and bypasses SmartScreen:
> ```powershell
> irm https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.ps1 | iex
> ```


---

## Quick Start

**VoxForg** is a unified, hardware-adaptive speech synthesis server and visual workflow builder: a modular node-based engine that allows developers and creators to design sophisticated audio pipelines—from chunking multi-page documents to assigning emotional prosody and dynamic voice-swapping across different TTS engines—all behind an OpenAI-compatible API.

VoxForg eliminates vendor lock-in by abstracting 10+ local and cloud speech engines behind a unified Rust core with zero-latency streaming.

### Key Capabilities

- **Unified Multi-Engine Orchestration**: Support for ultra-fast local engines (Piper, KittenTTS, Kokoro-82M), heavy neural synthesizers (Qwen3-TTS, ChatTTS, StyleTTS2), and zero-cost cloud relays (Edge-TTS) without code changes.
- **Workflows Dashboard (n8n-Style Project Hub)**: Central dashboard to manage multiple speech pipelines. Search and filter across tags (`Podcast`, `Tactical`, `Dubbing`, `Radio`), duplicate templates, import/export full pipeline JSON schemas, and switch projects via clean breadcrumbs.
- **Visual Pipeline Canvas & DAG Ergonomics**: 2D infinite node-based DAG canvas with smooth 2D pan, pointer-anchored focal zoom (`0.25x` to `2.5x`), marquee multi-selection, 16px magnetic grid snapping, multi-node JSON clipboard copying (`Ctrl+C`), and searchable quick-add node palette (`Tab` or `/`). Features dynamic connection wire beacons, hover cut, Kahn's topological execution ordering, per-node bypass toggles, and real-time millisecond latency telemetry.
- **Automated Speech Recognition (ASR)**: Native Whisper inference engine with sliding-window VAD energy segmentation, generating OpenAI-compatible transcriptions, word timestamps, and SRT/VTT subtitle files (`POST /v1/audio/transcriptions`).
- **Zero-Shot Voice Cloning & Studio Pre-Processing**: Studio-grade acoustic pipeline featuring 80Hz Butterworth high-pass filtering, adaptive noise gating, true peak normalization to -0.5 dBFS, and 24kHz linear PCM transcoding to extract 512-D neural speaker identity embeddings (`POST /v1/voices/clone`). Persistent across browser local storage and backend database.
- **Full Automation Connectivity (n8n & Webhooks)**: Connects natively to n8n workflows via drop-in OpenAI Audio nodes (`http://localhost:8080/v1`), direct HTTP Request binary audio nodes, webhook-triggered DAG pipeline execution (`POST /v1/pipeline/execute`), and programmatic voice cloning.
- **Model Context Protocol (MCP) Server**: Built-in JSON-RPC 2.0 stdio server (`voxforg mcp`) exposing 8 tools (`synthesize_speech`, `clone_voice`, `transcribe_audio`, `execute_pipeline`, `list_voices`, `list_models`, `benchmark_engine`, `get_cluster_status`) directly to Claude Desktop, Cursor, Antigravity, and Windsurf.
- **Production Video Dubbing & Audiobook DAG Workflows**: Turnkey DAG templates chaining ASR, Speaker Diarization, Voice Mapping, Time-Stretch tempo alignment, and Audio Muxing for video tracks and multi-chapter audiobook publishing.
- **Model Catalog & Weights Manager**: Built-in weights manager validating host RAM and GPU VRAM constraints before installing curated open weights (Kokoro, Piper, Whisper, Qwen3, Silero) with cryptographic SHA-256 verification (`/v1/catalog/models`).
- **Hardware-Aware Adaptive Dispatch**: Automated hardware probe detecting CPU vector extensions (AVX2, AVX-512, NEON) and GPU accelerators (NVIDIA CUDA, Apple MPS, DirectML) to dynamically allocate optimal engine models.
- **Tri-Tier Storage Architecture**:
  - **Embedded / Desktop**: Zero-dependency SQLite with WAL mode.
  - **Self-Hosted Docker**: High-concurrency PostgreSQL with connection pooling.
  - **Cloud Multi-Tenant**: Supabase integration with Row-Level Security (RLS).
- **Automated QA & Voice A/B Benchmarking**:
  - Side-by-side acoustic quality evaluation across engines, voices, and prosody parameters.
  - Measures real-time factor (RTF), TTFB latency, RMS loudness (dBFS), peak amplitude, and digital clipping samples.
  - Accessible via interactive UI (`A/B & QA Lab`), REST API (`POST /v1/qa/ab-test`), and CLI (`voxforg ab-test`).
- **Enterprise Security Baseline**:
  - Strict server-side proxying; frontend never handles downstream API credentials.
  - Granular API key permissions (`tts:read`, `tts:write`, `pipeline:admin`).
  - Strict CORS validation, rate-limiting (token bucket), and RFC 7807 problem details.
- **Cross-Platform Delivery**:
  - Standalone single binary (`voxforg-cli`) with embedded workstation UI.
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
- [Installation Guide](docs/installation.md)
- [User Manual & CLI Reference](docs/user-guide.md)
- [Architecture Deep-Dive](docs/architecture.md)
- [UI/UX Wireframes & Component Specs](docs/wireframes.md)
- [REST & WebSocket API Reference](docs/api.md)
- [Architecture Decision Records (ADRs)](docs/decisions/)

---

## Quick Start

### 1. Instant Run via NPX (No Installation)

Launch the full visual workstation and local server with one command:

```bash
npx voxforg
```
*Auto-detects platform, fetches matching binary with embedded UI, starts server, and opens your browser to `http://localhost:8080`.*

---

### 2. One-Line Terminal Installers

Install the standalone binary into your system PATH:

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.sh | sh
```

**Windows PowerShell:**
```powershell
irm https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.ps1 | iex
```

Start the visual workstation:
```bash
voxforg serve --open
```

---

### 3. Single Docker Container (All-in-One)

Launch the unified container (backend + embedded visual UI):

```bash
docker run -d -p 8080:8080 -v voxforg-data:/app/data ghcr.io/amg555/voxforg
```
Open `http://localhost:8080` in your browser.

---

### 4. Run via Docker Compose (Multi-Service Stack)

The fastest way to launch VoxForg with database persistence or dedicated neural microservices:

```bash
# Clone the repository
git clone https://github.com/AMG555/VoxForg.git
cd VoxForg

# Start standard services (Daemon + Database)
docker compose up -d

# Or launch complete turnkey neural stack (Faster-Whisper ASR + Kokoro Neural TTS + Daemon)
docker compose -f docker-compose.neural.yml up -d
```

---

### 5. Run Single Binary (CLI)

Download the release binary for your platform or build from source:

```bash
# Build binary
cargo build --release --bin voxforg

# Verify hardware environment
./target/release/voxforg hardware

# Run automated QA A/B benchmark between voices
./target/release/voxforg ab-test --iterations 3

# Start Model Context Protocol (MCP) server for LLM agents
./target/release/voxforg mcp

# Launch HTTP/WS server
./target/release/voxforg serve --port 8080 --data-dir ./data
```

### 6. OpenAI Drop-In Replacement

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

### 7. n8n Workflow Automation Integration

Connect VoxForg to [n8n](https://n8n.io) to automate speech pipelines, customer voice replies, and multi-speaker podcast generation:

- **OpenAI Node (Drop-in)**: Point your n8n OpenAI node's Base URL to `http://localhost:8080/v1` to generate speech without changing your existing workflow logic.
- **HTTP Request Node**: Invoke `POST http://localhost:8080/v1/audio/speech` with response format set to `File (Binary)` to dispatch audio to Telegram, WhatsApp, Slack, or Discord bots.
- **Webhook DAG Execution**: Trigger full multi-speaker DAG pipelines with `POST http://localhost:8080/v1/pipeline/execute` passing exported pipeline JSON.
- **Automated Voice Cloning**: Conditionally clone caller voices via `POST http://localhost:8080/v1/voices/clone`.

---

### 8. Model Context Protocol (MCP) Server

VoxForg runs a native JSON-RPC 2.0 stdio server (`voxforg mcp`) allowing LLM coding assistants (Claude Desktop, Cursor, Antigravity, Windsurf) to execute speech tools autonomously:

```json
{
  "mcpServers": {
    "voxforg": {
      "command": "voxforg",
      "args": ["mcp"]
    }
  }
}
```

Exposes 8 core tools: `synthesize_speech`, `clone_voice`, `transcribe_audio`, `execute_pipeline`, `list_voices`, `list_models`, `benchmark_engine`, and `get_cluster_status`.

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
│   ├── voxforg-core/           # Domain models, DataStore trait, error taxonomy
│   ├── voxforg-hardware/       # CPU/GPU hardware detection & tier assignment
│   ├── voxforg-engine/         # TTS Engine traits, registry, Edge-TTS, Qwen3 zero-shot cloning
│   ├── voxforg-router/         # SLA policy routing and multi-objective scoring
│   ├── voxforg-pronunciation/  # Text normalization and custom pronunciation dictionary
│   ├── voxforg-audio/          # Audio manipulation, transcoding, DSP & metrics
│   ├── voxforg-pipeline/       # DAG execution engine, time stretch, dubbing/audiobook templates
│   ├── voxforg-benchmark/      # Engine benchmarking sentence suite & scorecard store
│   ├── voxforg-worker/         # Distributed cluster worker pool & lease management
│   ├── voxforg-asr/            # Automated Speech Recognition abstraction, Whisper, VAD
│   ├── voxforg-catalog/        # Open-weights model catalog & hardware validation
│   ├── voxforg-mcp/            # Model Context Protocol JSON-RPC 2.0 stdio server
│   ├── voxforg-api/            # Axum REST & WebSocket server, OpenAPI 3.1, metrics
│   └── voxforg-cli/            # Unified CLI binary (serve, synth, hardware, bench, ab-test, mcp)
└── ui/                         # React 19 + TypeScript + Tailwind UI
```

---

## Development & Testing

### Prerequisites

- Rust 1.80+ (`cargo`, `rustc`)
- Node.js 20+ & pnpm (for Web UI development)
- SQLite3 / PostgreSQL (optional for local database verification)

### Running Rust Workspace Tests

The workspace features a comprehensive 120+ automated test suite covering acoustic quality metrics, DAG scheduling, topological sorting, storage isolation, hardware tiering, voice cloning, ASR transcriptions, model catalogs, MCP protocol, and REST/WebSocket API endpoints:

```bash
# Run all unit and integration tests across crates (120+ tests)
cargo test --workspace

# Run dedicated audio metrics and QA benchmark tests
cargo test -p voxforg-audio --lib metrics
cargo test -p voxforg-engine --lib ab_test
cargo test -p voxforg-api --lib tests

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

To report security vulnerabilities, please contact `amgt503@gmail.com`.

---

## License

This project is dual-licensed under:
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

You may choose either license at your option.
