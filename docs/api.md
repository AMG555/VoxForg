# VoxForg API Specification

**Version:** 1.0.0  
**OpenAPI Specification:** 3.1.0  
**Protocol:** HTTPS / WSS  
**Base URL:** `http://localhost:8080` (Default Local)

---

## 1. Authentication & Security

All API endpoints (except `/health` and public documentation) enforce token authentication:

```http
Authorization: Bearer <api_key>
```

### Permission Scopes
- `tts:synthesize`: Generate speech audio via `/v1/audio/speech`.
- `voices:read`: Query voice profiles and engine inventories.
- `pipeline:run`: Submit and execute DAG workflow graphs.
- `pipeline:admin`: Create, update, or delete pipeline definitions.
- `system:admin`: Access `/metrics`, hardware controls, and engine model downloads.

---

## 2. Audio Speech Synthesis (OpenAI Drop-In)

### `POST /v1/audio/speech`

Synthesizes speech audio from text. Fully compliant with OpenAI Speech API specifications with extended VoxForg parameters.

#### Request Headers
```http
Content-Type: application/json
Authorization: Bearer <api_key>
```

#### Request Body
```json
{
  "model": "edge-tts",
  "input": "VoxForg is now operational with hardware-accelerated speech synthesis.",
  "voice": "en-US-AriaNeural",
  "response_format": "wav",
  "speed": 1.0,
  "pitch": 0.0,
  "loudness_lufs": -14.0
}
```

#### Parameters
| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `model` | `string` | Yes | - | Engine ID or model name (`edge-tts`, `kokoro-82m`, `piper`, `qwen3-tts`, or `auto`). |
| `input` | `string` | Yes | - | Raw text or SSML snippet (Max length: 50,000 characters per request). |
| `voice` | `string` | Yes | - | Voice identifier (e.g. `en-US-AriaNeural`, `en_US-heart`). |
| `response_format`| `string` | No | `wav` | Audio container: `wav`, `mp3`, `opus`, `aac`, `flac`, `pcm`. |
| `speed` | `number` | No | `1.0` | Speaking rate multiplier between `0.25` and `4.0`. |
| `pitch` | `number` | No | `0.0` | Pitch shift in semitones between `-12.0` and `+12.0` (Engine-dependent). |
| `loudness_lufs` | `number` | No | `null`| Target EBU R128 integrated loudness (e.g., `-14.0` LUFS for podcasting). |

#### Response
- **Status:** `200 OK`
- **Content-Type:** `audio/wav` (or requested format)
- **Transfer-Encoding:** `chunked` (Streams audio bytes as produced)

---

## 3. Models & Voices Inventory

### `GET /v1/models`

Lists all available speech engines and models.

#### Response
```json
{
  "object": "list",
  "data": [
    {
      "id": "edge-tts",
      "object": "model",
      "created": 1700000000,
      "owned_by": "voxforg-cloud",
      "hardware_tier": "TIER_1_MINIMAL"
    },
    {
      "id": "kokoro-82m",
      "object": "model",
      "created": 1700000000,
      "owned_by": "voxforg-local",
      "hardware_tier": "TIER_3_PRO"
    },
    {
      "id": "piper",
      "object": "model",
      "created": 1700000000,
      "owned_by": "voxforg-local",
      "hardware_tier": "TIER_2_STANDARD"
    }
  ]
}
```

### `GET /v1/voices`

Returns a catalog of registered voices across all engines.

#### Query Parameters
- `language` (optional, e.g. `en-US`, `de-DE`)
- `gender` (optional, `male`, `female`, `neutral`)
- `engine` (optional, e.g. `kokoro-82m`)

#### Response
```json
{
  "voices": [
    {
      "id": "en-US-AriaNeural",
      "name": "Aria",
      "engine": "edge-tts",
      "language": "en-US",
      "gender": "female",
      "sample_rate_hz": 24000,
      "tags": ["conversational", "news", "clear"]
    },
    {
      "id": "en_US-heart",
      "name": "Heart",
      "engine": "kokoro-82m",
      "language": "en-US",
      "gender": "female",
      "sample_rate_hz": 24000,
      "tags": ["audiobook", "warm", "expressive"]
    }
  ]
}
```

---

## 4. Pipeline Execution API

### `POST /v1/pipeline/execute`

Executes an arbitrary DAG audio pipeline.

#### Request Body
```json
{
  "pipeline": {
    "name": "Audiobook Chapter 1",
    "nodes": [
      {
        "id": "node-1",
        "type": "text_input",
        "params": {
          "text": "The wind howled through the mountain pass. 'We must stop here,' said Sean."
        }
      },
      {
        "id": "node-2",
        "type": "speaker_parser",
        "params": {
          "rules": [
            { "speaker": "Sean", "regex": "'([^']*)' said Sean" }
          ]
        }
      },
      {
        "id": "node-3",
        "type": "voice_assigner",
        "params": {
          "default_voice": "en_US-heart",
          "speaker_map": {
            "Sean": "en-US-ChristopherNeural"
          }
        }
      },
      {
        "id": "node-4",
        "type": "synthesizer",
        "params": {
          "format": "wav"
        }
      },
      {
        "id": "node-5",
        "type": "audio_merge",
        "params": {
          "crossfade_ms": 100,
          "normalize_lufs": -14.0
        }
      }
    ],
    "edges": [
      { "from": "node-1", "to": "node-2" },
      { "from": "node-2", "to": "node-3" },
      { "from": "node-3", "to": "node-4" },
      { "from": "node-4", "to": "node-5" }
    ]
  }
}
```

#### Response
- **Status:** `200 OK`
- **Content-Type:** `audio/wav`
- Streaming binary output of merged pipeline audio.

---

## 5. WebSocket Real-Time Audio Streaming

### `GET /v1/stream`

Bi-directional WebSocket protocol for real-time conversational agents and chunked synthesis.

#### Connection Handshake
```
GET /v1/stream HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
Authorization: Bearer <api_key>
```

#### Protocol Frames
1. **Client sends configuration frame (JSON)**:
```json
{
  "type": "config",
  "model": "edge-tts",
  "voice": "en-US-AriaNeural",
  "sample_rate": 24000,
  "format": "pcm_s16le"
}
```
2. **Client streams text tokens (JSON)**:
```json
{
  "type": "text_chunk",
  "text": "Hello world, streaming audio in real-time."
}
```
3. **Server sends audio chunks (Binary)**:
Raw binary PCM/WAV byte frames delivered within milliseconds of phoneme synthesis.
4. **Client signals end of turn**:
```json
{
  "type": "flush"
}
```

---

## 6. Health & Telemetry

### `GET /health`
Returns quick liveness status:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3840
}
```

### `GET /health/ready`
Inspects local engine readiness and database connectivity:
```json
{
  "status": "ready",
  "hardware_tier": "TIER_4_ENTERPRISE",
  "database": "connected",
  "engines_ready": ["edge-tts", "kokoro-82m", "piper"]
}
```

### `GET /metrics`
Standard Prometheus metrics output (RTF, TTFB latency histograms, active connections, VRAM load).

---

## 7. Error Format (RFC 7807)

All non-2xx responses conform to RFC 7807 Problem Details:

```json
{
  "type": "https://voxforg.org/errors/invalid-engine-parameters",
  "title": "Invalid Engine Parameters",
  "status": 400,
  "detail": "Voice 'en_US-unknown' is not registered with engine 'kokoro-82m'.",
  "instance": "/v1/audio/speech",
  "timestamp": "2026-09-05T10:25:00Z"
}
```
