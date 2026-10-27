# Model Context Protocol (MCP) & AI Agent Integration Guide

VoxForg provides a first-class, enterprise-hardened **Model Context Protocol (MCP)** implementation (`crates/voxforg-mcp`). This enables any agentic IDE or AI coding framework (Cursor, Claude Desktop, Google Antigravity, Windsurf, VSCode Cline / Roo Code, AutoGen, CrewAI, LangChain) to connect to VoxForg, discover capabilities, inspect real-time compute hardware, synthesize studio-quality speech, clone voices, transcribe media, and orchestrate complex audio DAG pipelines.

---

## 1. Architectural Transports

VoxForg supports two distinct MCP communication transports:

```
┌──────────────────────────────────────────────────────────┐
│                   Agentic IDE / Client                   │
│   (Claude Desktop, Cursor, Antigravity, Windsurf, Roo)   │
└──────────────┬────────────────────────────┬──────────────┘
               │ Local Process              │ Remote / Network
               │ (JSON-RPC 2.0 via STDIO)   │ (Streamable HTTP / SSE)
               ▼                            ▼
┌──────────────────────────────┐ ┌─────────────────────────┐
│     voxforg-cli (mcp)        │ │   voxforg-api (serve)   │
│  - Dedicated stdio pipes     │ │  - /mcp (HTTP POST)     │
│  - Stderr-routed logging     │ │  - /mcp (SSE GET)       │
│  - Zero network port binding │ │  - Constant-time Auth   │
│  - Strict Path Sandboxing    │ │  - Strict CORS Policy   │
└──────────────┬───────────────┘ └──────────┬──────────────┘
               │                            │
               ▼                            ▼
┌──────────────────────────────────────────────────────────┐
│                   VoxForg Core Runtimes                  │
│  - Engines: Edge, Kokoro, Piper, Qwen3-TTS, OpenAI Router │
│  - Whisper ASR & Video Demuxer                           │
│  - Audio DSP: Watermarking, Time-Stretch, Equalizer      │
│  - Hardware Probe (SIMD, VRAM Budget, CUDA/DirectML)     │
└──────────────────────────────────────────────────────────┘
```

### Transport 1: Local Standard I/O (`stdio`)
- **Invocation:** `voxforg mcp` or `cargo run -p voxforg-cli -- mcp`
- **Mechanism:** Standard JSON-RPC 2.0 messages delimited by newlines over `stdin` and `stdout`.
- **Log Isolation:** All internal tracing and debug logs are routed strictly to `stderr` (`.with_writer(std::io::stderr)`), guaranteeing zero non-JSON pollution on `stdout`.
- **Security:** Zero network exposure. Bounded entirely to the local operating system process permissions and IDE lifecycle.

### Transport 2: Streamable HTTP POST & Server-Sent Events (SSE)
- **Invocation:** `voxforg serve --port 8080 --api-key <secret>`
- **HTTP POST Endpoint:** `POST /mcp` — processes individual JSON-RPC requests.
- **SSE Stream Endpoint:** `GET /mcp` — emits SSE events announcing endpoint availability.
- **Security:** Enforced token authentication, DNS rebinding validation, payload length limits, and strict origin allowlists.

---

## 2. Complete MCP Tool Registry (14 Tools)

VoxForg exposes 14 tools covering the full spectrum of speech engineering:

### 1. `synthesize_speech`
Synthesizes speech using local neural TTS engines (Edge-TTS, Kokoro, Piper, Qwen3-TTS), upstream routers, or cloned voices.
- **Inputs:**
  - `text` (*string, required*): Text content to speak (max 100,000 characters).
  - `voice` (*string, required*): Voice ID, persona name, or cloned profile ID.
  - `speed` (*number, optional, default: 1.0*): Speech rate multiplier (0.25 to 4.0).
  - `pitch` (*number, optional, default: 0.0*): Pitch shift in semitones (-12.0 to 12.0).
  - `response_format` (*string, optional, enum: `["wav", "pcm"]`, default: `"wav"`*).
  - `output_file` (*string, optional*): Local filesystem path to save audio directly. **Recommended for Agentic IDEs** to avoid polluting model token context with large base64/hex audio data.
- **Output:** Returns status, voice profile, duration in seconds, sample rate, and either `output_file` metadata or `audio_wav_hex`.

### 2. `clone_voice`
Extracts high-dimensional speaker embeddings from reference audio to create a persistent cloned voice profile.
- **Inputs:**
  - `name` (*string, required*): Display name for the new cloned voice profile.
  - `reference_audio_path` (*string, optional*): Local filesystem path to reference audio file (.wav, .mp3).
  - `reference_audio_base64` (*string, optional*): Base64 or hex encoded reference audio.
  - `engine_id` (*string, optional, default: `"qwen3-tts"`*): Target cloning engine.
  - `language` (*string, optional, default: `"en-US"`*): ISO language code.
  - `reference_transcript` (*string, optional*): Exact text spoken in reference audio to boost cloning fidelity.
