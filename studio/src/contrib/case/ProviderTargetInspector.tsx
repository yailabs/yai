import { useEffect, useState } from "react";
import type { AuxiliaryViewProps } from "../../workbench/kernel/types";
import type { LiveWorkspace } from "../../clients/live";
import { Badge, PanelHeader } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

/** Current Tenant inventory when authorized; Case-disclosed details remain separately scoped. */
export function ProviderTargetInspector({ workspace, selection, platform }: AuxiliaryViewProps) {
  const application = platform.application;
  const availability = useApplicationAvailability(application);
  const tenant = workspace.case.tenant_ref;
  const [read, setRead] = useState<{tenant: string; selection: string; catalog: typeof availability.catalog;
    target?: LiveWorkspace["compute"]["targets"][number]; error?: string}>();
  useEffect(() => {
    if (!tenant || !application?.supports("provider.inventory")) return;
    let active = true;
    let pending = false;
    const refresh = () => {
      if (pending) return;
      pending = true;
      void application.providerInventory(tenant).then(result => {
      if (!active) return;
      const target = result.result_state === "success" && result.data?.tenant_ref === tenant
        ? result.data.targets.find(item => item.id === selection) : undefined;
      setRead({tenant, selection, catalog: availability.catalog, target,
        error: target ? undefined : result.error?.safe_message ?? "This target is not available in the bounded Tenant inventory."});
    }).catch(() => { if (active) setRead({tenant, selection, catalog: availability.catalog, error: "Target details unavailable. Reconnect to YAI."}); }).finally(() => { pending = false; });
    };
    refresh();
    const interval = window.setInterval(() => { if (document.visibilityState === "visible") refresh(); }, 10_000);
    return () => { active = false; window.clearInterval(interval); };
  }, [application, availability.catalog, tenant, selection, workspace]);
  const current = read?.tenant === tenant && read?.selection === selection && read?.catalog === availability.catalog
    && availability.state === "available" ? read : undefined;
  const caseTarget = workspace.compute.targets.find(item => item.id === selection);
  const target = current?.target ?? ((!application || availability.state === "available") ? caseTarget : undefined);
  const posture = typeof target?.posture === "object" ? target.posture : undefined;
  return <div className="context-scroll inspector-view" data-inspected-ref={selection}>
    <PanelHeader title="Provider deployment" />
    {!target ? <p role="status" className="surface-note">{current?.error ??
      (!application?.supports("provider.inventory") ? "The connected Host does not expose target inventory." : "Loading exact target…")}</p> : <>
      {current?.error && <p className="surface-note" role="status">Tenant inventory unavailable. Showing only the current Case projection. {current.error}</p>}
      <h2>{target.provider_key}</h2><Badge>{posture?.trust?.posture ?? "Unreviewed"}</Badge>
      <dl className="object-facts provider-inspector-facts"><div className="provider-fact-wide"><dt>Model</dt><dd>{target.model_id}</dd></div>
        <div className="provider-fact-wide"><dt>Endpoint</dt><dd>{target.endpoint}</dd></div>
        <div><dt>Adapter</dt><dd>{target.adapter.replaceAll("_", " ")}</dd></div>
        <div><dt>Locality</dt><dd>{target.locality.replaceAll("_", " ")}</dd></div>
        <div><dt>Current Case</dt><dd>{workspace.compute.targets.some(item => item.id === selection) ? "Bound" : "Not bound"}</dd></div>
        <div><dt>YAI health posture</dt><dd>{posture?.health.effective_posture ?? "Not projected"}</dd></div>
        <div><dt>Last reported health</dt><dd>{posture?.health.posture ?? "Not exposed"}</dd></div>
        <div className="provider-fact-wide"><dt>Evaluated by YAI</dt><dd>{posture?.health.evaluated_at_unix_ms ? new Date(posture.health.evaluated_at_unix_ms).toLocaleString() : "Not projected"}</dd></div>
        <div className="provider-fact-wide"><dt>Last observation</dt><dd>{posture?.health.observed_at_unix_ms ? new Date(posture.health.observed_at_unix_ms).toLocaleString() : "Not observed"}</dd></div>
        <div className="provider-fact-wide"><dt>Failure</dt><dd>{posture?.health.failure_class?.replaceAll("_", " ") ?? "No failure class recorded"}</dd></div>
        <div><dt>Circuit</dt><dd>{posture?.health.circuit ?? "Not exposed"}</dd></div>
        <div><dt>Consecutive failures</dt><dd>{posture?.health.consecutive_failures ?? "Not exposed"}</dd></div></dl>
      <p className="surface-note">{current?.target ? "Current Tenant inventory." : "Current Case projection."} Trust and binding do not grant permission for an operation.</p>
      <details><summary>Technical details</summary><p>{target.id}</p><p>{tenant}</p>
        {posture?.qualification && <p>{posture.qualification.id}</p>}</details>
    </>}
  </div>;
}
