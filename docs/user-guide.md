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

The visual workstation interface runs in modern browsers at `http://localhost:8080` (or `http://localhost:3000` during frontend development).

### 3.1 Workflows Dashboard (n8n-Style Multi-Project Management)

The Workflows Dashboard serves as the command center for multi-project speech pipelines, providing an n8n-inspired workspace overview:

- **Project Gallery & Grid View**: Browse all configured pipelines with live metadata badges displaying node counts, category tags (`Podcast`, `Tactical`, `Dubbing`, `Radio`), and last updated timestamps.
- **Instant Search & Filtering**: Type in the top search bar to filter pipelines dynamically across name, description, and tag attributes.
- **Workflow Actions**:
  - **Open**: Enters the visual DAG Canvas editor with clean initial state and real-time execution controls.
  - **Duplicate**: Clones any active or template workflow into an independent copy for rapid iteration.
  - **Delete**: Safely removes redundant pipelines with built-in confirmation dialog.
  - **Export JSON**: Downloads the full pipeline graph (nodes, connection edges, parameters, and metadata) as an importable JSON file.
- **Create & Import Workflows**:
  - **"New Workflow" Button**: Creates a clean slate pipeline with default inputs and master audio sinks ready for composition.
  - **"Import JSON" Button**: Opens the native file dialog or drag-and-drop zone to import any VoxForg or n8n-exported audio pipeline schema.
- **Breadcrumb Navigation**: Header displays breadcrumb link (`Workflows / <Workflow Name>`), enabling one-click return to the central dashboard at any time without losing edits.

---

### 3.2 Pipeline Canvas (Modern DAG Studio & Ergonomics)

The Pipeline Canvas is a visual node-based Directed Acyclic Graph (DAG) editor engineered for intricate multi-speaker speech production, video dubbing, tactical communications, and automated audio mastering.

#### Infinite 2D Viewport & Navigation
- **2D Smooth Pan**: Click and drag anywhere on the canvas background to pan infinitely in 2D space without keyboard modifier requirements.
- **Pointer-Anchored Focal Zoom**: Double-click anywhere on the canvas to zoom in directly focused on your cursor location. Continuous mouse wheel and trackpad zooming scales between `0.25x` and `2.5x`.
- **Floating HUD Toolbar**:
  - `+` / `-`: Incremental zoom controls.
  - `100%`: Reset zoom to native 1:1 scale.
  - `Fit View`: Automatically calculates bounding box of all nodes and frames them into the viewport.
  - `Auto-Align`: Neatly organizes all nodes into clean left-to-right topological columns.
  - **16px Grid Snapping**: Toggleable magnetic alignment grid for crisp, clean node layout.

#### Selection & Editing Ergonomics
- **Clean Initial State**: Canvas initializes with zero nodes selected, keeping the viewport uncluttered and preventing accidental edits.
- **Single Node Selection**: Clicking any node card selects only that node and slides open the right-side Node Inspector drawer with tabs for parameters, configuration, and step outputs. Clicking empty canvas space or pressing `Escape` deselects and closes the inspector.
- **Marquee Multi-Selection**: Click and drag on empty canvas space to draw a golden dashed bounding box (`stroke-dasharray="4 4"`). All intersecting nodes are automatically added to the selection set. Hold `Shift` or `Ctrl` to perform union selection additions.
- **Multi-Node Drag**: Dragging any selected node moves the entire selected group simultaneously, snapped cleanly to 16px grid increments.
- **Multi-Node Subgraph JSON Copy**: Pressing `Ctrl+C` (or `Cmd+C`) with multiple nodes selected exports the selected subgraph (nodes, configuration, and internal connections) directly to your clipboard as formatted JSON, with on-screen toast confirmation.
- **Searchable Quick-Add Node Palette (`Tab` / `/`)**:
  - Pressing `Tab` or `/` anywhere on the canvas opens a floating searchable palette.
  - Instant live search filters available nodes by label, category (`Input`, `Parsing`, `Routing`, `Synthesis`, `DSP`, `Assembly`, `Output`), or description.
  - Use `ArrowUp` / `ArrowDown` and `Enter` (or direct mouse click) to insert the node at the pointer or canvas center.
