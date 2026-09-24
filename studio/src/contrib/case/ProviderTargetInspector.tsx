import { useEffect, useState } from "react";
import type { AuxiliaryViewProps } from "../../workbench/kernel/types";
import type { LiveWorkspace } from "../../clients/live";
import { Badge, PanelHeader } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

/** Tenant-authorized details for targets outside the current Case binding. */
export function ProviderTargetInspector({ workspace, selection, platform }: AuxiliaryViewProps) {
  const application = platform.application;
  const availability = useApplicationAvailability(application);
  const tenant = workspace.case.tenant_ref;
  const [read, setRead] = useState<{tenant: string; selection: string; catalog: typeof availability.catalog;
    target?: LiveWorkspace["compute"]["targets"][number]; error?: string}>();
  useEffect(() => {
    if (!tenant || !application?.supports("provider.inventory")) return;
    let active = true;
    void application.providerInventory(tenant).then(result => {
      if (!active) return;
      const target = result.result_state === "success" && result.data?.tenant_ref === tenant
        ? result.data.targets.find(item => item.id === selection) : undefined;
      setRead({tenant, selection, catalog: availability.catalog, target,
        error: target ? undefined : result.error?.safe_message ?? "This target is not available in the bounded Tenant inventory."});
    }).catch(() => { if (active) setRead({tenant, selection, catalog: availability.catalog, error: "Target details unavailable. Reconnect to YAI."}); });
    return () => { active = false; };
  }, [application, availability.catalog, tenant, selection, workspace]);
  const current = read?.tenant === tenant && read?.selection === selection && read?.catalog === availability.catalog
    && availability.state === "available" ? read : undefined;
  const target = current?.target;
  const posture = typeof target?.posture === "object" ? target.posture : undefined;
  return <div className="context-scroll inspector-view">
    <PanelHeader title="Provider deployment" />
    {!target ? <p role="status" className="surface-note">{current?.error ??
      (!application?.supports("provider.inventory") ? "The connected Host does not expose target inventory." : "Loading exact target…")}</p> : <>
      <h2>{target.provider_key}</h2><Badge>{posture?.trust?.posture ?? "Unreviewed"}</Badge>
      <dl className="object-facts"><div><dt>Model</dt><dd>{target.model_id}</dd></div>
        <div><dt>Adapter</dt><dd>{target.adapter.replaceAll("_", " ")}</dd></div>
        <div><dt>Endpoint</dt><dd>{target.endpoint}</dd></div>
        <div><dt>Locality</dt><dd>{target.locality.replaceAll("_", " ")}</dd></div>
        <div><dt>Current Case</dt><dd>{workspace.compute.targets.some(item => item.id === selection) ? "Bound" : "Not bound"}</dd></div>
        <div><dt>Observed health</dt><dd>{posture?.health.posture ?? "Not exposed"}</dd></div></dl>
      <p className="surface-note">Tenant-owned deployment. Trust and binding do not grant permission for an operation.</p>
      <details><summary>Technical details</summary><p>{target.id}</p><p>{tenant}</p>
        {posture?.qualification && <p>{posture.qualification.id}</p>}</details>
    </>}
  </div>;
}
