import type { PlatformServices } from "../../platform/services";
import { CollectionList } from "../../components/CollectionList";
import { Badge, Button } from "../../components/primitives";
import { useApplicationAvailability } from "../case/applicationActions";

import { studioOperationPosture } from "./capabilityPosture";

export function ApplicationCapabilities({ platform }: { platform: PlatformServices }) {
  const availability = useApplicationAvailability(platform.application);
  const catalog = availability.catalog;
  return <section className="application-capabilities" aria-label="Application capabilities">
    <header><div><h3>Connected application</h3><p>Operations reported by the current Host. Availability does not grant authority to execute them.</p></div><Badge tone={availability.state === "available" ? "success" : "warning"}>{availability.state}</Badge></header>
    {availability.reason && <p role="status">{availability.reason}</p>}
    {catalog && <><p>{catalog.operations.length} operations advertised · {catalog.operations.filter(item => studioOperationPosture(item.operation_id).state === "integrated").length} connected to Studio interactions</p><CollectionList label="Application operations" items={catalog.operations.map(item => ({ id: item.operation_id, title: item.meaning, detail: `${item.operation_id} · ${item.impact.replaceAll("_", " ")}${studioOperationPosture(item.operation_id).state === "integrated" ? "" : ` · ${studioOperationPosture(item.operation_id).detail}`}`, meta: studioOperationPosture(item.operation_id).state === "integrated" ? "Studio integrated" : studioOperationPosture(item.operation_id).state === "alternative" ? "Alternate path" : "UI debt", searchText: `${item.authority} ${item.input_contract} ${item.output_contract}` }))} /></>}
    {platform.application && <Button disabled={availability.state === "checking" || platform.host.snapshot().state !== "live"} onClick={() => void platform.application!.refresh()}>Refresh Host capabilities</Button>}
  </section>;
}