- **Dynamic Connection Wires & Beacons**:
  - Dragging from an output port draws an active glowing cubic Bezier wire (`M x1 y1 C ...`).
  - Available downstream target ports illuminate with a pulsating radial beacon ring (`ring-4 ring-amber-400/50 scale-125`), providing immediate visual target feedback.
  - Hovering over any existing wire displays a central disconnect circle with an `×` cut icon, allowing one-click wire severing.

#### Execution Engine & Node Bypass Telemetry
- **Kahn's Topological Execution Ordering**: When clicking "Run Pipeline" (or `Ctrl+Enter`), the client-side scheduler analyzes graph adjacency lists and in-degrees, executing upstream dependencies before downstream consumers.
- **Node Bypass Toggle**: Each node header features a bypass button. When toggled, the node enters `bypassed` state, rendering with reduced opacity and a striped amber badge. Upstream audio signals pass cleanly through without breaking pipeline execution.
- **Real-Time Node State Badges**:
  - `Idle`: Resting state with neutral slate border.
  - `Waiting`: Node queued for execution pending upstream dependencies.
  - `Running`: Active glowing aura with spinning progress indicator.
  - `Success`: Emerald checkmark badge with execution latency in milliseconds (e.g. `142ms`).
  - `Error`: Crimson badge with expandable diagnostic error message.
  - `Bypassed`: Amber striped badge indicating processing was skipped.

#### Keyboard Shortcuts Reference

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| `Tab` or `/` | Open Searchable Quick-Add Node Palette | Global Canvas |
| `Delete` or `Backspace` | Delete currently selected node(s) | Active Selection |
| `Ctrl+C` / `Cmd+C` | Copy selected node(s) / subgraph as JSON | Canvas Selection |
| `Ctrl+V` / `Cmd+V` | Paste nodes from clipboard | Canvas Center |
| `Ctrl+A` / `Cmd+A` | Select all nodes on canvas | Global Canvas |
| `Ctrl+Enter` | Run / Execute full DAG Pipeline | Global Canvas |
| `Escape` | Deselect all nodes & close inspector/palette | Global Viewport |
| `Space` + Drag | Pan canvas viewport | Viewport Navigation |
| Double Click | Zoom in focused on pointer location | Viewport Navigation |

---

### 3.3 Voice Lab & Zero-Shot Acoustic Cloning Studio

The Voice Lab provides an interactive sound engineering playground for exploring voices, tuning prosody, and cloning zero-shot speaker profiles from audio samples.

#### Acoustic Playground & Visualizers
- **Engine & Voice Filter**: Filter across 18+ voices by language, gender, and provider (Edge-TTS, Piper, Kokoro-82M, Qwen3-TTS, KittenTTS).
- **Parametric Sliders**: Real-time speaking rate (`0.50x` to `2.00x`) and pitch adjustment (`-12` to `+12` semitones).
- **Integrated FFT Spectrum Analyzer**: 64-band live frequency visualizer showing harmonic distribution during playback.
- **Oscilloscope Waveform**: Real-time time-domain audio rendering showing dynamics and zero-crossings.
- **Resilient Offline Studio Synthesis**: In-browser DSP fallback generates calibrated acoustic preview audio if the backend daemon is offline, ensuring uninterrupted UI testing.

#### Voice Cloning Engine & Signal Pipeline
Clicking **"Clone New Voice"** launches the studio voice cloning interface:
1. **Reference Audio Ingestion**: Drag and drop or browse any clean speech sample (`.wav`, `.mp3`, `.m4a`, `.ogg`, `.flac`, 3–15 seconds duration).
2. **Audio Preprocessing Suite**:
   - **80Hz 2nd-Order Butterworth High-Pass Filter**: Cuts DC rumble, handling noise, and AC mains hum without affecting vocal fundamentals.
   - **Dynamic Noise Gate**: Silences background room noise and breathing below `-42 dBFS` with a 20ms soft-release envelope.
   - **True Peak Normalization**: Automatically scales audio levels to `-0.5 dBFS`, maximizing signal-to-noise ratio while avoiding digital clipping.
   - **Sample Rate Transcoding**: Resamples to linear 16-bit 24,000 Hz mono PCM WAV for neural conditioning.
