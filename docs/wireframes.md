# VoxForg UI/UX Design System & Wireframes

**Design Direction:** High-density, dark engineering workstation.  
**Inspiration:** Ableton Live meets n8n — responsive, tactile, high contrast, zero unnecessary fluff.  
**Target Breakpoints:** Desktop primary (`1440px` and `1920px`), Laptop (`1024px`), Tablet minimum (`768px`).

---

## 1. Design System Tokens

### 1.1 Color Palette

```
Surface Neutral:
  --bg-app:        #0B0E14   (Deep Obsidian canvas background)
  --bg-surface:    #121820   (Card and panel backgrounds)
  --bg-elevated:   #1A222D   (Dropdowns, tooltips, modals)
  --border-subtle: #242E3D   (Separators, canvas grid lines)
  --border-bold:   #3B485C   (Node borders, focused inputs)

Typography:
  --text-primary:   #F0F4F8  (High contrast 15.2:1 AA / AAA compliant)
  --text-secondary: #94A3B8  (Metadata, helper text, timestamps)
  --text-muted:     #64748B  (Placeholders, disabled labels)

Brand Accent & Semantics:
  --accent-amber:   #F59E0B  (Primary active state, audio playback)
  --accent-cyan:    #38BDF8  (Connection edges, pipeline flow)
  --status-success: #10B981  (Engine ready, test passed)
  --status-warning: #FBBF24  (High VRAM utilization, fallback active)
  --status-error:   #EF4444  (Engine crash, validation failure)
```

### 1.2 Spacing & Typography

- **Font Family:** `JetBrains Mono` for numbers/telemetry/nodes; `Inter` for interface prose.
- **Type Scale:** 11px (micro tags), 13px (compact node labels), 14px (body/inputs), 16px (subheadings), 20px (panel titles), 28px (view headers).
- **Grid Spacing:** Strict 4px base (`4px`, `8px`, `12px`, `16px`, `24px`, `32px`).

---

## 2. Screen 1: Pipeline Canvas (Visual Node DAG Builder)

The central workstation screen where users build complex multi-speaker, chunked audio generation graphs.

### ASCII Wireframe

```
+---------------------------------------------------------------------------------------------------------------+
| [VOXFORG]  File  Edit  View  Pipeline: [ Audiobook Narrative v2.4 (Saved) v ]     [Test Run]  [Export Master] |
+---------------------------------------------------------------------------------------------------------------+
| TOOLBOX      | CANVAS (Infinite pan & zoom, 16px dot grid)                               | NODE INSPECTOR     |
|              |                                                                           |                    |
| > Search...  |  +-------------------+        +---------------------+                     | [x] Voice Assigner |
|              |  | [1] Text Input    |        | [2] Speaker Parser  |                     |                    |
| + Ingestion  |  | "Chapter 1: The..."|------->| Detects: Alice, Bob |                     | Target Speaker:    |
|   - Raw Text |  +-------------------+        +----------+----------+                     | [ Alice        v ] |
|   - Chunker  |                                          |                                |                    |
|   - SSML     |                   +----------------------+----------------------+         | Assigned Engine:   |
|              |                   |                                             |         | [ Kokoro-82M   v ] |
| + Voice      |                   v                                             v         |                    |
|   - Assigner |       +-----------------------+                     +-------------------+ | Voice Profile:     |
|   - Mixer    |       | [3] Voice: Alice      |                     | [4] Voice: Bob    | | [ en_US_Alice  v ] |
|   - Cloner   |       | Model: Kokoro-82M     |                     | Model: Piper-v1   | |                    |
|              |       +-----------+-----------+                     +---------+---------+ | Parameters:        |
| + Processing |                   |                                           |           | Speed:  [--o---]1.0|
|   - Normalizer                   v                                           v           | Pitch:  [---o--]0.0|
|   - Filter   |       +-----------------------+                     +-------------------+ | Volume: [-o----]0dB|
|   - Ducking  |       | [5] Synthesize Chunk  |                     | [6] Synthesize    | |                    |
|              |       | Est: 120ms (CUDA)     |                     | Est: 85ms (AVX2)  | | [Preview Voice]    |
| + Output     |       +-----------+-----------+                     +---------+---------+ |                    |
|   - WAV File |                   |                                           |           | Audio Output:      |
|   - Stream   |                   +--------------------+----------------------+           | Format: WAV 24kHz  |
|              |                                        v                                  |                    |
|              |                           +--------------------------+                    | Validation:        |
|              |                           | [7] Cross-Fade Merge     |                    | (o) Ready to run   |
|              |                           | Overlap: 150ms | -14 LUFS|                    |                    |
|              |                           +------------+-------------+                    |                    |
|              |                                        |                                  |                    |
|              |                                        v                                  |                    |
|              |                           +--------------------------+                    |                    |
|              |                           | [8] Output Audio Sink    |                    |                    |
|              |                           | Output: /out/chapter1.wav|                    |                    |
|              |                           +--------------------------+                    |                    |
+---------------------------------------------------------------------------------------------------------------+
| TIMELINE / LOGS: [Play] [00:00.00 / 02:45.12]   ||||||||||||||||||||||||......           | System: GPU 42%    |
+---------------------------------------------------------------------------------------------------------------+
```

