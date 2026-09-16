# VoxForg User Guide & Manual

Complete operational manual for developers, sound engineers, and system administrators.

---

## 1. CLI Reference Manual

The `voxforg` binary provides a unified command line interface for workstation operations, speech synthesis, hardware diagnostics, voice cloning, automated transcription, model management, benchmarking, and automated A/B evaluation.

```bash
voxforg <COMMAND> [OPTIONS]
```

### 1.1 Commands Summary

| Command | Description | Example |
| :--- | :--- | :--- |
| `serve` | Start HTTP REST, WebSocket, and embedded Webstation UI server | `voxforg serve --open` |
| `synth` | Synthesize speech text directly to audio file | `voxforg synth "Hello world" --voice alloy --out out.wav` |
| `transcribe` | Transcribe speech audio/video to text, SRT, VTT, or JSON subtitles | `voxforg transcribe clip.wav --format srt --out clip.srt` |
| `clone` | Extract zero-shot speaker embedding profile from reference audio | `voxforg clone --name "Rachel" --audio sample.wav` |
| `models` | Manage local neural model packages and download weights | `voxforg models list` |
| `hardware` | Inspect host CPU, SIMD, RAM, and GPU compute capabilities | `voxforg hardware` |
| `engines` | List registered speech engines and available voices | `voxforg engines` |
| `bench` | Run synthesis latency benchmark across iterations | `voxforg bench --iterations 10` |
| `ab-test` | Run automated objective A/B acoustic quality comparison | `voxforg ab-test --voice-a en-US-AriaNeural --voice-b alloy` |
| `mcp` | Start Model Context Protocol (MCP) JSON-RPC 2.0 stdio server | `voxforg mcp` |

---

### 1.2 `voxforg serve`

Launches the VoxForg application daemon and embedded visual workstation.

```bash
voxforg serve [OPTIONS]
```

#### Options:
- `-p, --port <PORT>`: Port to bind server to (Default: `8080`).
- `--host <HOST>`: Network interface address (Default: `0.0.0.0`).
- `--open`: Automatically open default web browser to workstation interface on startup.
- `--api-key <KEY>`: Optional Bearer token for API authentication (or via `VOXFORG_API_KEY`).
- `--data-dir <PATH>`: Directory for pipeline persistence and cache (Default: `./data`).
- `--router-url <URL>`: Upstream model router endpoint URL (e.g. `https://api.openai.com/v1` or `http://localhost:8000/v1`).
- `--router-api-key <KEY>`: API key for upstream model router.
- `--router-model <MODEL>`: Default TTS model name for router (e.g. `tts-1`, `kokoro`).

---

### 1.3 `voxforg synth`

Generates audio directly to disk without running the daemon.

```bash
voxforg synth "<TEXT>" [OPTIONS]
```

#### Options:
- `-v, --voice <VOICE_ID>`: Voice identifier (e.g. `en-US-AriaNeural`, `alloy`, `echo`, `mock-en-female`).
- `-o, --out <PATH>`: Output audio file path (Default: `output.wav`).
- `--speed <FLOAT>`: Speaking rate multiplier from `0.25` to `4.0` (Default: `1.0`).
- `--pitch <FLOAT>`: Pitch shift in semitones from `-50.0` to `50.0` (Default: `0.0`).
- `--router-url <URL>`: Optional upstream model router URL.
- `--router-api-key <KEY>`: Optional upstream router key.

---

### 1.4 `voxforg transcribe`

Transcribes speech audio or extracts audio from video containers, generating formatted text, word-level timestamps, or subtitle files (SRT / VTT).

```bash
voxforg transcribe <AUDIO_PATH> [OPTIONS]
```

#### Options:
- `-f, --format <FORMAT>`: Output format: `text`, `srt`, `vtt`, `json` (Default: `text`).
- `-l, --language <LANG>`: Optional language hint (e.g. `en`, `es`, `fr`, `de`).
- `-o, --out <PATH>`: Output file path (Default: stdout).

