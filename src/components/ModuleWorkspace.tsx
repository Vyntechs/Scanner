import { useMemo, useState } from "react";

import type { DtcInfo, ModuleInfo } from "../state/types";
import DtcList from "./DtcList";
import { Badge, Button } from "./ui";

interface Props {
  module: ModuleInfo;
  dtcs: DtcInfo[];
  onClear: () => void;
}

const tabs = ["Codes", "Live Data", "Actuations", "Info"] as const;

export default function ModuleWorkspace({ module, dtcs, onClear }: Props) {
  const [activeTab, setActiveTab] = useState<(typeof tabs)[number]>("Codes");

  const infoLines = useMemo(
    () => [
      { label: "TX ID", value: `0x${module.txId.toString(16).toUpperCase()}` },
      { label: "RX ID", value: `0x${module.rxId.toString(16).toUpperCase()}` },
      { label: "Bus", value: module.bus },
      { label: "Category", value: module.category },
      { label: "Status", value: module.dtcCount > 0 ? "Attention" : "OK" },
    ],
    [module]
  );

  return (
    <div className="module-workspace">
      <div className="module-header">
        <div>
          <h2>{module.name}</h2>
          <div className="module-subtitle">{module.id}</div>
        </div>
        <div className="module-actions">
          <Badge tone={module.dtcCount > 0 ? "warning" : "success"}>
            {module.dtcCount > 0 ? `${module.dtcCount} DTCs` : "OK"}
          </Badge>
          <Button variant="outline" onClick={onClear}>
            Clear This Module
          </Button>
        </div>
      </div>

      <div className="tabs">
        {tabs.map((tab) => (
          <button
            key={tab}
            className={`tab ${tab === activeTab ? "tab-active" : ""}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab}
          </button>
        ))}
      </div>

      <div className="tab-panel">
        {activeTab === "Codes" && (
          <DtcList title="Module DTCs" dtcs={dtcs} emptyMessage="No active DTCs." />
        )}
        {activeTab === "Live Data" && (
          <div className="stub-panel">
            <h4>Live Data</h4>
            <p>Real-time PID streaming will appear here in a future release.</p>
          </div>
        )}
        {activeTab === "Actuations" && (
          <div className="stub-panel">
            <h4>Actuations</h4>
            <p>Guided output controls will be added after MVP validation.</p>
          </div>
        )}
        {activeTab === "Info" && (
          <div className="info-grid">
            {infoLines.map((line) => (
              <div key={line.label} className="info-card">
                <span>{line.label}</span>
                <strong>{line.value}</strong>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
