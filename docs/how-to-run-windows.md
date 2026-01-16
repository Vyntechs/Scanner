# Running on Windows 10/11

## Prerequisites

- Node.js 18+
- Rust (stable) + MSVC build tools
- Tauri prerequisites (WebView2 + Visual Studio Build Tools)
- vLinker FS J2534 driver installed

## Install dependencies

```bash
npm install
```

## Run (development)

```bash
npm run tauri:dev
```

## Build (production)

```bash
npm run tauri:build
```

## Notes

- For live scanning, ensure the vLinker FS J2534 DLL is discoverable (see `docs/dev-notes.md`).
- Simulation mode uses `samples/f250_session.json` by default.
