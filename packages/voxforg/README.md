# voxforg

Unified hardware-adaptive speech synthesis workstation, voice cloning studio, and DAG audio pipeline runner.

## Zero-Install Quick Start

Run the entire visual workstation and API server instantly via `npx`:

```bash
npx voxforg
```

This automatically:
1. Detects your operating system and CPU architecture.
2. Downloads the matching native binary with embedded workstation UI.
3. Launches the local daemon on `http://localhost:8080`.
4. Opens your default web browser directly into the visual workstation.

## CLI Usage

Forward any VoxForg CLI command directly through the runner:

```bash
# Direct speech synthesis to WAV file
npx voxforg synth "Acoustic neural synthesis operational." -o hello.wav

# System compute & hardware acceleration probe
npx voxforg hardware

# Speech-to-text transcription with subtitle generation
npx voxforg transcribe lecture.wav --format srt --out subtitles.srt

# Zero-shot voice cloning
npx voxforg clone --name "Nova" --audio sample.wav --out nova.json

# Local model inventory
npx voxforg models list
```

## License

Apache-2.0 OR MIT
