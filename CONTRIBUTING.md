# Contributing to VoxForg

Thank you for your interest in contributing to VoxForg! Whether you are fixing a bug, adding new neural TTS engines, improving the audio workstation UI, or writing documentation, all contributions are welcome.

---

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all project interactions.

---

## Development Setup

### Prerequisites

- **Rust**: 1.80+ (`rustup default stable`)
- **Node.js**: 20+ & npm (for UI development)
- **Git**
- *(Windows optional)*: Inno Setup 6 for building the Windows installer

### Build and Run

1. **Clone the repository**:
   ```bash
   git clone https://github.com/AMG555/VoxForg.git
   cd VoxForg
   ```

2. **Run tests**:
   ```bash
   cargo test --workspace
   ```

3. **Check formatting and lints**:
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   ```

4. **Run the Studio locally**:
   ```bash
   cargo run -p voxforg-cli -- serve --open
   ```

---

## How to Contribute

### 1. Reporting Bugs
Check existing [Issues](https://github.com/AMG555/VoxForg/issues) before opening a new one. Provide:
- OS and architecture (e.g. Windows 11 x64, Ubuntu 24.04)
- Exact steps to reproduce
- Expected vs actual behavior
- Relevant log output

### 2. Suggesting Features & Engine Adapters
We welcome ideas for new engines, audio processing nodes, or UI workflows!
- Open a feature request on [Discussions](https://github.com/AMG555/VoxForg/discussions) or [Issues](https://github.com/AMG555/VoxForg/issues).

### 3. Pull Requests
- Branch from `main` with descriptive name (`fix/audio-buffer-overflow`, `docs/add-engine-guide`).
- Keep PRs focused on a single change.
- Ensure `cargo test --workspace` and `cargo fmt --all --check` pass.
- Submit PR against `main`.

---

## Architecture Overview

- `crates/voxforg-core`: Shared audio buffers, pipeline primitives, config types.
- `crates/voxforg-engine`: Neural TTS engine adapters (Piper, XTTS, Kokoro, Whisper).
- `crates/voxforg-server`: High-performance Axum HTTP/WebSocket streaming server.
- `crates/voxforg-cli`: Turnkey CLI and background server launcher.
- `ui/`: Modern neural audio workstation frontend built with TypeScript & Vanilla CSS.
