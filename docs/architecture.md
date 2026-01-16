# Architecture

## Layering

- `transport/`: Hardware I/O abstraction (`Transport` trait). MVP includes `VLinkerFsJ2534Transport` (Windows J2534). `SimTransport` is used for simulation workflows.
- `protocol/`: CAN + ISO-TP + UDS primitives (`IsoTpLink`, `UdsClient`). All VIN/DTC operations run through this layer.
- `discovery/`: Module discovery pipeline. MVP probes candidate ECUs with UDS tester-present and builds the module list.
- `topology/`: Builds an in-memory graph of buses and modules for UI rendering.
- `app_state/`: Deterministic state machine and snapshot structs for the UI.
- `scanner/`: Orchestrates connection, VIN read, discovery, DTC scan, and final state transition.
- `logger/`: Session logging for raw transport frames and protocol events.

## State Machine

`Disconnected → Connecting → Identifying → Discovering → ScanningDtc → Ready → Error`

All UI transitions listen to `app://snapshot` events emitted by the backend.

## Logging

- Raw transport frames are emitted by a `LoggingTransport` wrapper.
- Protocol-level milestones (VIN read, module discovered, DTC read, clear) are logged by the scanner.
- Logs are stored as JSONL files in the app data directory (`logs/`).

## Simulation Mode

Simulation sessions live in `/samples`. The backend replays VIN, module inventory, and DTCs from the JSON file to drive the UI without a vehicle.
