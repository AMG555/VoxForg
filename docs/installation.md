# VoxForg Installation Guide

Comprehensive setup and deployment guide for the VoxForg Speech Synthesis Workstation and Engine Platform.

---

## 1. Prerequisites

Before installing VoxForg, ensure the following core tools are available on your system:

| Component | Minimum Version | Recommended | Notes |
| :--- | :--- | :--- | :--- |
| **Rust Toolchain** | `1.80.0+` | Latest Stable | `cargo`, `rustc`, `rustup` |
| **Node.js** | `v20.0.0+` | `v22 LTS` | Required for Webstation UI |
| **Git** | `2.30+` | Latest | For cloning repository |
| **C/C++ Compiler** | GCC 10+ / Clang 12+ / MSVC | Host default | Required for native audio and SIMD primitives |

---

## 2. Platform-Specific Setup

### 2.1 Linux (Ubuntu / Debian / Linux Mint)

```bash
# 1. Install build essentials and audio development libraries
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    curl \
    git

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 3. Install Node.js (via NodeSource)
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### 2.2 macOS (Apple Silicon M1/M2/M3/M4 & Intel)

```bash
# 1. Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Install core dependencies
brew install openssl pkg-config node

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 2.3 Windows 10 / 11 (PowerShell)

1. **Install Visual Studio C++ Build Tools**:
   Download and install "Desktop development with C++" from the [Visual Studio Installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. **Install Rust**:
   Download and run [rustup-init.exe](https://rustup.rs/). Choose default installation (`x86_64-pc-windows-msvc`).
3. **Install Node.js**:
   Download the Windows MSI installer from [nodejs.org](https://nodejs.org/).
4. Verify installations in PowerShell:
   ```powershell
   rustc --version
   cargo --version
   node --version
   ```

---

## 3. Building From Source

### Step 1: Clone Repository
```bash
git clone https://github.com/AMG555/VoxForg.git
cd VoxForg
```

### Step 2: Build Rust Workspace
```bash
# Build optimized release binaries
cargo build --release --workspace

# The binary will be located at:
# Linux/macOS: ./target/release/voxforg
# Windows:     .\target\release\voxforg.exe
```

### Step 3: Build Webstation Frontend
```bash
cd ui
npm install
npm run build
cd ..
```

### Step 4: Verify Workspace Tests
```bash
cargo test --workspace
```
*Expected: 60 tests passed, 0 failed.*

---

## 4. Hardware Acceleration Setup

VoxForg auto-detects host hardware on boot. You can inspect your host capabilities with:

```bash
cargo run --release --bin voxforg -- hardware
```

### 4.1 NVIDIA CUDA Setup (Linux / Windows)
- Install NVIDIA Driver 550+ and CUDA Toolkit 12.0+.
- Verify driver:
  ```bash
  nvidia-smi
  ```
- VoxForg's `HardwareProbe` will automatically recognize available VRAM and assign Tier 4 (Enterprise).

### 4.2 Apple Silicon (Metal / MPS)
- macOS Apple Silicon devices automatically leverage integrated unified memory and Accelerate framework.

### 4.3 AVX2 / AVX-512 CPU SIMD
- On x86_64 CPUs, VoxForg automatically detects AVX, AVX2, and AVX-512 instruction sets for real-time DSP biquad filtering and normalization.

---

## 5. Docker Deployment

### 5.1 Quick Start with Docker Compose
To run VoxForg in a containerized environment with Postgres:

```bash
docker compose up -d
```

- API Server: `http://localhost:8080`
- Webstation UI: `http://localhost:3000`
- Interactive API Docs: `http://localhost:8080/docs`
- Prometheus Metrics: `http://localhost:8080/metrics`

### 5.2 Standalone Container
```bash
# Build minimal image
docker build -t voxforg:latest .

# Run container
docker run -d \
  -p 8080:8080 \
  -v voxforg_data:/data \
  --name voxforg-server \
  voxforg:latest
```

---

## 6. Verification Checklist

1. **Check Hardware Profiler**:
   ```bash
   voxforg hardware
   ```
2. **List Registered Engines & Voices**:
   ```bash
   voxforg engines
   ```
   *Should list `edge-tts`, `mock-tts`, `openai-router`, and 18 available voices.*
3. **Synthesize Test Audio**:
   ```bash
   voxforg synth "Installation verified successfully." --voice en-US-AriaNeural --out test.wav
   ```
4. **Launch Server**:
   ```bash
   voxforg serve --port 8080
   ```
5. **Check Health & Telemetry**:
   ```bash
   curl http://localhost:8080/health
   curl http://localhost:8080/metrics
   ```
