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
| `input` | `string` | Yes | - | Raw text or SSML snippet (Max length: 50,000 characters). Text is pre-processed through the pronunciation normalizer. |
| `voice` | `string` | Yes | - | Voice identifier (e.g. `en-US-AriaNeural`, `en_US-heart`). |
| `response_format`| `string` | No | `wav` | Audio container: `wav`, `mp3`, `opus`, `aac`, `flac`, `pcm`. |
| `speed` | `number` | No | `1.0` | Speaking rate multiplier between `0.25` and `4.0`. |
| `pitch` | `number` | No | `0.0` | Pitch shift in semitones between `-12.0` and `+12.0` (Engine-dependent). |
| `loudness_lufs` | `number` | No | `null`| Target EBU R128 integrated loudness (e.g., `-14.0` LUFS for podcasting). |
| `policy` | `object` | No | `null`| SLA routing policy. When set, `VoiceRouter` auto-selects the best engine. See §9. |

#### Response
- **Status:** `200 OK`
- **Content-Type:** `audio/wav` or `audio/pcm` (when `response_format: "pcm"`)
- **Headers:** `x-request-id`, `cache-control: no-cache`

---

### `POST /v1/audio/speech/stream`

Streams audio chunks in real-time using HTTP Chunked Transfer Encoding. Cuts time-to-first-byte (TTFB) down to `< 50ms`.

#### Request Body
Same JSON payload as `POST /v1/audio/speech`.

#### Response
- **Status:** `200 OK`
- **Content-Type:** `audio/pcm`
- **Transfer-Encoding:** `chunked`
- **Body:** Continuous binary stream of raw 16-bit little-endian PCM sample slices as emitted by synthesis engine.

---

### `GET /v1/audio/speech/ws`

Full-duplex real-time WebSocket speech synthesis streaming endpoint.

#### Protocol Flow:
1. **Client connects**: `ws://localhost:8080/v1/audio/speech/ws`
2. **Client sends JSON request frame**:
```json
{
  "model": "edge-tts",
  "voice": "en-US-AriaNeural",
  "input": "Streaming low-latency audio via WebSocket connection.",
  "speed": 1.0
}
```
3. **Server streams binary audio frames**:
   - Multiple binary WebSocket messages containing 16-bit PCM audio chunks.
4. **Server sends completion message**:
```json
{
  "event": "done",
  "total_samples": 84960
}
```

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
Standard Prometheus text format (`text/plain; version=0.0.4; charset=utf-8`) output for Grafana, VictoriaMetrics, and Prometheus scrapers:

```prometheus
# HELP voxforg_active_requests Currently in-flight HTTP requests
# TYPE voxforg_active_requests gauge
voxforg_active_requests 0

# HELP voxforg_requests_total Total number of HTTP requests processed by endpoint and status
# TYPE voxforg_requests_total counter
voxforg_requests_total{endpoint="/v1/audio/speech",status="200"} 24

# HELP voxforg_synthesis_total Total number of completed speech syntheses
# TYPE voxforg_synthesis_total counter
voxforg_synthesis_total 18

# HELP voxforg_synthesis_duration_seconds_total Total duration in seconds spent synthesizing audio
# TYPE voxforg_synthesis_duration_seconds_total counter
voxforg_synthesis_duration_seconds_total 1.2940

# HELP voxforg_audio_samples_total Total 16-bit PCM audio samples generated
# TYPE voxforg_audio_samples_total counter
voxforg_audio_samples_total 169920

# HELP voxforg_cache_hits_total In-memory audio synthesis LRU cache hits
# TYPE voxforg_cache_hits_total counter
voxforg_cache_hits_total 6

# HELP voxforg_cache_misses_total In-memory audio synthesis LRU cache misses
# TYPE voxforg_cache_misses_total counter
voxforg_cache_misses_total 18
```

---

## 7. Headers & Error Format (RFC 7807)

### Distributed Tracing Header
- `x-request-id`: Clients may supply a correlation UUID or string; if omitted, the server generates a UUIDv4 and returns it in the response headers.

### Request Body & Payload Limits
- Maximum Request Body Size: `2MB` (`DefaultBodyLimit::max(2 * 1024 * 1024)`).
- Maximum Text Input Length: `10,000` characters per request. Requests exceeding 10k characters receive HTTP 422 Unprocessable Entity.

All non-2xx responses conform to RFC 7807 Problem Details:

