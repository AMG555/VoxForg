# ADR-003: Axum 0.8 HTTP Framework & OpenAI API Compatibility

## Status
Accepted

## Date
2026-09-05

## Context
Developers already have extensive tooling built around the OpenAI Audio API (`/v1/audio/speech`). Providing seamless compatibility allows instant integration with existing AI agents, LLM pipelines, and automated creative workflows. Additionally, the server requires high-concurrency WebSocket support for low-latency streaming.

## Decision
1. Use **Axum 0.8** (Tokio / Tower ecosystem) for the HTTP/WS server.
2. Provide strict drop-in compatibility for `POST /v1/audio/speech` and `GET /v1/models`.
3. Support audio response formats: `wav`, `mp3`, `opus`, `pcm`, `flac`.

## Alternatives Considered

### Actix-Web
- **Pros:** Fast benchmark scores.
- **Cons:** More complex actor-based mental model; less ergonomic integration with modern Tower middleware ecosystem compared to Axum.
- **Rejected:** Axum has direct first-class Tokio alignment and superior middleware ergonomics.

## Consequences
- Existing scripts written for OpenAI Audio API work with VoxForg simply by changing `base_url`.
- Built-in Tower middleware enables fine-grained rate limiting, CORS controls, and security headers out of the box.