- **Output:** Returns `profile_id`, name, embedding dimension (e.g. 512), and language.

### 3. `transcribe_audio`
Transcribes speech audio or video media into text or timed subtitles using Whisper ASR engines.
- **Inputs:**
  - `audio_path` (*string, optional*): Local path to audio or video file (`.wav`, `.mp3`, `.mp4`, `.mkv`, `.webm`). Automatically demuxes video tracks using `MediaProcessor`.
  - `audio_base64` (*string, optional*): Base64 or hex encoded WAV/PCM audio.
  - `model` (*string, optional*): ASR model identifier (e.g. `whisper-base`).
  - `language` (*string, optional*): Language code hint (e.g. `en`, `es`).
  - `response_format` (*string, optional, enum: `["json", "verbose_json", "srt", "vtt", "text"]`, default: `"json"`*).
- **Output:** Structured transcript with word-level timestamps, SRT subtitles, VTT, or plain text.

### 4. `speech_to_speech`
Converts source voice audio recording into a target voice persona while preserving timing and cadence.
- **Inputs:**
  - `target_voice` (*string, required*): Target voice ID or profile to convert into.
  - `source_audio_path` (*string, optional*): Path to source speech audio file.
  - `source_audio_base64` (*string, optional*): Base64 or hex encoded source speech.
  - `speed` (*number, optional, default: 1.0*): Speed multiplier (0.5 to 2.5).
  - `preserve_tempo` (*boolean, optional*): Whether to preserve exact source duration.
  - `output_file` (*string, optional*): Local path to save converted WAV audio.
- **Output:** Transcribed text, target voice, duration, and output file path or audio hex.

### 5. `execute_pipeline`
Executes pre-built production speech DAG pipelines.
- **Inputs:**
  - `template` (*string, required, enum: `["video_dubbing", "audiobook"]`*).
  - `title` (*string, optional*): Pipeline execution title.
  - `voice` (*string, optional*): Primary voice identifier.
  - `input_text` (*string, optional*): Manuscript or dialogue text.
  - `output_file` (*string, optional*): Path to save pipeline master WAV audio.
- **Output:** Pipeline ID, execution name, output byte size, and output audio file.

### 6. `manage_pronunciation`
Runtime management of custom phonetic replacements, brand names, and heteronyms.
- **Inputs:**
  - `action` (*string, required, enum: `["list", "add", "remove", "apply"]`*).
  - `term` (*string, optional*): Term to override (e.g. `"SQL"`, `"nginx"`).
  - `replacement` (*string, optional*): Phonetic replacement (e.g. `"sequel"`, `"engine X"`).
  - `text` (*string, optional*): Sample text to test against dictionary normalization.
- **Output:** List of dictionary entries, addition confirmation, or normalized text.

### 7. `refine_text`
Cleans dictated, transcribed, or LLM-drafted speech text by removing conversational filler words and Whisper hallucination repetition loops.
- **Inputs:**
  - `text` (*string, required*): Raw text to refine.
  - `strip_fillers` (*boolean, optional, default: true*): Removes `um`, `uh`, `like`, `you know`, etc.
  - `strip_hallucinations` (*boolean, optional, default: true*): Removes Whisper loop repetitions.
- **Output:** Original text, refined text, count of fillers removed, count of loops removed.

### 8. `embed_watermark`
Embeds an invisible spread-spectrum provenance watermark signature into a WAV recording.
- **Inputs:**
  - `audio_path` (*string, optional*): Path to source WAV audio file.
  - `audio_base64` (*string, optional*): Base64 WAV data.
  - `payload` (*integer, optional, default: 0x5658*): 16-bit payload signature (0 - 65535).
  - `output_file` (*string, optional*): Path to save watermarked WAV audio.
- **Output:** Status, signature payload, repetition count, duration, output file.

### 9. `verify_watermark`
Detects and verifies invisible provenance watermarks in an audio recording.
- **Inputs:**
  - `audio_path` (*string, optional*): Path to WAV audio file.
  - `audio_base64` (*string, optional*): Base64 WAV data.
- **Output:** `is_detected` (bool), `payload` (u16), `payload_hex` (string), `confidence` (float), `signature_match` (bool).

### 10. `list_voices`
Lists all available voices across registered TTS engines, portable voice identities, and persistent cloned profiles.
- **Inputs:**
  - `language` (*string, optional*): ISO language filter (e.g. `en`, `es`, `fr`).
- **Output:** Array of voice objects with ID, engine, language, gender, and cloned flag.

### 11. `list_models`
Browses the local open-weights neural model catalogue.
- **Inputs:**
  - `type` (*string, optional, enum: `["tts", "asr", "vad", "diarizer"]`*).
  - `installed_only` (*boolean, optional*): Only show downloaded models.
- **Output:** Model count and models with ID, type, status, size, and minimum RAM requirement.

### 12. `install_model`
Downloads and verifies neural model weight packages into the local cache.
- **Inputs:**
  - `model_id` (*string, required*): Model package ID (e.g. `kokoro-v0_19`, `piper-en-lessac-medium`).
