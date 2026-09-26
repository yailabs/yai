import { ProviderQualificationCheck } from "./ProviderQualification";
import { ProviderRegistrationFields } from "./ProviderRegistrationFields";
import { ProviderWorkspace } from "./ProviderWorkspace";
import { YvexWorkspace } from "./YvexWorkspace";
import { ConversationModelSetup } from "./ConversationModelSetup";
import { CognitiveExecution } from "./CognitiveExecution";
import { useCallback, useEffect, useId, useState, useSyncExternalStore } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { ProviderProbeEvidence, ProviderQualification, ProviderTarget } from "../../clients/compute";
import { readProbeEvidence } from "../../clients/compute";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button, EmptyState } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { useApplicationAvailability } from "./applicationActions";

type Action = "register" | "qualify" | "trust" | "bind";
const titles: Record<Action, string> = { register: "Register provider target", qualify: "Import qualification evidence", trust: "Set target trust", bind: "Bind provider to Case" };

export function ComputeSurface({ workspace, platform, actions, scope = "case" }: Pick<SurfaceRendererProps, "workspace" | "platform" | "actions"> & { scope?: "case" | "tenant" | "yvex" }) {
  const sections = ["Bindings", "Conversation", "Execution"] as const;
  type Section = typeof sections[number];
  const identity = useId();
  const sectionKey = `studio.compute.section:${JSON.stringify([workspace.case.case_ref, workspace.case.participant_ref])}`;
  const subscribe = useCallback((listener: () => void) => platform.context.subscribe(listener).dispose, [platform]);
  const section = useSyncExternalStore(subscribe, () => {
    const value = platform.context.get(sectionKey) as Section;
    return sections.includes(value) ? value : "Bindings";
  });
  const select = (value: Section) => platform.context.update(sectionKey, value);
  const panel = (value: Section) => ({ role: "tabpanel" as const, id: `${identity}-${value}-panel`, "aria-labelledby": `${identity}-${value}-tab`, hidden: section !== value, tabIndex: 0 });
  const application = platform.application;
  const availability = useApplicationAvailability(application);
  const [revision, setRevision] = useState(0);
  const [inventory, setInventory] = useState<{tenant: string; catalog: typeof availability.catalog; targets: typeof workspace.compute.targets; omitted: number}>();
  const [inventoryError, setInventoryError] = useState<string>();
  const [refreshedAt, setRefreshedAt] = useState<number>();
  const [refreshing, setRefreshing] = useState(false);
  const tenant = workspace.case.tenant_ref;
  useEffect(() => {
    if (scope === "case" || !tenant || !application?.supports("provider.inventory")) return;
    let current = true;
    let pending = false;
    const read = () => {
    if (pending) return;
    pending = true;
    setRefreshing(true);
    setInventoryError(undefined);
    void application.providerInventory(tenant).then(result => {
      if (!current) return;
      if (result.result_state === "success" && result.data?.tenant_ref === tenant) {
        setRefreshedAt(Date.now());
        setInventory({tenant, catalog: availability.catalog, targets: result.data.targets, omitted: result.data.omitted});
      } else { setInventory(undefined); setInventoryError(result.error?.safe_message ?? "Provider inventory unavailable."); }
    }).catch(() => { if (current) { setInventory(undefined); setInventoryError("Provider inventory connection unavailable."); } }).finally(() => { pending = false; if (current) setRefreshing(false); });
    };
    read();
    const timer = window.setInterval(() => { if (document.visibilityState === "visible") read(); }, 10000);
    return () => { current = false; window.clearInterval(timer); };
  }, [application, availability.catalog, tenant, scope, revision, workspace]);
  const currentInventory = inventory && inventory.tenant === tenant && inventory.catalog === availability.catalog && availability.state === "available" ? inventory : undefined;
  const targets = scope === "case" ? workspace.compute.targets : (currentInventory?.targets ?? []).filter(target => scope !== "yvex" || target.extension_adapter_id === "yvex.http.v1");
  const [yvexSetup, setYvexSetup] = useState(false);
  const [action, setAction] = useState<Action>();
  const [candidate, setCandidate] = useState<ProviderTarget>();
  const [target, setTarget] = useState("");
  const [selectedTarget, setSelectedTarget] = useState(false);
  const [qualification, setQualification] = useState<ProviderQualification>();
  const [evidence, setEvidence] = useState<ProviderProbeEvidence>();
  const [fileError, setFileError] = useState<string>();
  const [trustReceipt, setTrustReceipt] = useState<{ target: string; posture: string }>();
  const operation = (name: Action) => ({ register: "provider.register", qualify: "provider.qualify", trust: "provider.trust.set", bind: "provider.case.bind" })[name];
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => { setRevision(value => value + 1); });
  const begin = (name: Action, ref?: string) => { setTarget(ref ?? ""); setSelectedTarget(Boolean(ref)); setEvidence(undefined); setFileError(undefined); setAction(name); };
  const controls = (ref?: string) => <div className="object-action-row">{(["qualify", "trust", "bind"] as const).map(name => <Button key={name} disabled={!application?.supports(operation(name))} onClick={() => begin(name, ref)}>{titles[name]}</Button>)}</div>;
  const models = [...new Set(targets.map(item => item.model_id))];
  return <article className={scope === "case" ? "compute-case-workspace compute-surface" : "compute-surface platform-surface"} data-surface-type={scope === "case" ? "case.compute" : scope === "tenant" ? "platform.providers" : "platform.yvex"}>
    {scope !== "case" && (scope === "yvex" ? <YvexWorkspace key={tenant} application={application} workspace={workspace} targets={targets} omitted={currentInventory?.omitted} error={inventoryError ?? (!application?.supports("provider.inventory") ? "The connected Host does not expose Tenant inventory." : undefined)} ready={Boolean(currentInventory)} refreshedAt={refreshedAt} refreshing={refreshing} refresh={() => setRevision(value => value + 1)} connect={() => { setYvexSetup(true); begin("register"); }} canConnect={Boolean(tenant && application?.supports("provider.register"))} controls={controls} qualification={ref => tenant ? <ProviderQualificationCheck platform={platform} tenant={tenant} target={ref} onCompleted={refresh} /> : null} actions={actions} /> : <ProviderWorkspace key={tenant} application={application} scope="tenant" workspace={workspace} targets={targets} omitted={currentInventory?.omitted} error={inventoryError ?? (!application?.supports("provider.inventory") ? "The connected Host does not expose Tenant inventory." : undefined)} ready={Boolean(currentInventory)} refreshedAt={refreshedAt} refreshing={refreshing} refresh={() => setRevision(value => value + 1)} connect={() => { setYvexSetup(false); begin("register"); }} canConnect={Boolean(tenant && application?.supports("provider.register"))} controls={controls} qualification={ref => tenant ? <ProviderQualificationCheck platform={platform} tenant={tenant} target={ref} onCompleted={refresh} /> : null} actions={actions} />)}
    {scope === "case" && <>
    <header className="operational-heading"><div><h1>Compute</h1><p>Computational capability bound to this Case.</p></div><Button disabled={!workspace.case.tenant_ref || !application?.supports("provider.register")} onClick={() => { setYvexSetup(false); begin("register"); }}>Register provider target</Button></header>
    <nav className="compute-sections" role="tablist" aria-label="Compute sections" onKeyDown={event => {
      const index = sections.indexOf(section);
      const next = event.key === "ArrowRight" ? (index + 1) % sections.length : event.key === "ArrowLeft" ? (index + sections.length - 1) % sections.length : event.key === "Home" ? 0 : event.key === "End" ? sections.length - 1 : undefined;
      if (next === undefined) return;
      event.preventDefault(); select(sections[next]);
      event.currentTarget.querySelector<HTMLButtonElement>(`[data-compute-section="${sections[next]}"]`)?.focus();
    }}>{sections.map(value => <button key={value} role="tab" id={`${identity}-${value}-tab`} data-compute-section={value} aria-controls={`${identity}-${value}-panel`} aria-selected={section === value} tabIndex={section === value ? 0 : -1} onClick={() => select(value)}>{value}</button>)}</nav>
    <div className="compute-case-content">
    <section {...panel("Bindings")}>
    <section className="compute-section"><h2>Targets / deployments <span>{targets.length}</span></h2>
      {!targets.length && <EmptyState title={scope === "case" ? "No bound target" : "No matching deployment"} body="Registration, qualification, trust and Case binding are separate governed steps." />}
      {targets.map(item => { const posture = typeof item.posture === "object" ? item.posture : undefined; return <article className="compute-target" key={item.id}><header><Icon name="compute" /><button className="object-link" onClick={() => actions.inspect(item.id)}>{item.provider_key}</button><Badge tone={posture?.trust?.posture === "approved" ? "success" : posture?.trust?.posture === "denied" ? "error" : "warning"}>{posture?.trust?.posture ?? "Unreviewed"}</Badge></header>
        <dl className="object-facts"><div><dt>Model</dt><dd>{item.model_id}</dd></div><div><dt>YAI health posture</dt><dd>{posture?.health.effective_posture ?? "Not projected"}{posture?.health.observed_at_unix_ms ? ` · last report ${posture.health.posture} ${new Date(posture.health.observed_at_unix_ms).toLocaleString()}` : " · no report observed"}</dd></div></dl>{controls(item.id)}
        <details><summary>Exact target / evidence</summary><code>{item.id}</code>{posture?.qualification && <p>{posture.qualification.id} · run {posture.qualification.run_id}</p>}</details>
      </article>; })}
    </section>
    {candidate && candidate.tenant_id === workspace.case.tenant_ref && !targets.some(item => item.id === candidate.target_id) && <section className="compute-section candidate-target"><h2>Registered in this session</h2><p><strong>{candidate.provider_key}</strong> · {candidate.model_id} · {candidate.endpoint}</p><code>{candidate.target_id}</code><p>Retained by YAI; not bound to this Case. Open Providers to discover the Tenant inventory independently of Case bindings.</p>{controls(candidate.target_id)}</section>}
    {qualification && <p className="operation-receipt" role="status">Qualification recorded: {qualification.capabilities.map(item => item.capability).join(", ") || "no proven capability"} · {qualification.qualification_id}</p>}
    {trustReceipt && <p className="operation-receipt" role="status">Trust {trustReceipt.posture} recorded for {trustReceipt.target}. Binding and effect-time admission are separate.</p>}
    <section className="compute-section"><h2>Available deployments</h2><p>Choose a deployment from the Tenant inventory, check its qualification and trust, then bind it to this Case.</p><Button onClick={() => actions.openPerspective("Providers")}>Browse Providers</Button>
      <details className="compute-exact-target"><summary>Advanced: exact target reference</summary><p>Use a reference already returned by YAI. The current Case binding and authority checks still apply.</p>{controls()}</details>
    </section>
    </section>
    <section {...panel("Conversation")}><ConversationModelSetup workspace={workspace} platform={platform} /></section>
    <section {...panel("Execution")}><CognitiveExecution workspace={workspace} platform={platform} />
    {scope === "case" && <section className="compute-section"><h2>Models & cognition</h2><p>{models.length ? `Models in this Case's bound targets: ${models.join(", ")}.` : "No bound model identity is exposed."} A model name is distinct from its provider/runtime and exact deployment.</p><p>Provider binding does not establish a cognitive-role binding. Work offers bounded execution and exact submission observation when advertised by the connected Host. Conversation sends committed text through the current primary cognitive assignment. Interrupted delivery is observed without automatic redispatch.</p></section>}
    </section></div>
    </>}
    {action && application && <ApplicationActionDialog key={action} title={titles[action]} description={action === "register" ? "Register an immutable target in the current Tenant. Enter a credential reference only, never a token. YAI validates the endpoint and ownership." : action === "qualify" ? "Import a measured ProviderProbeEvidence record from a real YAI probe. This records evidence; it does not run a network test. The current Application operation does not execute the probe." : action === "trust" ? "Set owner-authenticated trust for this exact target. Trust does not imply capability qualification or permission to execute." : "Bind one exact governed target to the current Participant. YAI checks Tenant ownership, Participant and exact target identity. Qualification and trust are checked again when execution is admitted. This replaces the Case provider envelope; it does not send a prompt."}
      submitLabel={action === "register" ? "Register target" : action === "qualify" ? "Record evidence" : action === "trust" ? "Record trust" : "Bind target"} close={() => setAction(undefined)}
      enabled={application.supports(operation(action)) && (action !== "qualify" || Boolean(evidence && evidence.target_id === target.trim()))}
      submit={async form => {
        if (action === "register") { const result = await application.registerProvider({ tenant_id: workspace.case.tenant_ref!, provider_key: String(form.get("key")).trim(), adapter: "open_ai_compatible", endpoint: String(form.get("endpoint")).trim(), model_id: String(form.get("model")).trim(), credential_ref: String(form.get("credential")).trim(), locality: String(form.get("locality")) as ProviderTarget["locality"], ...(form.get("extension") === "yvex.http.v1" ? { extension_adapter_id: "yvex.http.v1" as const } : {}) }); if (result.result_state === "success" && result.data) { setCandidate(result.data); setTarget(result.data.target_id); if (scope === "case") select("Bindings"); } return result; }
        if (action === "qualify") { const result = await application.qualifyProvider({ target_ref: target.trim(), evidence: evidence!, suite_ref: String(form.get("suite")).trim() }); if (result.result_state === "success" && result.data) setQualification(result.data); return result; }
        if (action === "trust") { const posture = String(form.get("posture")) as "approved" | "denied"; const result = await application.trustProvider({ target_ref: target.trim(), posture }); if (result.result_state === "success") setTrustReceipt({ target: target.trim(), posture }); return result; }
        return application.bindProvider({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, ordered_target_refs: [target.trim()], failover_policy: "none", max_attempts_per_turn: 1 });
      }} committed={refresh} resync={refresh}>
      {action === "register" ? <ProviderRegistrationFields application={application} tenant={tenant!} yvex={yvexSetup} /> : !selectedTarget ? <label>Exact target reference<input autoFocus required value={target} onChange={event => setTarget(event.target.value)} /></label> : <p>Selected deployment: {targets.find(item => item.id === target)?.provider_key ?? (candidate?.target_id === target ? candidate.provider_key : target)}</p>}
      {action === "qualify" && <><label>Measured evidence file<input type="file" accept="application/json,.json" required onChange={async event => { setEvidence(undefined); setFileError(undefined); const file = event.target.files?.[0]; if (!file) return; if (file.size > 65536) { setFileError("Evidence must be at most 64 KiB."); return; } try { const value = readProbeEvidence(JSON.parse(await file.text())); if (!value) throw new Error(); setEvidence(value); } catch { setFileError("This is not a valid ProviderProbeEvidence record."); } }} /></label><label>Qualification suite reference<input required name="suite" placeholder="Exact suite reference associated with the probe" /></label>{fileError && <p role="alert">{fileError}</p>}{evidence && <p>Run {evidence.run_id} · {evidence.failure_codes.length} reported failures{evidence.target_id !== target.trim() && " · target mismatch: not submitted"}</p>}</>}
      {action === "trust" && <label>Trust decision<select name="posture"><option value="approved">Approve</option><option value="denied">Deny</option></select></label>}
      {action === "bind" && <p>Participant: {workspace.case.participant_ref}. One target, no failover, one attempt. Existing ordered bindings will be replaced.</p>}
    </ApplicationActionDialog>}
  </article>;
}

export function ProvidersSurface(props: SurfaceRendererProps) { return <ComputeSurface {...props} scope="tenant" />; }
export function YvexSurface(props: SurfaceRendererProps) { return <ComputeSurface {...props} scope="yvex" />; }

export function ProviderNavigation({ workspace, actions, containerId }: import("../../workbench/kernel/types").SidebarViewProps) {
  return <nav className="provider-navigation" aria-label="Computational platform">
    <small>Tenant platform</small>
    {(["Providers", "YVEX"] as const).map(id => <button key={id} className="fact-row" aria-current={containerId === id ? "page" : undefined} onClick={() => actions.openPerspective(id)}><Icon name="compute" /><span>{id}</span></button>)}
    <small>Current Case</small><button className="fact-row" onClick={() => actions.openPerspective("Compute")}><Icon name="case" /><span>{workspace.case.display_name}<small>Bindings &amp; cognition</small></span></button>
  </nav>;
}
