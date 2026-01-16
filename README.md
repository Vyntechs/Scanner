# Scanner

Scanner is a Windows desktop automotive diagnostic scan tool MVP built for a clean, professional workflow.
It connects to a vehicle through a vLinker FS USB adapter, reads VINs, discovers modules, builds a topology view,
reads/clears DTCs, and logs transport/protocol activity.

## Supported OS

- Windows 10/11

## Hardware

- vLinker FS USB (J2534)

## How to run

```bash
npm install
npm run tauri:dev
```

## Simulation vs Live Adapter

- Simulation mode replays a captured session from `samples/` so you can test the UI without a vehicle.
- Live Adapter mode connects to the vLinker FS over J2534 and performs real VIN/module/DTC operations.

## J2534 DLL requirement

The app looks for the J2534 DLL in these locations, in order:

1. `J2534_DLL` environment variable
2. `C:\Program Files (x86)\vLinker\J2534.dll`
3. `C:\Program Files\vLinker\J2534.dll`
4. `C:\Windows\System32\J2534.dll`

If the DLL is missing, the UI reports a driver error with guidance.