- **Output:** Installation status, model ID, display name, and local cache path.

### 13. `benchmark_engine`
Runs latency, throughput, and Real-Time Factor (RTF) benchmarks on any registered TTS engine.
- **Inputs:**
  - `engine_id` (*string, optional, default: `"mock-tts"`*).
- **Output:** Engine ID, sentences tested, average latency in milliseconds, RTF, quality MOS score.

### 14. `get_cluster_status`
Inspects local compute accelerators and active distributed cluster worker nodes.
- **Inputs:** None.
- **Output:** Host OS, CPU architecture, total RAM, GPU device list, active worker count, worker metrics.

---

## 3. MCP Resources

Agents can read system state directly without invoking tools:

| Resource URI | Description | MIME Type |
|---|---|---|
| `voxforg://voices` | Complete JSON inventory of voices, engines, and cloned profiles | `application/json` |
| `voxforg://models` | Neural models in catalogue and their installation state | `application/json` |
| `voxforg://hardware` | CPU SIMD features, system memory, GPUs, VRAM allocations | `application/json` |
| `voxforg://cluster` | Active distributed worker nodes, status, and health metrics | `application/json` |

---

## 4. MCP Prompts

Pre-engineered prompt workflows available to agents:
- `speech_production`: Guided workflow to take raw text script, run pronunciation dictionary normalization, refine fillers, and synthesize master audio.
- `audio_transcription_and_dubbing`: Workflow to take video/audio media path, transcribe with word timestamps, and synthesize dubbed track in a target voice.

---

## 5. Security & Enterprise Hardening

The MCP implementation follows a defense-in-depth security model:

1. **Authentication Enforcement (`auth_middleware`):**
   When `VOXFORG_API_KEY` is configured, both `/v1/*` and `/mcp` endpoints require authentication. Tokens can be provided via `Authorization: Bearer <key>`, `x-api-key: <key>`, or `?api_key=<key>` (query parameter support enables SSE browser connections). Tokens are compared strictly using constant-time equality (`subtle::ConstantTimeEq`).
2. **Restricted CORS Policy:**
   Wildcard CORS (`*`) is strictly blocked. Only requests from `localhost`, `127.0.0.1`, `[::1]`, IDE extension origins (`vscode-webview://`, `app://`), or explicit `VOXFORG_ALLOWED_ORIGINS` are accepted. Prevents malicious web pages from interacting with local VoxForg instances.
3. **Host Header Validation:**
   Protects local loopback endpoints from DNS rebinding attacks. Rejects requests with non-whitelisted Host headers.
4. **Strict Path Sandboxing:**
   All tool arguments accepting filesystem paths (`output_file`, `audio_path`, `reference_audio_path`, `source_audio_path`) are sanitized:
   - Traversal sequences (`..`) are strictly rejected.
   - Null bytes (`\0`) are forbidden.
   - Operating system protected paths (`C:\Windows`, `C:\Program Files`, `/etc`, `/proc`, `/sys`) are blocked.
   - File extensions are validated against an allowlist (`.wav`, `.pcm`, `.json`, `.srt`, `.vtt`, `.txt`, `.mp3`, `.mp4`, `.mkv`).
5. **Memory Caps & DoS Protection:**
   Text inputs are capped at 100,000 characters. Audio base64 payloads are capped at 32MB string length before memory allocation.

---

## 6. Client Configuration Snippets

### A. Cursor (`.cursor/mcp.json` or `.mcp.json`)
```json
{
  "mcpServers": {
    "voxforg": {
      "command": "cargo",
      "args": ["run", "-p", "voxforg-cli", "--", "mcp"],
      "env": {
        "RUST_LOG": "info",
        "VOXFORG_PROFILES_DIR": "./data/profiles",
        "VOXFORG_MODELS_DIR": "./data/models"
      }
    }
  }
}
```

### B. Claude Desktop (`%APPDATA%\Claude\claude_desktop_config.json`)
```json
{
  "mcpServers": {
    "voxforg": {
      "command": "d:\\PROJECTS\\AI_BASED_PROJECTS\\VoxForg\\target\\debug\\voxforg.exe",
      "args": ["mcp"]
    }
  }
}
```

### C. Windsurf (`~/.codeium/windsurf/mcp_config.json`)
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

### D. VSCode Cline / Roo Code (`cline_mcp_settings.json`)
```json
{
  "mcpServers": {
    "voxforg": {
      "command": "cargo",
      "args": ["run", "-p", "voxforg-cli", "--", "mcp"]
    }
  }
}
```

### E. Remote / Daemon SSE Mode (Any Client)
Start server:
```bash
cargo run -p voxforg-cli -- serve --port 8080 --api-key your-secure-token
```
Client configuration:
```json
{
  "mcpServers": {
    "voxforg-remote": {
      "url": "http://127.0.0.1:8080/mcp?api_key=your-secure-token"
    }
  }
}
```