3. **512-Dimensional Acoustic Embedding**: Conditioning extractors calculate speaker identity vectors capturing pitch contours, formant distribution, and vocal timbre.
4. **Persistent Profile Storage**:
   - Cloned voices are synchronized immediately with browser local storage (`voxforg_custom_voices`) and the backend SQLite/Postgres database.
   - Cloned profiles appear instantly in the Voice Lab directory and in DAG Canvas `VoiceAssigner` dropdown menus.

---

### 3.4 Model Catalogue

- Comprehensive inventory of open-weights models categorized by TTS, ASR, VAD, and Diarization.
- Host RAM/VRAM compatibility indicators prevent out-of-memory crashes before initiating downloads.
- Direct one-click install button with live progress spinner and status badges.

---

### 3.5 Automated A/B & QA Lab

- Interactive UI to configure comparison scenarios, run batch benchmarks, and view side-by-side metric tables with automated winner recommendations.
- Evaluates real-time factor (RTF), TTFB latency, RMS loudness (dBFS), peak amplitude, and digital clipping sample count.

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

---

## 6. Model Context Protocol (MCP) Integration

VoxForg includes a built-in Model Context Protocol (MCP) server adhering to the JSON-RPC 2.0 specification over standard I/O (`stdio`). This enables autonomous LLM coding agents and desktop assistants (Claude Desktop, Cursor, Antigravity, Windsurf) to synthesize speech, clone voices, transcribe audio, and run full DAG pipelines natively.

### 6.1 Available MCP Tools

| Tool Name | Parameters | Description |
| :--- | :--- | :--- |
| `synthesize_speech` | `text`, `voice_id`, `speed`, `pitch`, `engine` | Generates speech audio buffer from input text |
| `clone_voice` | `name`, `audio_path`, `transcript`, `gender` | Extracts 512-D speaker acoustic embedding and saves persistent profile |
| `transcribe_audio` | `audio_path`, `format`, `language` | Transcribes audio to text, SRT, VTT, or timestamped JSON |
| `execute_pipeline` | `pipeline_json` | Runs full multi-speaker audio graph and returns master WAV |
| `list_voices` | `engine`, `language` | Returns all available built-in and cloned voice profiles |
| `list_models` | `filter_category`, `installed_only` | Lists open-weights neural models and host hardware compatibility |
| `benchmark_engine` | `voice_a`, `voice_b`, `iterations` | Runs objective acoustic quality and latency A/B comparison |
| `get_cluster_status` | *(none)* | Returns CPU SIMD capabilities, GPU accelerators, and memory stats |

### 6.2 Agent Configurations

#### Claude Desktop
Add to `claude_desktop_config.json`:
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

#### Cursor / Windsurf
Add to your project's `.cursor/mcp.json` or global configuration:
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

#### Antigravity IDE
Add to `mcp_config.json`:
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

## 7. n8n Workflow Automation Integration