```json
{
  "type": "https://voxforg.org/errors/input-too-large",
  "title": "Input Text Exceeds Limit",
  "status": 422,
  "detail": "The 'input' parameter cannot exceed 10,000 characters per request",
  "instance": "/v1/audio/speech"
}
```

---

## 8. Automated QA & Voice A/B Testing

### `POST /v1/qa/ab-test`

Executes multi-run statistical and acoustic quality benchmarks comparing two engine/voice/parameter variants side-by-side. Evaluates real-time factor (RTF), latency, dynamic range, RMS loudness, peak amplitude, and digital clipping samples.

#### Request Body
```json
{
  "scenario_name": "Studio vs Field Comparison",
  "text": "The atmospheric density on Kepler-452b enables acoustic wave amplification.",
  "variant_a_engine": "edge-tts",
  "variant_a_voice": "en-US-AriaNeural",
  "variant_a_speed": 1.0,
  "variant_b_engine": "mock-tts",
  "variant_b_voice": "mock-en-female",
  "variant_b_speed": 1.0,
  "iterations": 3
}
```

#### Parameters
| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `scenario_name` | `string` | No | `"A/B Voice Comparison"` | Human-readable scenario name |
| `text` | `string` | Yes | - | Prompt text synthesized across both variants |
| `variant_a_engine` | `string` | Yes | - | Engine ID for Variant A |
| `variant_a_voice` | `string` | Yes | - | Voice ID for Variant A |
| `variant_a_speed` | `number` | No | `1.0` | Playback speed multiplier for Variant A |
| `variant_b_engine` | `string` | Yes | - | Engine ID for Variant B |
| `variant_b_voice` | `string` | Yes | - | Voice ID for Variant B |
| `variant_b_speed` | `number` | No | `1.0` | Playback speed multiplier for Variant B |
| `iterations` | `number` | No | `3` | Number of benchmark runs averaged (clamped 1-10) |

#### Response (`200 OK`)
```json
{
  "scenario": "Studio vs Field Comparison",
  "variant_a": {
    "engine_id": "edge-tts",
    "voice_id": "en-US-AriaNeural",
    "speed": 1.0,
    "latency_ms": 142.5,
    "rtf": 0.08,
    "metrics": {
      "duration_seconds": 1.82,
      "sample_rate_hz": 24000,
      "channels": 1,
      "peak_amplitude": 0.76,
      "peak_dbfs": -2.38,
      "rms_dbfs": -16.42,
      "clipping_samples": 0
    },
    "audio_base64": "<base64_encoded_audio>"
  },
  "variant_b": {
    "engine_id": "mock-tts",
    "voice_id": "mock-en-female",
    "speed": 1.0,
    "latency_ms": 1.8,
    "rtf": 0.001,
    "metrics": {
      "duration_seconds": 1.80,
      "sample_rate_hz": 24000,
      "channels": 1,
      "peak_amplitude": 0.50,
      "peak_dbfs": -6.02,
      "rms_dbfs": -9.03,
      "clipping_samples": 0
    },
    "audio_base64": "<base64_encoded_audio>"
  },
  "latency_delta_ms": 140.7,
  "rtf_delta": 0.079,
  "loudness_delta_db": -7.39,
  "latency_winner": "Variant B",
  "loudness_winner": "Variant A",
  "recommended_variant": "Variant B"
}
```

---

## 9. SLA Policy-Based Routing (Phase 2A)

The `POST /v1/audio/speech` endpoint accepts an optional `policy` field.
When present, `VoiceRouter` selects the best-scoring registered engine automatically.

### Policy Object

```json
{
  "model": "auto",
  "input": "Quarterly earnings rose 18%.",
  "voice": "alloy",
  "policy": {
    "quality_min": 0.8,
    "latency_max_ms": 500,
    "cost_max_per_1k": 0.0,
    "language": "en-US",
    "style": "conversational",
    "fallback_chain": ["edge-tts", "mock-tts"]
  }
}
```

#### Policy Fields
| Field | Type | Description |
|---|---|---|
| `quality_min` | `number` | Minimum quality score (0.0–1.0). Engines below this are rejected. |
| `latency_max_ms` | `integer` | Maximum acceptable latency ceiling in milliseconds. |
| `cost_max_per_1k` | `number` | Cost ceiling in USD per 1,000 characters. Set `0.0` for free/local only. |
| `language` | `string` | BCP-47 language tag preference (e.g. `"en-US"`, `"hi-IN"`). |
| `style` | `string` | Voice style hint: `any`, `conversational`, `narration`, `news`, `assistant`. |
| `fallback_chain` | `string[]` | Ordered engine IDs to consider. Empty = auto-select from all registered. |

