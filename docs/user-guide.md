# VoxForg User Guide & Manual

Complete operational manual for developers, sound engineers, and system administrators.

---

## 1. CLI Reference Manual

The `voxforg` binary provides a unified command line interface for server operations, direct synthesis, hardware diagnostics, benchmarking, and automated A/B evaluation.

```bash
voxforg <COMMAND> [OPTIONS]
```

### 1.1 Commands Summary

| Command | Description | Example |
| :--- | :--- | :--- |
| `serve` | Start HTTP REST, WebSocket, and Scalar docs server | `voxforg serve --port 8080` |
| `synth` | Synthesize speech text directly to audio file | `voxforg synth "Hello world" --voice alloy --out out.wav` |
| `hardware` | Inspect host CPU, SIMD, RAM, and GPU compute capabilities | `voxforg hardware` |
| `engines` | List registered speech engines and available voices | `voxforg engines` |
| `bench` | Run synthesis latency benchmark across iterations | `voxforg bench --iterations 10` |
| `ab-test` | Run automated objective A/B acoustic quality comparison | `voxforg ab-test --voice-a en-US-AriaNeural --voice-b alloy` |
| `mcp` | Start Model Context Protocol (MCP) JSON-RPC 2.0 stdio server | `voxforg mcp` |

---

### 1.2 `voxforg serve`

Launches the VoxForg HTTP/WS application server.

```bash
voxforg serve [OPTIONS]
```

#### Options:
- `-p, --port <PORT>`: Port to bind server to (Default: `8080`).
- `--host <HOST>`: Network interface address (Default: `0.0.0.0`).
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

### 1.4 `voxforg ab-test`

Runs automated acoustic quality, latency, and distortion comparisons between two engines or voices.

```bash
voxforg ab-test \
  --name "Neural vs Router Comparison" \
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

### 1.5 `voxforg mcp`

Launches an in-process Model Context Protocol (MCP) server adhering to JSON-RPC 2.0 over standard I/O (`stdio`). This exposes 8 core tools (`synthesize_speech`, `clone_voice`, `transcribe_audio`, `execute_pipeline`, `list_voices`, `list_models`, `benchmark_engine`, `get_cluster_status`) directly to LLM agent workstations like Claude Desktop or Cursor.

```bash
voxforg mcp
```

#### Claude Desktop Integration:
Add to your `claude_desktop_config.json`:
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
| `VOXFORG_API_KEY` | String | `None` | Enforces Bearer token authentication on all non-health routes |
| `VOXFORG_ROUTER_URL` | String | `None` | Upstream OpenAI-compatible router endpoint (`/v1/audio/speech`) |
| `VOXFORG_ROUTER_API_KEY` | String | `None` | Bearer token for upstream model router (automatically masked in logs) |
| `VOXFORG_ROUTER_MODEL` | String | `tts-1` | Model identifier sent in router payloads |
| `RUST_LOG` | String | `info` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`) |

---

## 3. Webstation UI Walkthrough

The visual workstation interface runs in modern browsers at `http://localhost:3000` (or embedded in desktop builds).

### 3.1 Voice Lab
- **Voice Selection**: Filter 18+ voices across language (`en-US`, `en-GB`, `de-DE`, `fr-FR`, `zh-CN`, `ja-JP`) and engine (`edge-tts`, `openai-router`, `mock-tts`).
- **Real-Time Web Audio Visualizer**: Live HTML5 Canvas visualization during playback:
  - **FFT Spectrum Mode**: Visualizes frequency energy distribution from 20Hz to 20kHz.
  - **Waveform Mode**: Real-time oscilloscope showing time-domain amplitude dynamics.
- **Parametric Sliders**: Real-time speaking rate (`0.5x` to `2.0x`) and pitch tuning (`-12` to `+12` semitones).

### 3.2 Pipeline Canvas (Visual Audio DAG Builder)
- **Node-Based Composition**: Connect `TextInput` → `SpeakerParser` → `VoiceAssigner` → `Synthesizer` → `AudioFilter` → `OutputSink`.
- **Studio DSP Node**:
  - **Silence Trimmer**: Windowed energy detector trimming dead air before and after speech.
  - **3-Band Parametric EQ**: Low Shelf (250Hz), Mid Peaking (1kHz), High Shelf (4kHz).
  - **Dynamic Compressor**: Envelope follower with configurable threshold, ratio, attack, and release.
  - **Brickwall Limiter**: Soft-knee saturation preventing clipping above `-0.5 dBFS`.
  - **Peak Normalizer**: Automatic gain adjustment to target peak dBFS.
- **Preset Templates**:
  - `Narrative Dialogue`: Multi-speaker dialogue parsing and voice assignment.
  - `Studio Podcast Mastering`: Speech synthesis followed by EQ, compression, and limiting.
  - `Punchy Radio Broadcaster`: Aggressive mid-frequency boost and high-ratio limiting.
- **JSON Export & Import**: One-click download and upload of DAG graph files (`.voxforg.json`).

### 3.3 Automated A/B & QA Lab
- Interactive UI to configure comparison scenarios, run batch benchmarks, and view side-by-side metric tables with automated winner recommendations.

---

## 4. OpenAI Drop-In API Integration

VoxForg can replace OpenAI TTS in existing applications without changing application logic.

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

### 4.3 cURL Example
```bash
curl http://localhost:8080/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "model": "edge-tts",
    "voice": "en-US-JennyNeural",
    "input": "High-fidelity neural speech test.",
    "speed": 1.0
  }' \
  --output test.wav
```

---

## 5. Prometheus Telemetry & Monitoring

VoxForg includes a native, zero-dependency Prometheus metrics exporter at `GET /metrics`.

### 5.1 Prometheus Configuration (`prometheus.yml`)
```yaml
scrape_configs:
  - job_name: 'voxforg'
    scrape_interval: 5s
    static_configs:
      - targets: ['localhost:8080']
```

### 5.2 Key Metrics Exported:
- `voxforg_requests_total{endpoint, status}`: Total request counts partitioned by route and HTTP status code.
- `voxforg_synthesis_total`: Total number of audio speech generations completed.
- `voxforg_synthesis_duration_seconds_total`: Cumulative synthesis compute time in seconds.
- `voxforg_audio_samples_total`: Cumulative count of 16-bit PCM samples generated.
- `voxforg_cache_hits_total` & `voxforg_cache_misses_total`: In-memory LRU audio synthesis cache performance.
- `voxforg_active_requests`: Instantaneous gauge of concurrent requests currently being processed.