VoxForg connects seamlessly with [n8n](https://n8n.io) to automate speech generation, customer service audio replies, automated podcast publishing, and voice cloning directly inside your automation flows.

### 7.1 Method 1: Drop-In OpenAI Audio Node

Because VoxForg implements the standard OpenAI Audio Speech API (`/v1/audio/speech`), you can use n8n's native **OpenAI Node**:

1. Add an **OpenAI** node to your n8n workflow.
2. Select Resource: **Audio**, Operation: **Generate Speech**.
3. In Credential settings, click **Create New Credential**:
   - **Base URL**: `http://localhost:8080/v1` (or your Docker host IP)
   - **API Key**: Any string (or matches `VOXFORG_API_KEY` if set)
4. Configure parameters:
   - **Model**: `edge-tts` (or `kokoro-82m`, `piper`, `qwen3-tts`)
   - **Voice**: `en-US-AriaNeural` (or any cloned voice ID)
   - **Input Text**: Dynamic expression (e.g. `{{ $json.messageText }}`)
5. Output binary audio is immediately usable in Telegram, WhatsApp, Slack, or S3 nodes.

---

### 7.2 Method 2: HTTP Request Node (Binary Audio Bots)

To send spoken notifications to Telegram, Discord, or Slack:

1. Add an **HTTP Request** node.
2. Method: `POST`
3. URL: `http://localhost:8080/v1/audio/speech`
4. Response Format: **File (Binary)**
5. Put Output in Field: `data`
6. Body Content Type: **JSON**
7. Specify Body:
```json
{
  "model": "edge-tts",
  "voice": "en-US-ChristopherNeural",
  "input": "Attention team: deployment succeeded at {{ $now.format('HH:mm') }}.",
  "response_format": "wav",
  "speed": 1.0
}
```
8. Connect directly to a **Telegram** (Send Audio) or **Slack** (Upload File) node.

---

### 7.3 Method 3: Webhook DAG Pipeline Execution

Execute complex multi-speaker podcasts or audiobooks triggered by webhooks:

1. In VoxForg Workflows Dashboard or DAG Canvas, click **Export JSON** to obtain your pipeline definition.
2. In n8n, add an **HTTP Request** node:
   - Method: `POST`
   - URL: `http://localhost:8080/v1/pipeline/execute`
   - Body Content: Insert the exported pipeline JSON, binding dynamic text fields (e.g. `{{ $json.script }}`).
3. The response returns the fully mastered multi-speaker WAV file buffer.

---

### 7.4 Method 4: Automated Voice Cloning Workflow

Create cloned voices on the fly from user voicemail or voice messages:

1. Receive voice note from WhatsApp or Telegram.
2. Transcode or pass Base64 audio payload to **HTTP Request** node:
   - Method: `POST`
   - URL: `http://localhost:8080/v1/voices/clone`
   - Body:
```json
{
  "name": "Customer-{{ $json.customerId }}",
  "reference_audio_base64": "{{ $binary.data.base64 }}",
  "language": "en-US",
  "gender": "neutral"
}
```
3. Use the returned `voice_id` in subsequent synthesis nodes to respond with their cloned voice persona.

---

### 7.5 Copy-Pasteable n8n Workflow JSON Template

You can copy and paste this complete workflow snippet directly into the n8n canvas (`Ctrl+V`):

```json
{
  "name": "VoxForg Neural Audio Automation",
  "nodes": [
    {
      "parameters": {
        "httpMethod": "POST",
        "path": "speak",
        "responseMode": "lastNode",
        "options": {}
      },
      "id": "1d8b9a10-0001-4000-8000-000000000001",
      "name": "Webhook Inbound",
      "type": "n8n-nodes-base.webhook",
      "typeVersion": 1,
      "position": [240, 300]
    },
    {
      "parameters": {
        "method": "POST",
        "url": "http://localhost:8080/v1/audio/speech",
        "sendBody": true,
        "specifyBody": "json",
        "jsonBody": "={\n  \"model\": \"edge-tts\",\n  \"voice\": \"{{ $json.body.voice || 'en-US-AriaNeural' }}\",\n  \"input\": \"{{ $json.body.text || 'Welcome to VoxForg automated speech processing.' }}\",\n  \"response_format\": \"wav\",\n  \"speed\": 1.0\n}",
        "options": {
          "response": {
            "response": {
              "responseFormat": "file",
              "outputFieldName": "data"
            }
          }
        }
      },
      "id": "1d8b9a10-0001-4000-8000-000000000002",
      "name": "VoxForg Speech Generator",
      "type": "n8n-nodes-base.httpRequest",
      "typeVersion": 4.2,
      "position": [460, 300]
    }
  ],
  "connections": {
    "Webhook Inbound": {
      "main": [
        [
          {
            "node": "VoxForg Speech Generator",
            "type": "main",
            "index": 0
          }
        ]
      ]
    }
  }
}
```