#### Examples:
```bash
# Generate subtitle file for video
voxforg transcribe interview.mp4 --format srt --out interview.srt

# Verbose JSON with word timestamps
voxforg transcribe lecture.wav --format json --out timestamps.json
```

---

### 1.5 `voxforg clone`

Extracts a 512-dimensional zero-shot speaker acoustic embedding vector from a short clean reference audio sample (3-15 seconds) and saves a reusable `VoiceProfile` JSON file.

```bash
voxforg clone --name "<NAME>" --audio <PATH> [OPTIONS]
```

#### Options:
- `-n, --name <NAME>`: Human-readable display name for the cloned voice.
- `-a, --audio <PATH>`: Path to reference audio clip (`.wav`, `.mp3`, `.flac`).
- `-t, --transcript <TEXT>`: Optional ground-truth transcript spoken in reference audio (improves acoustic alignment).
- `-l, --language <LANG>`: Language code (Default: `en-US`).
- `-g, --gender <GENDER>`: Target gender: `male`, `female`, `neutral` (Default: `neutral`).
- `-o, --out <PATH>`: Output voice profile JSON file path (Default: `voice_profile.json`).

---

### 1.6 `voxforg models`

Inspects the curated open-weights model catalogue, checks host hardware compatibility against RAM/VRAM requirements, and triggers streaming downloads from HuggingFace.

```bash
# List available and installed models
voxforg models list [--filter tts|asr|vad|diarizer] [--installed-only]

# Download and install model weights
voxforg models install <MODEL_ID>
```

#### Example:
```bash
voxforg models list
voxforg models install kokoro-v0_19
voxforg models install piper-en-lessac-medium
```

---

### 1.7 `voxforg ab-test`

Runs automated acoustic quality, latency, and distortion comparisons between two engines or voices.

```bash
voxforg ab-test \
  --name "Neural vs Cloud Comparison" \
  --text "The quick brown fox jumps over the lazy dog." \
  --voice-a en-US-AriaNeural \
  --voice-b alloy \
  --iterations 3
```

#### Evaluated Metrics:
- **Latency (ms)**: Time to synthesize full audio chunk.
- **Audio Duration (s)**: Length of synthesized audio.
- **Real-Time Factor (RTF)**: `Latency / Audio Duration`. (Lower is faster; `< 1.0` is real-time).
- **Peak Amplitude (dBFS)**: Maximum sample level.
- **RMS Loudness (dBFS)**: Root-mean-square average energy.
- **Clipping Sample Count**: Number of digital samples hitting or exceeding `0.0 dBFS`.
 
---

### 1.8 `voxforg mcp`

Launches an in-process Model Context Protocol (MCP) server adhering to JSON-RPC 2.0 over standard I/O (`stdio`). This exposes 8 core tools (`synthesize_speech`, `clone_voice`, `transcribe_audio`, `execute_pipeline`, `list_voices`, `list_models`, `benchmark_engine`, `get_cluster_status`) directly to LLM agent workstations like Claude Desktop or Cursor.

```bash
voxforg mcp
```

#### Claude Desktop Integration (`claude_desktop_config.json`):
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

---

## 2. Environment Variables Reference

| Variable | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `VOXFORG_API_KEY` | String | `None` | Enforces Bearer token authentication on `/v1/*` API endpoints |
| `VOXFORG_ROUTER_URL` | String | `None` | Upstream OpenAI-compatible router endpoint (`/v1/audio/speech`) |
| `VOXFORG_ROUTER_API_KEY` | String | `None` | Bearer token for upstream model router (automatically masked in logs) |
| `VOXFORG_ROUTER_MODEL` | String | `tts-1` | Model identifier sent in router payloads |
| `VOXFORG_ASR_URL` | String | `None` | Upstream OpenAI-compatible transcription endpoint (Faster-Whisper / WhisperX) |
| `VOXFORG_QWEN3_URL` | String | `None` | Upstream zero-shot voice cloning service endpoint |
| `VOXFORG_FFMPEG_PATH` | String | `ffmpeg` | Custom binary path for FFmpeg audio demuxing and video muxing |
| `RUST_LOG` | String | `info` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`) |

---

## 3. Webstation UI Walkthrough

The visual workstation interface runs in modern browsers at `http://localhost:8080`.

