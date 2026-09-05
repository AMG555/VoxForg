# ADR-005: Desktop Packaging via Tauri v2

## Status
Accepted

## Date
2026-09-05

## Context
A substantial segment of creators, podcasters, and voice artists prefer a native desktop application over running command-line daemons or browser tabs. The desktop client needs to operate with native audio sinks, direct local file-system drag-and-drop, and minimal RAM footprint.

## Decision
Use **Tauri v2** to bundle the React 19 frontend with the VoxForg Rust core.
- The Tauri application runs the embedded VoxForg core daemon in-process or as a managed sidecar.
- UI renders inside the OS native webview (WebView2 on Windows, WebKit on macOS, WebKitGTK on Linux).
- Packages cross-platform installers: `.msi` (Windows), `.dmg` (macOS Universal), `.AppImage` / `.deb` (Linux).

## Alternatives Considered

### Electron
- **Pros:** Mature ecosystem.
- **Cons:** Ships entire Chromium runtime and Node.js; installer sizes > 120MB; consumes > 300MB idle RAM before loading audio buffers.
- **Rejected:** Incompatible with our lean, high-performance architectural standard.

## Consequences
- Desktop bundle size is under 25MB.
- Idle memory footprint is under 40MB.
- Single unified Rust codebase bridges both the server daemon and the desktop application.