### Component Breakdown
1. **Left Sidebar (Node Palette)**: Grouped accordions of draggable node blocks (Ingestion, Voice, Audio Modifiers, Output Sinks).
2. **Center Canvas**: WebGL/SVG rendered infinite canvas with snap-to-grid (`16px`), curved Bézier spline connectors, and execution state halos (idle: gray, executing: pulsing cyan, completed: emerald, failed: ruby).
3. **Right Drawer (Node Inspector)**: Contextual properties panel reflecting selected node with dynamic validation and inline audio preview.
4. **Bottom Dock (Playback & Telemetry)**: Synchronized multi-track waveform player and system load gauges.

---

## 3. Screen 2: Voice Lab & Model Manager

Interactive sandbox for testing voice engines, tuning phonetic pronunciations, and downloading offline ONNX models.

### ASCII Wireframe

```
+---------------------------------------------------------------------------------------------------------------+
| [VOXFORG]  Dashboard   Pipelines   [Voice Lab]   Batch Queue   Engines   Settings                 v1.0.0-stable |
+---------------------------------------------------------------------------------------------------------------+
| INSTALLED ENGINES & VOICES (42 Available)   | VOICE SYNTHESIS PLAYGROUND                                      |
|                                             |                                                                 |
| [Search voices, languages, tags...      ]   | Test Prompt:                                                    |
|                                             | +-------------------------------------------------------------+ |
| + LOCAL ENGINES                             | | The atmospheric density on Kepler-452b allows acoustic      | |
|   v Kokoro-82M (GPU Accelerated - CUDA)     | | waves to travel 1.4 times faster than standard Earth normal.| |
|     * en_US-heart (Warm / Narration)        | +-------------------------------------------------------------+ |
|     * en_GB-george (Authoritative)          |                                                                 |
|     * ja_JP-sakura (Expressive)             | Selected Voice: en_US-heart (Kokoro-82M)                        |
|                                             | Model Source: Local ONNX (weights/kokoro-v0.82.onnx)            |
|   v Piper (CPU Optimized - AVX2)            |                                                                 |
|     * en_US-ryan-medium (Conversational)    | Dynamic Controls:                                               |
|     * de_DE-thorsten (Broadcast)            | Speaking Rate:  0.5x  [-------o-------] 2.0x  (1.05x)           |
|                                             | Pitch Shift:   -12st  [-------o-------] +12st (0.0st)           |
| + CLOUD ENGINES                             | Energy Level:    0%   [---------o-----] 100%  (72%)             |
|   v Microsoft Edge TTS (Cloud Relay)        | Phoneme Tuning: [ Auto (IPA) v ]                                |
|     * en-US-AriaNeural                      |                                                                 |
|     * en-US-ChristopherNeural               | [ Generate Preview (Space) ]       Latency: 38ms (TTFB: 14ms)   |
|                                             |                                                                 |
| + [ + Download New Engine / Voice Pack ]    | WAVEFORM & PHONEME SPECTROGRAM:                                 |
|                                             | +-------------------------------------------------------------+ |
|                                             | |  /\    ||/\    /\/\    |  /\  /\/\    ||/\  /\                | |
|                                             | | /  \  /||  \  /    \  /| /  \/    \  /||  \/  \               | |
|                                             | |/    \/ ||   \/      \/ |/    \     \/ ||   \   \  [00:04.28]  | |
|                                             | +-------------------------------------------------------------+ |
|                                             | Phonemes: [dh][iy] [ae][t][m][ah][s][f][ih][r][ih][k] ...       |
|                                             | [Play] [Pause] [Loop] [Download .WAV] [Export to Pipeline]      |
+---------------------------------------------------------------------------------------------------------------+
```

