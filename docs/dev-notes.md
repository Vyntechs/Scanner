# Developer Notes

## Extending discovery

- Add OEM-specific module candidates in `src-tauri/src/discovery/mod.rs`.
- Use logical names where possible; fallback naming is `0x7E0` style for unknown ECUs.

## J2534 DLL lookup

The app resolves the J2534 DLL in this order:

1. `J2534_DLL` environment variable.
2. `C:\Program Files (x86)\vLinker\J2534.dll`
3. `C:\Program Files\vLinker\J2534.dll`
4. `C:\Windows\System32\J2534.dll`

If none are found, the UI shows a calm error with the missing-driver hint.

## Logging format

Logs are JSONL. Each line includes:

- `timestamp` (UTC)
- `level` (info/warn/error)
- `kind` (transport/protocol/system)
- `message`
- `payload` (structured JSON)

## Simulation sessions

Simulation JSON schema lives in `src-tauri/src/simulation/mod.rs`.

### Recording a new simulation session

1. Run a live scan and export logs.
2. Capture VIN, module list, and per-module DTCs.
3. Create a new JSON file under `/samples` using the same schema as `samples/f250_session.json`.
4. Update the UI connect screen or pass `simulation_path` to `start_scan` to use the new file.
