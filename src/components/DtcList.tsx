import type { DtcInfo } from "../state/types";
import { Badge } from "./ui";

interface Props {
  title: string;
  dtcs: Array<DtcInfo & { moduleId?: string }>;
  emptyMessage?: string;
}

export default function DtcList({ title, dtcs, emptyMessage }: Props) {
  return (
    <div className="dtc-list">
      <div className="dtc-header">
        <h4>{title}</h4>
        <Badge tone={dtcs.length ? "warning" : "success"}>
          {dtcs.length ? `${dtcs.length} active` : "Clean"}
        </Badge>
      </div>
      {dtcs.length === 0 ? (
        <div className="dtc-empty">{emptyMessage ?? "No DTCs present"}</div>
      ) : (
        <div className="dtc-table">
          {dtcs.map((dtc) => (
            <div key={dtc.code} className="dtc-row">
              <div>
                <div className="dtc-code">{dtc.code}</div>
                <div className="dtc-desc">{dtc.description}</div>
                {dtc.moduleId && <div className="dtc-module">{dtc.moduleId}</div>}
              </div>
              <Badge tone="warning">{dtc.status}</Badge>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
