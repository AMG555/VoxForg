# ADR-004: Hardware-Adaptive Engine Scheduling

## Status
Accepted

## Date
2026-09-05

## Context
TTS engines exhibit vastly different compute profiles. Kokoro-82M and Piper run efficiently on CPU with AVX2/NEON SIMD instructions, whereas heavier models like Qwen3-TTS or StyleTTS2 require dedicated GPU accelerators (CUDA/MPS) with >= 8GB VRAM. If a user runs VoxForg on a modest laptop without a discrete GPU, loading an enterprise model causes out-of-memory errors or system freezing.

## Decision
Implement an automated hardware probe (`voxforg-hardware`) that analyzes host resources at startup:
- CPU SIMD flags (AVX2, AVX-512, ARM NEON).
- Total system RAM and available headroom.
- GPU presence, compute capability, and VRAM (via CUDA runtime / DirectML / Metal).

Assign the system to a `HardwareTier` (Tier 1: Minimal to Tier 4: Enterprise) and dynamically fallback to cloud relay (Edge-TTS) or optimized ONNX models (Piper) if requested engines exceed host hardware limits.

## Consequences
- Guarantees zero-crash startup even on resource-constrained environments (e.g. Raspberry Pi 5 or low-spec cloud VMs).
- Users can override recommendations via `--force-engine` flag in CLI or settings.