### 3.1 Pipeline Canvas (Multi-Voice Studio DAG Builder)

The Pipeline Canvas is a visual node-based Directed Acyclic Graph (DAG) editor designed for complex multi-voice speech production, including podcasts, tactical communications, radio broadcasting, narrative audiobooks, and video dubbing.

#### Infinite 2D Viewport & Navigation
- **Pan Navigation**: Click and drag anywhere on the canvas background to pan infinitely in 2D space.
- **Smooth Cursor-Anchored Zoom**: Scroll the mouse wheel to zoom continuously between `0.25x` and `2.5x`, centered precisely on your cursor position.
- **Floating HUD Toolbar**:
  - `+` / `-`: Zoom in and out incrementally.
  - `100%`: Reset zoom to native scale.
  - `Fit View`: Automatically frame all pipeline nodes into the viewport.
  - `Auto-Align`: Neatly organizes all nodes into clean left-to-right topological columns.
- **Dynamic Wires**: Curved cubic Bezier connection wires (`M x1 y1 C ...`) link node output ports to downstream input ports with animated glowing energy pulses.
- **Freeform Node Dragging**: Grab and position any node card anywhere on the canvas with real-time wire tracking.

#### Core Node Architecture
1. **`TextInput` (Script Input)**:
   - Enter dialogue or monologue text.
   - Supports standard multi-speaker syntax:
     - `Host: Welcome to the studio!` / `Guest: Glad to be here!`
     - `[Dispatch] Falcon-1, report status.` / `[Falcon-1] Waypoint Alpha cleared.`
     - `(Captain) Status report.` / `(Nav) Hyperdrive ready.`
2. **`SpeakerParser` (Dialogue Segmenter)**:
   - Automatically splits multi-speaker script into discrete chronological utterances with speaker labels.
3. **`VoiceAssigner` (Speaker-to-Voice Router)**:
   - **Auto-Detect from Script**: Scans all script nodes in the canvas, extracts unique speaker tags, and automatically populates speaker mapping with distinct neural voices.
   - **Manual Speaker Mapping**: Assign specific voices (e.g. `JennyNeural` for Host, `GuyNeural` for Guest, `AriaNeural` for Dispatch).
   - **Default Voice Profile**: Fallback voice for unassigned or narrator lines.
4. **`Synthesizer` (Neural Speech Engine)**:
   - Synthesizes each speaker segment individually using assigned voices.
   - Controls speaking rate multiplier (`0.50x` to `2.00x`) and pitch shift (`-12` to `+12` semitones).
   - Built-in automatic fallback ensures synthesis continues even if a cloud voice is offline.
5. **`AudioFilter` (Studio DSP Mastering)**:
   - **3-Band Parametric EQ**: Low Shelf (250Hz), Mid Peaking (1kHz), High Shelf (4kHz) based on Robert Bristow-Johnson biquad filter equations.
   - **Dynamic Compressor**: Threshold (`-36` to `-6 dBFS`), ratio (`1.5:1` to `8:1`), attack, release, and makeup gain.
   - **Brickwall Limiter**: Clamps peaks below `-0.5 dBFS` with soft-knee cubic saturation to prevent digital clipping.
   - **Silence Trimmer**: Windowed energy detector removing dead air at segment start and end.
   - **Peak Normalizer**: Scales amplitude to 95% (-0.5 dBFS) for broadcast loudness compliance.
6. **`AudioMerge` (Crossfade & Multiplexing)**:
   - Sequentially stitches processed speaker segments with customizable inter-speaker pause durations (`pause_ms`).
   - Quick presets:
     - **Rapid Comms (80ms)**: Fast-paced tactical transmissions and radio dispatch.
     - **Conversational (150ms)**: Natural dialogue and casual interviews.
     - **Podcast Studio (220ms)**: Professional studio interview pacing.
     - **Dramatic Pause (450ms)**: Cinematic narrative storytelling.
