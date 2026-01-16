import clsx from "clsx";
import type { ModuleInfo, TopologyGraph } from "../state/types";

import { Badge } from "./ui";

interface Props {
  topology: TopologyGraph;
  modules: ModuleInfo[];
  selectedId?: string | null;
  onSelect: (id: string) => void;
}

export default function TopologyTree({ topology, modules, selectedId, onSelect }: Props) {
  const moduleMap = new Map(modules.map((module) => [module.id, module]));

  return (
    <div className="topology-tree">
      {topology.buses.map((bus) => (
        <div key={bus.name} className="bus-group">
          <div className="bus-header">
            <span className="bus-dot" />
            <span className="bus-title">{bus.name}</span>
            <Badge tone="info">{bus.modules.length} nodes</Badge>
          </div>
          <div className="bus-list">
            {bus.modules.map((moduleId) => {
              const module = moduleMap.get(moduleId);
              if (!module) return null;
              const isSelected = module.id === selectedId;
              return (
                <button
                  key={module.id}
                  className={clsx("module-node", isSelected && "module-node-active")}
                  onClick={() => onSelect(module.id)}
                >
                  <div className="module-node-main">
                    <span className="module-name">{module.name}</span>
                    <span className="module-meta">{module.category}</span>
                  </div>
                  <div className="module-node-status">
                    <Badge tone={module.dtcCount > 0 ? "warning" : "success"}>
                      {module.dtcCount > 0 ? `${module.dtcCount} DTCs` : "OK"}
                    </Badge>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
