# ADR-001: Use Rust for Core Synthesis & Audio Engine

## Status
Accepted

## Date
2026-09-05

## Context
VoxForg requires high-throughput, low-latency audio manipulation, streaming chunk delivery, and multi-engine process orchestration. Key technical constraints:
- Real-time factor (RTF) must remain below 0.1 for interactive speech synthesis.
- Zero-garbage-collection pauses during live audio streaming to prevent acoustic pops and buffer underruns.
- Direct hardware probing (x86 SIMD, ARM NEON, CUDA C bindings, ONNX Runtime native bindings).
- Safe cross-platform compilation across Linux, macOS, and Windows targets.

## Decision
Implement the entire core daemon, audio DSP pipeline, and engine abstraction layer in **Rust** (2021 edition), leveraging Tokio for asynchronous I/O and Rayon for CPU-bound audio DSP tasks.

## Alternatives Considered

### Python (FastAPI + PyTorch)
- **Pros:** Native home for machine learning researchers, fast prototyping.
- **Cons:** Global Interpreter Lock (GIL) stalls concurrent audio streams; high memory footprint (>1.5 GB idle); packaging single binaries across OSes is brittle (PyInstaller issues).
- **Rejected:** Incompatible with single-binary and lightweight desktop requirements.

### Go (Golang)
- **Pros:** Fast compilation, straightforward concurrency model with goroutines.
- **Cons:** Lacks zero-cost abstractions for SIMD audio DSP; CGo overhead when binding to ONNX Runtime / CUDA causes context-switch penalties; garbage collector causes jitter in 48kHz audio streams.
- **Rejected:** Rust provides superior memory control and zero-overhead native bindings.

## Consequences
- Single statically linked release binary with sub-30ms startup time.
- Compile-time safety guarantees for multi-threaded audio ring buffers.
- Requires team proficiency in Rust memory model, lifetimes, and async traits.