---

## 4. Screen 3: Batch Transcribe & Production Queue

Bulk conversion dashboard for podcasts, audiobooks, and bulk API job monitoring.

### ASCII Wireframe

```
+---------------------------------------------------------------------------------------------------------------+
| [VOXFORG]  Dashboard   Pipelines   Voice Lab   [Batch Queue]   Engines   Settings                 v1.0.0-stable |
+---------------------------------------------------------------------------------------------------------------+
| QUEUE OVERVIEW: Active Jobs: 3 | Queued: 14 | Completed: 1,842 | Throughput: 14.8x Realtime | Workers: 8/8    |
+---------------------------------------------------------------------------------------------------------------+
| [ + Submit New Batch ] [ Pause Queue ] [ Clear Completed ]                       Filter: [ All Statuses v ]   |
|                                                                                                               |
| ID      NAME / SOURCE               ENGINE         PROGRESS        WORDS    ELAPSED   EST. REMAIN   STATUS    |
| ------------------------------------------------------------------------------------------------------------- |
| #4091   Dune_Chapter_04.txt         Kokoro-82M     [========>  ] 78% 12,400   01m 14s   00m 22s       SYNTH     |
| #4092   Tech_Podcast_Ep42.json      Multi-Pipeline [====>      ] 42% 18,200   02m 40s   03m 15s       PARSING   |
| #4093   Customer_IVR_Prompts.csv    Piper (CPU)    [>          ]  9%    850   00m 08s   01m 10s       QUEUED    |
| #4090   Release_Notes_Audio.md      Edge-TTS       [===========]100%  3,100   00m 18s   --            DONE      |
| #4089   Medical_Glossary_A.txt      Kokoro-82M     [===========]100% 45,000   04m 02s   --            DONE      |
|                                                                                                               |
| WORKER UTILIZATION:                                                                                           |
| Worker 1 [CUDA-0]: Kokoro-82M  (Batch #4091 - Chunk 84/110) [████████████████░░░░] 82%                      |
| Worker 2 [CUDA-0]: Kokoro-82M  (Batch #4092 - Chunk 12/48)  [██████████████░░░░░░] 70%                      |
| Worker 3 [AVX2]:   Piper (CPU) (Batch #4093 - Chunk 2/15)   [██████████░░░░░░░░░░] 50%                      |
| Worker 4 [IO]:     Disk Writer (Flushing chapter4_master.wav)[████████████████████] 100%                     |
+---------------------------------------------------------------------------------------------------------------+
```

---

## 5. Screen 4: Hardware Profiler & Engine Diagnostics

Detailed hardware telemetry panel with automated capability benchmark.

### ASCII Wireframe

