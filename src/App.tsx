import { AnimatePresence, motion } from "framer-motion";
import { type CSSProperties, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

import TopologyTree from "./components/TopologyTree";
import ModuleWorkspace from "./components/ModuleWorkspace";
import DtcList from "./components/DtcList";
import LogsDrawer from "./components/LogsDrawer";
import { Badge, Button, Card, Pill, SectionTitle } from "./components/ui";
import type { AdapterStatus, ModuleInfo, TransportMode } from "./state/types";
import { useAppState } from "./state/useAppState";

const fade = {
  initial: { opacity: 0, y: 12 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -12 },
};

function formatTimestamp(timestamp?: string | null) {
  if (!timestamp) return "";
  const date = new Date(timestamp);
  return date.toLocaleString();
}

export default function App() {
  const { snapshot, dtcTotal } = useAppState();
  const [selectedModuleId, setSelectedModuleId] = useState<string | null>(null);
  const [logsOpen, setLogsOpen] = useState(false);
  const [adapterStatus, setAdapterStatus] = useState<AdapterStatus | null>(null);
  const [connectMode, setConnectMode] = useState<TransportMode>("simulation");

  const modules = snapshot.modules;
  const selectedModule = modules.find((module) => module.id === selectedModuleId) ?? null;
  const aggregatedDtcs = useMemo(() => {
    const entries = Object.entries(snapshot.dtcs);
    return entries.flatMap(([moduleId, dtcs]) =>
      dtcs.map((dtc) => ({ ...dtc, moduleId }))
    );
  }, [snapshot.dtcs]);

  useEffect(() => {
    invoke<AdapterStatus>("get_adapter_status")
      .then(setAdapterStatus)
      .catch(() => null);
  }, []);

  useEffect(() => {
    if (snapshot.phase === "disconnected") {
      setConnectMode(snapshot.transport);
    }
  }, [snapshot.phase, snapshot.transport]);

  useEffect(() => {
    if (snapshot.phase === "ready" && !selectedModuleId && snapshot.modules.length) {
      setSelectedModuleId(snapshot.modules[0]?.id ?? null);
    }
  }, [snapshot.phase, snapshot.modules, selectedModuleId]);

  const handleConnect = () => {
    const simulationPath = connectMode === "simulation" ? "samples/f250_session.json" : null;
    invoke("start_scan", { mode: connectMode, simulation_path: simulationPath });
  };

  const handleClearAll = () => {
    invoke("clear_dtcs", {});
  };

  const handleClearModule = (module: ModuleInfo) => {
    invoke("clear_dtcs", { module_id: module.id });
  };

  return (
    <div className="app-shell">
      <header className="topbar">
        <div>
          <div className="brand">Vyntool</div>
          <div className="subtitle">Desktop Diagnostic Suite</div>
        </div>
        <div className="status-row">
          <Badge tone={snapshot.adapterConnected ? "success" : "danger"}>
            {snapshot.adapterConnected ? "Adapter connected" : "Adapter offline"}
          </Badge>
          <Badge tone="info">{snapshot.transport === "simulation" ? "Simulation" : "Live"}</Badge>
          {snapshot.vin && <Badge tone="neutral">VIN {snapshot.vin}</Badge>}
          <Button variant="ghost" onClick={() => setLogsOpen(true)}>
            Logs
          </Button>
        </div>
      </header>

      <main className="main-content">
        <AnimatePresence mode="wait">
          {snapshot.phase === "disconnected" && (
            <motion.div key="connect" {...fade} className="screen">
              <div className="connect-grid">
                <div className="connect-hero">
                  <h1>Run a full-system scan in one motion.</h1>
                  <p>
                    Plug in the vLinker FS, turn the ignition on, and let Vyntool build a complete
                    vehicle topology with DTC coverage.
                  </p>
                  <div className="connect-actions">
                    <Button onClick={handleConnect}>Connect</Button>
                    <div className="mode-toggle">
                      <Pill
                        active={connectMode === "simulation"}
                        onClick={() => setConnectMode("simulation")}
                      >
                        Simulation
                      </Pill>
                      <Pill
                        active={connectMode === "j2534"}
                        onClick={() => setConnectMode("j2534")}
                      >
                        Live Adapter
                      </Pill>
                    </div>
                  </div>
                </div>

                <Card className="connect-card">
                  <SectionTitle>Adapter status</SectionTitle>
                  <div className="adapter-status">
                    <div>
                      <div className="status-label">
                        {adapterStatus?.available ? "Driver ready" : "Driver missing"}
                      </div>
                      <div className="status-sub">
                        {adapterStatus?.message ?? "Checking J2534 driver"}
                      </div>
                    </div>
                    <Badge tone={adapterStatus?.available ? "success" : "warning"}>
                      {adapterStatus?.available ? "J2534 OK" : "Install driver"}
                    </Badge>
                  </div>
                </Card>

                <Card className="connect-card">
                  <SectionTitle>Last session</SectionTitle>
                  {snapshot.lastSession ? (
                    <div className="session-summary">
                      <div>
                        <span>VIN</span>
                        <strong>{snapshot.lastSession.vin ?? "Unknown"}</strong>
                      </div>
                      <div>
                        <span>Modules</span>
                        <strong>{snapshot.lastSession.moduleCount}</strong>
                      </div>
                      <div>
                        <span>DTCs</span>
                        <strong>{snapshot.lastSession.dtcCount}</strong>
                      </div>
                      <div>
                        <span>Timestamp</span>
                        <strong>{formatTimestamp(snapshot.lastSession.timestamp)}</strong>
                      </div>
                    </div>
                  ) : (
                    <div className="session-empty">No prior sessions saved.</div>
                  )}
                </Card>
              </div>
            </motion.div>
          )}

          {snapshot.phase !== "disconnected" && snapshot.phase !== "ready" && snapshot.phase !== "error" && (
            <motion.div key="scanning" {...fade} className="screen">
              <div className="scan-layout">
                <Card className="scan-card">
                  <div className="scan-progress">
                    <div>
                      <SectionTitle>Scanning</SectionTitle>
                      <h2>{snapshot.progress?.message ?? "Working"}</h2>
                      <p>
                        Phase: {snapshot.phase.replace(/([A-Z])/g, " $1").trim()}
                      </p>
                    </div>
                    <div
                      className="progress-ring"
                      style={{ "--progress": snapshot.progress?.percent ?? 0 } as CSSProperties}
                    >
                      <div className="progress-value">
                        {snapshot.progress?.percent ?? 0}%
                      </div>
                    </div>
                  </div>
                  <div className="progress-bar">
                    <div
                      className="progress-fill"
                      style={{ width: `${snapshot.progress?.percent ?? 0}%` }}
                    />
                  </div>
                </Card>

                <Card className="scan-card">
                  <SectionTitle>Live discoveries</SectionTitle>
                  <div className="scan-list">
                    {modules.length === 0 && <div className="muted">Awaiting responses...</div>}
                    {modules.map((module) => (
                      <div key={module.id} className="scan-row">
                        <div>
                          <strong>{module.name}</strong>
                          <span>{module.bus}</span>
                        </div>
                        <Badge tone={module.dtcCount > 0 ? "warning" : "success"}>
                          {module.dtcCount > 0 ? `${module.dtcCount} DTCs` : "OK"}
                        </Badge>
                      </div>
                    ))}
                  </div>
                </Card>
              </div>
            </motion.div>
          )}

          {snapshot.phase === "ready" && (
            <motion.div key="ready" {...fade} className="screen">
              <div className="summary-strip">
                <div>
                  <h1>Vehicle scan complete</h1>
                  <p>
                    {snapshot.vin ?? "VIN unavailable"} · {modules.length} modules · {dtcTotal} DTCs
                  </p>
                </div>
                <div className="summary-actions">
                  <Button onClick={handleClearAll} variant="danger">
                    Clear All DTCs
                  </Button>
                  <Button variant="outline" onClick={() => setLogsOpen(true)}>
                    Export Logs
                  </Button>
                </div>
              </div>

              <div className="ready-grid">
                <Card className="topology-card">
                  <SectionTitle>Topology</SectionTitle>
                  {snapshot.topology.buses.length === 0 ? (
                    <div className="muted">No topology data available yet.</div>
                  ) : (
                    <TopologyTree
                      topology={snapshot.topology}
                      modules={modules}
                      selectedId={selectedModule?.id}
                      onSelect={setSelectedModuleId}
                    />
                  )}
                </Card>

                <div className="workspace">
                  {selectedModule ? (
                    <ModuleWorkspace
                      module={selectedModule}
                      dtcs={snapshot.dtcs[selectedModule.id] ?? []}
                      onClear={() => handleClearModule(selectedModule)}
                    />
                  ) : (
                    <Card>
                      <SectionTitle>All DTCs</SectionTitle>
                      <DtcList
                        title="Aggregated DTCs"
                        dtcs={aggregatedDtcs}
                        emptyMessage="No DTCs stored across all modules."
                      />
                    </Card>
                  )}
                </div>

                <Card className="dtc-card">
                  <SectionTitle>All modules</SectionTitle>
                  <DtcList
                    title="Aggregated DTCs"
                    dtcs={aggregatedDtcs}
                    emptyMessage="No DTCs stored across all modules."
                  />
                </Card>
              </div>
            </motion.div>
          )}

          {snapshot.phase === "error" && (
            <motion.div key="error" {...fade} className="screen">
              <Card className="error-card">
                <SectionTitle>Something went wrong</SectionTitle>
                <h2>{snapshot.lastError?.summary ?? "Scan failed"}</h2>
                <p>{snapshot.lastError?.details ?? "Please retry the connection."}</p>
                <div className="error-actions">
                  <Button onClick={handleConnect}>Retry scan</Button>
                  <Button variant="outline" onClick={() => setLogsOpen(true)}>
                    Advanced details
                  </Button>
                </div>
              </Card>
            </motion.div>
          )}
        </AnimatePresence>
      </main>

      <LogsDrawer open={logsOpen} onClose={() => setLogsOpen(false)} logsPath={snapshot.logsPath} />
    </div>
  );
}
