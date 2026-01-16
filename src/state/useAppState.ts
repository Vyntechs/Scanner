import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import type { AppSnapshot } from "./types";

const emptySnapshot: AppSnapshot = {
  phase: "disconnected",
  transport: "simulation",
  adapterConnected: false,
  vin: null,
  modules: [],
  dtcs: {},
  topology: { buses: [] },
  progress: null,
  lastError: null,
  sessionId: null,
  logsPath: null,
  lastSession: null,
};

export function useAppState() {
  const [snapshot, setSnapshot] = useState<AppSnapshot>(emptySnapshot);

  useEffect(() => {
    invoke<AppSnapshot>("get_snapshot")
      .then((data) => {
        setSnapshot(data);
      })
      .catch(() => null);

    let unlisten: (() => void) | undefined;
    listen<AppSnapshot>("app://snapshot", (event) => {
      setSnapshot(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const dtcTotal = useMemo(
    () => Object.values(snapshot.dtcs).reduce((sum, list) => sum + list.length, 0),
    [snapshot.dtcs]
  );

  return {
    snapshot,
    setSnapshot,
    dtcTotal,
  };
}
