import { save } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";

import { Button } from "./ui";

interface Props {
  open: boolean;
  onClose: () => void;
  logsPath?: string | null;
}

export default function LogsDrawer({ open, onClose, logsPath }: Props) {
  const [tail, setTail] = useState<string>("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;
    invoke<string>("read_log_tail", { lines: 80 })
      .then(setTail)
      .catch(() => setTail("No logs available yet."));
  }, [open]);

  const exportLogs = async () => {
    setBusy(true);
    try {
      const destination = await save({
        title: "Export Vyntool Logs",
        defaultPath: "vyntool-session.jsonl",
      });
      if (destination) {
        await invoke("export_logs", { destination });
      }
    } finally {
      setBusy(false);
    }
  };

  if (!open) return null;

  return (
    <div className="drawer-backdrop" onClick={onClose}>
      <div className="drawer" onClick={(event) => event.stopPropagation()}>
        <div className="drawer-header">
          <div>
            <h3>Advanced Logs</h3>
            <p>{logsPath ?? "Session log"}</p>
          </div>
          <div className="drawer-actions">
            <Button variant="outline" onClick={exportLogs} disabled={busy}>
              Export Logs
            </Button>
            <Button variant="ghost" onClick={onClose}>
              Close
            </Button>
          </div>
        </div>
        <pre className="drawer-log">{tail}</pre>
      </div>
    </div>
  );
}