When no engine satisfies the policy, the endpoint returns `422 Unprocessable Entity`.

---

## 10. Pronunciation Dictionary (Phase 2C)

### `GET /v1/pronunciation/dictionary`

Returns all active pronunciation overrides.

**Response:**
```json
{
  "count": 2,
  "entries": [
    { "term": "SQL",  "replacement": "sequel", "note": "database query language" },
    { "term": "nginx", "replacement": "engine X" }
  ]
}
```

### `POST /v1/pronunciation/dictionary`

Add or update a pronunciation entry. Returns `201 Created` on insert, `200 OK` on update.

```json
{ "term": "VoxForg", "replacement": "Vox Forge", "note": "brand name" }
```

### `DELETE /v1/pronunciation/dictionary/{term}`

Remove a pronunciation entry. Returns `204 No Content`, or `404` if not found.

---

## 11. Engine Benchmarking (Phase 3A)

### `POST /v1/benchmark/run`

Triggers an asynchronous benchmark across all registered engines. Returns `202 Accepted` immediately. Results become available via `GET /v1/benchmark/results` when complete.

### `GET /v1/benchmark/results`

Returns stored benchmark scorecards, sorted by overall score descending.

```json
{
  "count": 1,
  "results": [
    {
      "engine_id": "edge-tts",
      "sentences_ok": 10,
      "sentences_failed": 0,
      "latency": { "p50_ms": 312, "p95_ms": 480, "max_ms": 620 },
      "throughput_sps": 2.4,
      "overall_score": 0.88,
      "run_at": "2026-09-11T08:00:00Z"
    }
  ]
}
```

---

## 12. Voice CI/CD Regression (Phase 3B)

### `GET /v1/voice-ci/profiles`

List all captured voice quality profiles.

### `POST /v1/voice-ci/profiles`

Capture a voice quality baseline by running the benchmark sentence suite against a specific engine/voice pair.

```json
{ "engine_id": "edge-tts", "voice_id": "en-US-AriaNeural" }
```

Returns `201 Created` with the stored `VoiceProfile` or `200 OK` on update.

### `POST /v1/voice-ci/compare`

Run the benchmark suite against a stored profile and report regressions.

```json
{
  "profile_id": "edge-tts:en-US-AriaNeural",
  "latency_threshold_pct": 20.0,
  "success_threshold": 1.0
}
```

**Response:**
```json
{
  "profile_id": "edge-tts:en-US-AriaNeural",
  "engine_id": "edge-tts",
  "voice_id": "en-US-AriaNeural",
  "passed": true,
  "current_latency_ms": 320,
  "baseline_latency_ms": 310,
  "latency_delta_pct": 3.2,
  "current_success_rate": 1.0,
  "baseline_success_rate": 1.0,
  "run_at": "2026-09-11T09:00:00Z"
}
```

`passed: false` is returned when latency increased beyond `latency_threshold_pct`
or `current_success_rate < success_threshold`.

---

## Endpoint Summary

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/audio/speech` | Synthesize speech (supports `policy` field) |
| POST | `/v1/audio/speech/stream` | Streaming speech synthesis (chunked) |
| GET  | `/v1/audio/speech/ws` | WebSocket real-time streaming |
| GET  | `/v1/voices` | List all voices from all engines |
| GET  | `/v1/models` | List all registered engines |
| POST | `/v1/pipeline/execute` | Execute a DAG synthesis pipeline |
| POST | `/v1/qa/ab-test` | A/B compare two synthesis variants |
| GET  | `/v1/pronunciation/dictionary` | List pronunciation overrides |
| POST | `/v1/pronunciation/dictionary` | Add/update a pronunciation entry |
| DELETE | `/v1/pronunciation/dictionary/{term}` | Remove a pronunciation entry |
| POST | `/v1/benchmark/run` | Trigger async engine benchmark |
| GET  | `/v1/benchmark/results` | Get benchmark scorecards |
| GET  | `/v1/voice-ci/profiles` | List voice quality profiles |
| POST | `/v1/voice-ci/profiles` | Capture voice quality baseline |
| POST | `/v1/voice-ci/compare` | Run regression against baseline |
| GET  | `/health` | Liveness check |
| GET  | `/health/ready` | Readiness check (deep probing) |
| GET  | `/metrics` | Prometheus-style metrics |
| GET  | `/docs` | Scalar API documentation UI |
| GET  | `/openapi.json` | OpenAPI 3.1 spec |
