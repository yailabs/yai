import { DeploymentModelObservation } from "./DeploymentModelObservation";
import type { ApplicationAccess } from "../../clients/application";
import { useState, type ReactNode } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { WorkbenchActions } from "../../workbench/kernel/types";
import { Badge, Button } from "../../components/primitives";
import { Icon } from "../../components/Icon";

type Target = CasePresentation["compute"]["targets"][number];
const time = (value?: number) => value ? new Date(value).toLocaleString() : "Not observed";
const reasons: Record<string, string> = {
  delivery_or_response_unknown: "YAI could not establish whether a previous request was delivered or completed. This historical report does not prove the model is unavailable. Inspect that execution; no automatic resend occurs.",
  remote_response_rejected: "The provider rejected a previous request. Inspect the execution for its HTTP status and exact outcome.",
  request_capacity_refused: "The prepared request could not be admitted against the target capacity. No automatic truncation or resend is performed.",
  qualification_probe_failed: "The last synthetic probe did not establish a usable response from this exact target.",
};
export function ProviderWorkspace({ scope, workspace, application, targets, omitted = 0, error, ready, refreshedAt, refreshing, refresh, connect, canConnect, controls, qualification, actions }: {
  scope: "tenant" | "yvex"; workspace: CasePresentation; application?: ApplicationAccess; targets: Target[]; omitted?: number; error?: string;
  ready: boolean; refreshedAt?: number; refreshing: boolean; refresh(): void; connect(): void; canConnect: boolean;
  controls(id: string): ReactNode; qualification(id: string): ReactNode; actions: WorkbenchActions;
}) {
  const [selected, setSelected] = useState<string>();
  const [section, setSection] = useState("Runtime");
  const item = targets.find(target => target.id === selected) ?? targets[0];
  const posture = typeof item?.posture === "object" ? item.posture : undefined;
  const health = posture?.health;
  const bound = item && workspace.compute.targets.some(target => target.id === item.id);
  const effectiveHealth = health?.effective_posture;
  const expiredPositive = effectiveHealth === "unknown" && health?.posture !== "unknown" && Boolean(health?.observed_at_unix_ms);
  const tone = effectiveHealth === "unavailable" ? "error" : effectiveHealth === "degraded" ? "warning" : "neutral";
  return <div className="provider-workspace">
    <header className="provider-workspace-toolbar"><div><Icon name={scope === "yvex" ? "processor" : "providers"} /><h1>{scope === "yvex" ? "YVEX" : "Providers"}</h1><span>{targets.length} deployment{targets.length === 1 ? "" : "s"}{omitted > 0 ? ` · ${omitted} omitted by read bound` : ""}</span></div><div><Button onClick={refresh} disabled={refreshing}>{refreshing ? "Refreshing…" : "Refresh inventory"}</Button><Button disabled={!canConnect} onClick={connect}>{scope === "yvex" ? "Connect compatible deployment" : "Register provider target"}</Button></div></header>
    <div className="provider-workspace-status" role="status">{error ?? (!ready ? "Reading authorized inventory…" : `YAI inventory updated ${time(refreshedAt)}`)}<span>Inventory refreshes every 10s · model catalog only when checked</span></div>
    <div className="provider-workspace-body">
      <label className="deployment-compact-picker">Deployment<select aria-label="Selected deployment" value={item?.id ?? ""} onChange={event => { setSelected(event.target.value); actions.inspect(event.target.value); }}>
        {!targets.length && <option value="">No deployment</option>}{targets.map(target => <option key={target.id} value={target.id}>{target.provider_key} · {target.model_id}</option>)}
      </select></label>
      <section className="deployment-workspace" aria-label="Deployment details">
        {item ? <><header className="deployment-heading"><div><small>Selected deployment</small><h2>{item.provider_key}</h2><Badge tone={posture?.trust?.posture === "approved" ? "success" : posture?.trust?.posture === "denied" ? "error" : "neutral"}>{posture?.trust?.posture ?? "Unreviewed"}</Badge></div><Button onClick={() => actions.inspect(item.id)}>Inspect deployment</Button></header>
          <nav className="deployment-sections" aria-label="Deployment sections">{["Runtime", "Evidence", "Platform"].map(name => <button key={name} aria-pressed={section === name} onClick={() => setSection(name)}>{name}</button>)}</nav>
          <div className="deployment-content">
          {section === "Runtime" && <>
            {workspace.case.tenant_ref && <DeploymentModelObservation key={item.id} application={application} tenant={workspace.case.tenant_ref} target={item.id} model={item.model_id} />}
            <section className="deployment-health" aria-label="Observed deployment health"><header><h3>YAI provider observation</h3><Badge tone={tone}>{effectiveHealth === undefined ? "Not projected" : expiredPositive ? "Observation expired" : effectiveHealth}</Badge></header>
              <p>{effectiveHealth === undefined ? "This Host does not project YAI's effective health posture. The last report below is historical." : expiredPositive ? `The ${health?.posture} report has expired. It does not describe current model health.` : effectiveHealth === "unknown" ? "YAI has no current health observation for this target." : health?.failure_class ? reasons[health.failure_class] ?? `YAI recorded: ${health.failure_class.replaceAll("_", " ")}.` : effectiveHealth === "healthy" ? "YAI considers its recent observation healthy at the time shown below." : "No failure explanation is exposed in the current observation."}</p>
              {expiredPositive && health?.failure_class && <p>{reasons[health.failure_class] ?? `Historical cause: ${health.failure_class.replaceAll("_", " ")}.`}</p>}
              <dl><div><dt>Last report</dt><dd>{health?.observed_at_unix_ms ? `${health.posture} · ${time(health.observed_at_unix_ms)}` : "Not observed"}</dd></div><div><dt>Evaluated by YAI</dt><dd>{time(health?.evaluated_at_unix_ms ?? undefined)}</dd></div><div><dt>Circuit</dt><dd>{health?.circuit ?? "Unknown"}</dd></div><div><dt>Consecutive failures</dt><dd>{health?.consecutive_failures ?? "Unknown"}</dd></div></dl>
              {health?.failure_class && <code>{health.failure_class}</code>}
              <small>Effective posture comes from YAI's health owner. Check exposed model in this page to refresh catalog evidence; the report is not a live connectivity probe.</small><Button onClick={() => setSection("Evidence")}>Check deployment</Button>
            </section>
            <section className="deployment-case"><header><h3>Current Case</h3><Badge tone={bound ? "info" : "neutral"}>{bound ? "Bound" : "Not bound"}</Badge></header><p>{workspace.case.display_name}</p><Button onClick={() => actions.openPerspective("Compute")}>Open Case Compute</Button></section>
          </>}
          {section === "Evidence" && <>{qualification(item.id)}<details className="deployment-retained-qualification"><summary>Current retained qualification</summary>{posture?.qualification ? <><ul className="deployment-capabilities">{posture.qualification.capabilities.map(capability => <li key={capability.capability}><strong>{capability.capability}</strong><span>{capability.provenance}</span></li>)}</ul><p>Measured {time(posture.qualification.qualified_at_unix_ms)}</p><code>{posture.qualification.run_id}</code></> : <p>No qualification is exposed for this target.</p>}</details><section><h3>Governance</h3><p>These actions apply to the selected deployment. Trust does not grant execution authority.</p>{controls(item.id)}</section><Button onClick={() => actions.inspect(item.id)}>Inspect exact identities</Button></>}
          {section === "Platform" && <><h3>{scope === "yvex" ? "YVEX management boundary" : "Provider management boundary"}</h3><p>Inference compatibility and native platform management are separate connections.</p><div className="platform-contracts">{[
            ["Sources & artifacts", "Acquisition, tensors and representation identity", "No typed management connection"],
            ["Compiler", "Preparation, compilation and produced artifacts", "No typed management connection"],
            ["Runtime & devices", "Registered machines, engines, load/unload and memory", "No typed management connection"],
            ["Inference", "Exact target, qualification and Case binding", "Available through YAI governance"],
            ["Logs & telemetry", "Server events, device accounting and sessions", "No typed management connection"],
          ].map(([name, detail, state]) => <div key={name}><strong>{name}</strong><span>{detail}</span><small>{state}</small></div>)}</div></>}
          </div>
        </> : <div className="deployment-idle"><Icon name="providers" size={32} /><h2>No deployment connected</h2><p>Connect a published endpoint to inspect its operational state and evidence.</p></div>}
      </section>
    </div>
  </div>;
}