7. **`OutputSink` (Master Audio Export)**:
   - Assembles final master broadcast audio buffer into 16-bit linear PCM RIFF WAV format.

#### Built-In Studio Presets
- **Multi-Voice Studio Podcast**: Host (Alex) and Guest (Dr. Vance) interview, studio broadcast DSP mastering, and 220ms conversational pause.
- **Tactical Communications Studio**: Dispatch and Falcon-1 pilot callsigns, 1.1x speaking rate, VHF radio bandpass EQ (-6dB low shelf, +4dB high boost), and 140ms channel multiplexing.
- **Narrative Sci-Fi Dialogue**: Space dialogue between Captain and Navigator with 450ms dramatic pauses.
- **Punchy Radio Broadcaster**: Fast-paced commercial broadcast pipeline with DJ and Caller.

#### Execution & Live Telemetry
- Click **"Run Pipeline"** in the top navigation bar to compile and execute the active DAG graph.
- **Real-Time Audio Telemetry**: Integrated dock featuring HTML5 Web Audio API visualizers:
  - **FFT Spectrum Analyzer**: 64-band real-time frequency bar graph.
  - **Oscilloscope Waveform**: Real-time time-domain audio wave rendering.
  - **Master Playback & Download**: Immediate in-browser playback and one-click master WAV export.

### 3.2 Voice Lab & Zero-Shot Cloning Studio
- **Voice Selection & Testing**: Filter 18+ voices across language and engine.
- **Parametric Sliders**: Real-time speaking rate (`0.5x` to `2.0x`) and pitch tuning (`-12` to `+12` semitones).
- **Audio Oscilloscope & FFT Spectrum**: Real-time time-domain and frequency-domain visualizers.
- **Voice Cloning Studio**: Click "Clone New Voice" in the sidebar to upload a reference audio sample, input optional ground-truth transcript, select target gender, and extract an acoustic profile into your voice library.

### 3.3 Model Catalogue
- Comprehensive inventory of open-weights models categorized by TTS, ASR, VAD, and Diarization.
- Host RAM/VRAM compatibility indicators prevent out-of-memory crashes before initiating downloads.
- Direct one-click install button with live progress spinner and status badges.

### 3.4 Automated A/B & QA Lab
- Interactive UI to configure comparison scenarios, run batch benchmarks, and view side-by-side metric tables with automated winner recommendations.

---

## 4. OpenAI Drop-In API Integration

VoxForg replaces OpenAI TTS in existing applications without changing application logic.

### 4.1 Python OpenAI SDK
```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="your-key-here"  # Matches VOXFORG_API_KEY if enabled
)

response = client.audio.speech.create(
    model="edge-tts",
    voice="en-US-AriaNeural",
    input="VoxForg provides zero-latency neural speech generation."
)

response.stream_to_file("output.wav")
```

### 4.2 TypeScript / JavaScript SDK
```typescript
import OpenAI from "openai";
import fs from "fs";

const openai = new OpenAI({
  baseURL: "http://localhost:8080/v1",
  apiKey: "your-key-here",
});

async function main() {
  const response = await openai.audio.speech.create({
    model: "openai-router",
    voice: "alloy",
    input: "Hello from VoxForg enterprise audio platform!",
  });

  const buffer = Buffer.from(await response.arrayBuffer());
  await fs.promises.writeFile("output.wav", buffer);
}

main();
```

---

## 5. Prometheus Telemetry & Monitoring

VoxForg includes a native Prometheus metrics exporter at `GET /metrics`.

Key metrics exported:
- `voxforg_requests_total{endpoint, status}`
- `voxforg_synthesis_total`
- `voxforg_synthesis_duration_seconds_total`
- `voxforg_audio_samples_total`
- `voxforg_cache_hits_total` & `voxforg_cache_misses_total`
- `voxforg_active_requests`