```
+---------------------------------------------------------------------------------------------------------------+
| [VOXFORG]  Dashboard   Pipelines   Voice Lab   Batch Queue   [Engines & Hardware]   Settings      v1.0.0-stable |
+---------------------------------------------------------------------------------------------------------------+
| HOST TELEMETRY                                   | ENGINE CAPABILITY MATRIX                                   |
|                                                  |                                                            |
| CPU Architecture: x86_64                         | Engine       Type   Backend     Latency    Status          |
| Processor: AMD Ryzen 9 5900X (12C / 24T)         | ---------------------------------------------------------- |
| SIMD Extensions:                                 | Edge-TTS     Cloud  WSS Relay   110ms      [ ONLINE ]      |
|   [x] AVX    [x] AVX2    [ ] AVX-512   [ ] NEON  | Kokoro-82M   Local  CUDA / ONNX  32ms      [ ACTIVE/GPU ]  |
| Memory: 32,768 MB Total | 24,120 MB Available    | Piper-TTS    Local  CPU / AVX2   65ms      [ ACTIVE/CPU ]  |
|                                                  | KittenTTS    Local  DirectML    48ms      [ STANDBY ]     |
| GPU Accelerators:                                | Qwen3-TTS    Local  CUDA         145ms     [ NOT LOADED ]  |
|   Device 0: NVIDIA GeForce RTX 3080              |                                                            |
|   VRAM: 10,240 MB Total | 7,810 MB Free          | [ Run Automated Hardware Benchmark ]                       |
|   Compute Driver: CUDA 12.4 (Driver 555.42)      | Benchmark Results:                                         |
|                                                  |   - Realtime Factor (RTF): 0.042 (23.8x faster than audio) |
| Assigned Hardware Tier: TIER_4_ENTERPRISE        |   - Peak Memory Bandwidth: 760 GB/s                        |
| Recommended Default: Kokoro-82M (CUDA)           |   - Max Concurrent Local Streams: 18 streams               |
+---------------------------------------------------------------------------------------------------------------+
```

---

## 6. Interaction & Accessibility Standards

1. **Keyboard-First Workstation**:
   - `Space`: Toggle audio playback in previewer.
   - `Ctrl/Cmd + S`: Save pipeline DAG.
   - `Ctrl/Cmd + Enter`: Run current pipeline.
   - `Delete / Backspace`: Remove selected node or wire connector.
   - `Tab`: Cycle through node input parameters sequentially.
2. **WCAG 2.1 AA Compliance**:
   - Contrast ratio exceeds `4.5:1` across all active UI text.
   - Distinctive border highlights accompany color state changes (never rely on color alone).
   - ARIA live regions announce batch completion and background render errors.

---

## 7. Real-Time Web Audio Visualizer & Pipeline Presets

### 7.1 Real-Time Audio Telemetry Visualizer

Embedded in Voice Lab and Canvas Bottom Dock during playback:

```
+---------------------------------------------------------------------------------+
| [*] REAL-TIME AUDIO TELEMETRY                       [ FFT Spectrum ] [ Waveform ]|
| +-----------------------------------------------------------------------------+ |
| |  |                                                                          | |
| |  ||   ||   |||                                                              | |
| |  ||| |||| |||||  ||   |                                                     | |
| | |||||||||||||||||||| |||  ||   |                                            | |
| | ||||||||||||||||||||||||||||||||| | |   |                                   | |
| |+---------------------------------------------------------------------------+| |
|  20Hz                           1kHz                             20kHz (24kHz)  |
+---------------------------------------------------------------------------------+
```

### 7.2 Pipeline Presets & Import / Export Controls

Located in the central Canvas toolbar:

```
+---------------------------------------------------------------------------------------------------------------+
| Pipeline: [ Narrative Dialogue v ] [Preset: Studio Podcast v] [Export JSON] [Import JSON] [Execute Pipeline] |
+---------------------------------------------------------------------------------------------------------------+
| Presets Available:                                                                                            |
|   1. Narrative Dialogue: Multi-speaker script -> Parser -> Voice Allocator -> Synth -> Crossfade Merger      |
|   2. Studio Podcast: Voice Synthesis -> 3-Band Parametric EQ -> Dynamic Compressor -> Limiter Mastering     |
|   3. Punchy Radio: Aggressive EQ mid-boost -> Heavy dynamic compression -> Brickwall Limiter                 |
+---------------------------------------------------------------------------------------------------------------+
```

