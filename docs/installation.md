# VoxForg Installation & Deployment Guide

Comprehensive setup and deployment guide for the VoxForg Speech Synthesis Workstation, DAG Pipeline Engine, and Turnkey Runtime.

---

## 1. Turnkey Quick Start (Zero-Setup)

For the vast majority of users, developers, and creators, VoxForg can be launched or installed immediately with a single command without configuring compilers, Node.js, or external database services.

### 1.1 Pre-Compiled Standalone Releases (Direct Download)

Download zero-install executables directly from GitHub Releases:

| Platform | Asset Name | Format | Instructions |
| :--- | :--- | :--- | :--- |
| **Windows 10/11** | [`VoxForg-Setup-v0.1.0-windows-x64.exe`](https://github.com/AMG555/VoxForg/releases/latest/download/VoxForg-Setup-v0.1.0-windows-x64.exe) | Setup Wizard | Double click setup wizard to install with desktop icons |
| **Windows 10/11** | [`VoxForg-Windows-x64.zip`](https://github.com/AMG555/VoxForg/releases/latest/download/VoxForg-Windows-x64.zip) | Portable Bundle | Extract zip, double-click `VoxForg-Start.bat` |
| **Windows 10/11** | [`voxforg-windows-x86_64.exe`](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-windows-x86_64.exe) | Standalone .exe | Zero-install standalone binary |
| **macOS** (Universal) | [`voxforg-macos-universal`](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-macos-universal) | Mach-O FAT Binary | `chmod +x voxforg-macos-universal && ./voxforg-macos-universal` |
| **Linux** (x86_64) | [`voxforg-linux-x86_64.tar.gz`](https://github.com/AMG555/VoxForg/releases/latest/download/voxforg-linux-x86_64.tar.gz) | Tarball | `tar -xzf voxforg-linux-x86_64.tar.gz && ./voxforg-linux-x86_64` |

---

### 1.2 Instant Run via NPX (No Installation Required)

Launch the unified workstation and local engine server immediately:

```bash
npx voxforg
```

What this does:
1. Automatically queries system OS (`linux`, `darwin`, `win32`) and CPU architecture (`x64`, `arm64`).
2. Downloads the native pre-compiled binary with embedded UI assets into `~/.voxforg/bin/`.
3. Clears macOS Gatekeeper quarantine flags automatically.
4. Starts the local daemon on `http://localhost:8080`.
5. Automatically opens your default web browser to the visual workstation.

---

### 1.2 One-Line Terminal Installers

Install the standalone `voxforg` executable permanently into your system `PATH`:

#### Linux & macOS (POSIX Shell)
```bash
curl -fsSL https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.sh | sh
```

#### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.ps1 | iex
```

Once installed, start the visual workstation anytime with:
```bash
voxforg serve --open
```

---

### 1.3 Single Docker Container (All-in-One)

Launch the self-contained workstation (daemon + embedded visual UI) in a single lightweight container:

```bash
docker run -d \
  -p 8080:8080 \
  -v voxforg_data:/app/data \
  --name voxforg \
  ghcr.io/amg555/voxforg:latest
```

Open `http://localhost:8080` in your web browser.

---

## 2. Building From Source

If you are developing custom engines, modifying the core DAG scheduler, or building in air-gapped environments, you can compile from source.

### 2.1 Prerequisites

| Component | Minimum Version | Recommended | Notes |
| :--- | :--- | :--- | :--- |
| **Rust Toolchain** | `1.80.0+` | Latest Stable | `cargo`, `rustc`, `rustup` |
| **Node.js** | `v20.0.0+` | `v22 LTS` | Only required if modifying the React UI frontend |
| **Git** | `2.30+` | Latest | For cloning repository |
| **C/C++ Compiler** | GCC 10+ / Clang 12+ / MSVC | Host default | Required for native audio decoding and SIMD primitives |

---

### 2.2 Platform-Specific Dependencies

#### Linux (Ubuntu / Debian / Linux Mint)
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    ffmpeg \
    curl \
    git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

#### macOS (Apple Silicon & Intel)
```bash
# Install Homebrew dependencies
brew install openssl pkg-config ffmpeg

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

#### Windows 10 / 11 (PowerShell)
1. Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++").
2. Install [Rust via rustup-init.exe](https://rustup.rs/) (`x86_64-pc-windows-msvc`).
3. Optional: Install FFmpeg via Scoop (`scoop install ffmpeg`) or Winget (`winget install Gyan.FFmpeg`).

---

### 2.3 Compilation Steps

#### Step 1: Clone Repository
```bash
git clone https://github.com/AMG555/VoxForg.git
cd VoxForg
```

#### Step 2: (Optional) Recompile Webstation UI
The repository includes pre-built production assets in `crates/voxforg-api/ui_dist/`. If you modified the React application in `ui/`:
```bash
cd ui
npm install
npm run build
cd ..

# Copy updated distribution to API crate for binary embedding
cp -r ui/dist/* crates/voxforg-api/ui_dist/
```

#### Step 3: Build Rust Workspace
```bash
# Build optimized release binary
cargo build --release --bin voxforg

# Binary location:
# Linux/macOS: ./target/release/voxforg
# Windows:     .\target\release\voxforg.exe
```

#### Step 4: Verify Workspace Tests
```bash
cargo test --workspace
```
*Expected: 106+ tests passed across 14 crates, 0 failed.*

---

## 3. Hardware Acceleration Setup

VoxForg auto-probes compute hardware on initialization. You can verify available tiers using:

```bash
./target/release/voxforg hardware
```

### 3.1 NVIDIA CUDA Setup (Linux / Windows)
- Install NVIDIA Driver 550+ and CUDA Toolkit 12.0+.
- Verify driver with `nvidia-smi`.
- VoxForg's `HardwareProbe` detects CUDA VRAM and enables local heavy neural models.

### 3.2 Apple Silicon (Metal / MPS)
- macOS Apple Silicon systems (M1 through M4) automatically leverage unified memory and Accelerate framework vector accelerators.

### 3.3 AVX2 / AVX-512 CPU SIMD
- On x86_64 CPUs, VoxForg detects AVX2 and AVX-512 instruction sets for real-time DSP biquad filtering and gain normalization.

---

## 4. Multi-Service Container Orchestration

### 4.1 Turnkey Neural Stack (`docker-compose.neural.yml`)
To launch VoxForg with dedicated offline neural containers:
- **VoxForg Daemon**: HTTP API, WebSocket streaming, and embedded Webstation UI on port `8080`.
- **Faster-Whisper Container**: Local OpenAI-compatible ASR engine on port `8000`.
- **Kokoro-FastAPI Container**: Ultra-fast local neural TTS engine on port `8880`.

```bash
docker compose -f docker-compose.neural.yml up -d
```

### 4.2 Standard Database Stack (`docker-compose.yml`)
To launch VoxForg with dedicated PostgreSQL storage:

```bash
docker compose up -d
```

---

## 5. Verification Checklist

1. **Check Hardware Profiler**:
   ```bash
   voxforg hardware
   ```
2. **List Registered Engines & Voices**:
   ```bash
   voxforg engines
   ```
3. **List Curated Neural Models**:
   ```bash
   voxforg models list
   ```
4. **Synthesize Test Audio**:
   ```bash
   voxforg synth "VoxForg workstation operational." --voice en-US-AriaNeural --out test.wav
   ```
5. **Launch Server with Browser Auto-Launch**:
   ```bash
   voxforg serve --open
   ```
6. **Verify Endpoints**:
   - Webstation Workstation: `http://localhost:8080/`
   - Interactive OpenAPI Docs: `http://localhost:8080/docs`
   - Health Check: `http://localhost:8080/health`
   - Prometheus Telemetry: `http://localhost:8080/metrics`
