import type { PlatformServices } from "../../platform/services";
import { CollectionList } from "../../components/CollectionList";
import { Badge, Button } from "../../components/primitives";
import { useApplicationAvailability } from "../case/applicationActions";

// Explicit implemented interactions, not the catalog's desired studio_posture.
const consumed = new Set([
  "application.capabilities", "case.list", "case.open", "case.summary", "material.read",
  "events.subscribe", "events.heartbeat", "tenant.list", "case.create", "case.close", "case.cancel",
  "review.approve", "review.deny", "review.defer", "workflow.input.record",
  "participant.role.add", "participant.principal.link",
]);

export function ApplicationCapabilities({ platform }: { platform: PlatformServices }) {
  const availability = useApplicationAvailability(platform.application);
  const catalog = availability.catalog;
  return <section className="application-capabilities" aria-label="Application capabilities">
    <header><div><h3>Connected application</h3><p>Operations reported by the current Host. Availability does not grant authority to execute them.</p></div><Badge tone={availability.state === "available" ? "success" : "warning"}>{availability.state}</Badge></header>
    {availability.reason && <p role="status">{availability.reason}</p>}
    {catalog && <><p>{catalog.operations.length} operations advertised · {catalog.operations.filter(item => consumed.has(item.operation_id)).length} connected to Studio interactions</p><CollectionList label="Application operations" items={catalog.operations.map(item => ({ id: item.operation_id, title: item.meaning, detail: `${item.operation_id} · ${item.impact.replaceAll("_", " ")}`, meta: consumed.has(item.operation_id) ? "Studio integrated" : "UI not integrated", searchText: `${item.authority} ${item.input_contract} ${item.output_contract}` }))} /></>}
    {platform.application && <Button disabled={availability.state === "checking" || platform.host.snapshot().state !== "live"} onClick={() => void platform.application!.refresh()}>Refresh Host capabilities</Button>}
  </section>;
}
