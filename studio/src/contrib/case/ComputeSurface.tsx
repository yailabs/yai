import { ConversationModelSetup } from "./ConversationModelSetup";
import { useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { ProviderProbeEvidence, ProviderQualification, ProviderTarget } from "../../clients/compute";
import { readProbeEvidence } from "../../clients/compute";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button, EmptyState } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { useApplicationAvailability } from "./applicationActions";

type Action = "register" | "qualify" | "trust" | "bind";
const titles: Record<Action, string> = { register: "Register provider target", qualify: "Import qualification evidence", trust: "Set target trust", bind: "Bind provider to Case" };

export function ComputeSurface({ workspace, platform, actions }: Pick<SurfaceRendererProps, "workspace" | "platform" | "actions">) {
  const application = platform.application;
  useApplicationAvailability(application);
  const [yvexSetup, setYvexSetup] = useState(false);
  const [action, setAction] = useState<Action>();
  const [candidate, setCandidate] = useState<ProviderTarget>();
  const [modelsFound, setModelsFound] = useState<string[]>([]);
  const [discovering, setDiscovering] = useState(false);
  const [discoveryError, setDiscoveryError] = useState<string>();
  const [target, setTarget] = useState("");
  const [qualification, setQualification] = useState<ProviderQualification>();
  const [evidence, setEvidence] = useState<ProviderProbeEvidence>();
  const [fileError, setFileError] = useState<string>();
  const [trustReceipt, setTrustReceipt] = useState<{ target: string; posture: string }>();
  const operation = (name: Action) => ({ register: "provider.register", qualify: "provider.qualify", trust: "provider.trust.set", bind: "provider.case.bind" })[name];
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  const begin = (name: Action, ref?: string) => { if (ref) setTarget(ref); setEvidence(undefined); setFileError(undefined); setModelsFound([]); setDiscoveryError(undefined); setAction(name); };
  const controls = (ref?: string) => <div className="object-action-row">{(["qualify", "trust", "bind"] as const).map(name => <Button key={name} disabled={!application?.supports(operation(name))} onClick={() => begin(name, ref)}>{titles[name]}</Button>)}</div>;
  const models = [...new Set(workspace.compute.targets.map(item => item.model_id))];
  return <article className="live-page compute-surface" data-surface-type="case.compute">
    <header className="operational-heading"><div><h1>Compute</h1><p>Exact models, provider targets and Case bindings.</p></div><Button disabled={!workspace.case.tenant_ref || !application?.supports("provider.register")} onClick={() => { setYvexSetup(false); begin("register"); }}>Register provider target</Button></header>
    <section className="compute-section"><h2>Targets / deployments <span>{workspace.compute.targets.length}</span></h2>
      {!workspace.compute.targets.length && <EmptyState title="No bound target" body="Register a target, record measured qualification, set trust and bind it explicitly. Registration alone does not enable inference for this Case." />}
      {workspace.compute.targets.map(item => { const posture = typeof item.posture === "object" ? item.posture : undefined; return <article className="compute-target" key={item.id}><header><Icon name="compute" /><button className="object-link" onClick={() => actions.inspect(item.id)}>{item.provider_key}</button><Badge tone={posture?.trust?.posture === "approved" ? "success" : posture?.trust?.posture === "denied" ? "error" : "warning"}>{posture?.trust?.posture ?? "Unreviewed"}</Badge></header>
        <dl className="object-facts"><div><dt>Model</dt><dd>{item.model_id}</dd></div><div><dt>Provider adapter</dt><dd>{item.adapter.replaceAll("_", " ")}</dd></div><div><dt>Endpoint</dt><dd>{item.endpoint}</dd></div><div><dt>Locality</dt><dd>{item.locality.replaceAll("_", " ")}</dd></div><div><dt>Observed health</dt><dd>{posture?.health.posture ?? "Unavailable"}{posture?.health.observed_at_unix_ms ? ` · ${new Date(posture.health.observed_at_unix_ms).toLocaleString()}` : " · no current observation exposed"}</dd></div><div><dt>Qualification</dt><dd>{posture?.qualification ? posture.qualification.capabilities.map(value => value.capability).join(", ") || "No capability proven" : "Not exposed"}</dd></div></dl>{controls(item.id)}
        <details><summary>Exact target / evidence</summary><code>{item.id}</code>{posture?.qualification && <p>{posture.qualification.id} · run {posture.qualification.run_id}</p>}</details>
      </article>; })}
    </section>
    {candidate && candidate.tenant_id === workspace.case.tenant_ref && !workspace.compute.targets.some(item => item.id === candidate.target_id) && <section className="compute-section candidate-target"><h2>Registered in this session</h2><p><strong>{candidate.provider_key}</strong> · {candidate.model_id} · {candidate.endpoint}</p><code>{candidate.target_id}</code><p>Retained by YAI; not bound to this Case. The current Application projection lists only bound targets.</p>{controls(candidate.target_id)}</section>}
    {qualification && <p className="operation-receipt" role="status">Qualification recorded: {qualification.capabilities.map(item => item.capability).join(", ") || "no proven capability"} · {qualification.qualification_id}</p>}
    {trustReceipt && <p className="operation-receipt" role="status">Trust {trustReceipt.posture} recorded for {trustReceipt.target}. Binding and effect-time admission are separate.</p>}
    <section className="compute-section"><h2>Existing target</h2><p>Use an exact target reference returned by YAI. Unbound Tenant target discovery is not exposed by the current Application read contract.</p>{controls()}</section>
    <ConversationModelSetup workspace={workspace} platform={platform} />
    <section className="compute-section"><h2>Models & cognition</h2><p>{models.length ? `Models in this Case's bound targets: ${models.join(", ")}.` : "No bound model identity is exposed."} A model name is distinct from its provider/runtime and exact deployment.</p><p>Provider binding does not establish a cognitive-role binding. Work offers bounded execution and exact submission observation when advertised by the connected Host. Conversation sends committed text through the current primary cognitive assignment. Interrupted delivery is observed without automatic redispatch.</p></section>
    <section className="compute-section yvex-setup"><header className="operational-heading"><div><h2>YVEX</h2><p>Connect a running YVEX deployment to this Case.</p></div><Button disabled={!application?.supports("provider.register")} onClick={() => { setYvexSetup(true); begin("register"); }}>Configure YVEX</Button></header>
      <ol className="provider-setup-steps"><li><strong>Acquire & prepare</strong><span>Owned by the YVEX installation. Its local management protocol is not exposed by this remote HTTP connection.</span></li><li><strong>Load & discover</strong><span>Load the prepared model on YVEX; use “Discover exposed models” during registration to select its exact serving identity.</span></li><li><strong>Qualify & approve</strong><span>Import real measured probe evidence, approve trust and bind the exact target to this Case.</span></li><li><strong>Assign & converse</strong><span>Record your suitability assessment, assign the primary conversation role, then send messages from Conversation.</span></li></ol>
      <p>Model acquisition, preparation and engine loading need a qualified connection to the YVEX management plane. They are not available through OpenAI-compatible HTTP. Studio does not infer completion of these stages from a model name.</p>
    </section>
    {action && application && <ApplicationActionDialog key={action} title={titles[action]} description={action === "register" ? "Register an immutable target in the current Tenant. Enter a credential reference only, never a token. YAI validates the endpoint and ownership." : action === "qualify" ? "Import a measured ProviderProbeEvidence record from a real YAI probe. This records evidence; it does not run a network test. The current Application operation does not execute the probe." : action === "trust" ? "Set owner-authenticated trust for this exact target. Trust does not imply capability qualification or permission to execute." : "Bind one exact governed target to the current Participant. YAI checks Tenant ownership, Participant and exact target identity. Qualification and trust are checked again when execution is admitted. This replaces the Case provider envelope; it does not send a prompt."}
      submitLabel={action === "register" ? "Register target" : action === "qualify" ? "Record evidence" : action === "trust" ? "Record trust" : "Bind target"} close={() => setAction(undefined)}
      enabled={application.supports(operation(action)) && (action !== "qualify" || Boolean(evidence && evidence.target_id === target.trim()))}
      submit={async form => {
        if (action === "register") { const result = await application.registerProvider({ tenant_id: workspace.case.tenant_ref!, provider_key: String(form.get("key")).trim(), adapter: "open_ai_compatible", endpoint: String(form.get("endpoint")).trim(), model_id: String(form.get("model")).trim(), credential_ref: String(form.get("credential")).trim(), locality: String(form.get("locality")) as ProviderTarget["locality"], ...(form.get("extension") === "yvex.http.v1" ? { extension_adapter_id: "yvex.http.v1" as const } : {}) }); if (result.result_state === "success" && result.data) { setCandidate(result.data); setTarget(result.data.target_id); } return result; }
        if (action === "qualify") { const result = await application.qualifyProvider({ target_ref: target.trim(), evidence: evidence!, suite_ref: String(form.get("suite")).trim() }); if (result.result_state === "success" && result.data) setQualification(result.data); return result; }
        if (action === "trust") { const posture = String(form.get("posture")) as "approved" | "denied"; const result = await application.trustProvider({ target_ref: target.trim(), posture }); if (result.result_state === "success") setTrustReceipt({ target: target.trim(), posture }); return result; }
        return application.bindProvider({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, ordered_target_refs: [target.trim()], failover_policy: "none", max_attempts_per_turn: 1 });
      }} committed={refresh} resync={refresh}>
      {action === "register" ? <><label>Provider / runtime label<input autoFocus required name="key" placeholder="local-inference" /></label><label>Endpoint<input required type="url" name="endpoint" placeholder="http://127.0.0.1:8080" /></label><Button type="button" disabled={discovering || !application.supports("provider.models")} onClick={async event => {
        const form = event.currentTarget.closest("form"); if (!form) return;
        const values = new FormData(form); setDiscovering(true); setDiscoveryError(undefined); setModelsFound([]);
        const result = await application.discoverProviderModels({ tenant_id: workspace.case.tenant_ref!, endpoint: String(values.get("endpoint")).trim(), locality: String(values.get("locality")) as ProviderTarget["locality"], credential_ref: String(values.get("credential")).trim() });
        setDiscovering(false); if (result.result_state === "success" && result.data) setModelsFound(result.data.models); else setDiscoveryError(result.error?.safe_message ?? "Model discovery unavailable.");
      }}>{discovering ? "Discovering…" : "Discover exposed models"}</Button>{discoveryError && <p role="alert">{discoveryError}</p>}<label>Exact model identity<input required name="model" list="provider-discovered-models" placeholder="Model ID exposed by the server" /><datalist id="provider-discovered-models">{modelsFound.map(model => <option key={model} value={model} />)}</datalist></label>{modelsFound.length > 0 && <p role="status">{modelsFound.length} exposed model(s). Discovery does not qualify inference.</p>}<label>Locality<select name="locality"><option value="loopback">Loopback</option><option value="private_network">Private network</option><option value="remote">Remote</option></select></label><label>Credential reference<input required name="credential" defaultValue="none" /></label><label>Compatibility extension<select name="extension" defaultValue={yvexSetup ? "yvex.http.v1" : ""}><option value="">None</option><option value="yvex.http.v1">yvex.http.v1</option></select></label></> : <label>Exact target reference<input autoFocus required value={target} onChange={event => setTarget(event.target.value)} /></label>}
      {action === "qualify" && <><label>Measured evidence file<input type="file" accept="application/json,.json" required onChange={async event => { setEvidence(undefined); setFileError(undefined); const file = event.target.files?.[0]; if (!file) return; if (file.size > 65536) { setFileError("Evidence must be at most 64 KiB."); return; } try { const value = readProbeEvidence(JSON.parse(await file.text())); if (!value) throw new Error(); setEvidence(value); } catch { setFileError("This is not a valid ProviderProbeEvidence record."); } }} /></label><label>Qualification suite reference<input required name="suite" placeholder="Exact suite reference associated with the probe" /></label>{fileError && <p role="alert">{fileError}</p>}{evidence && <p>Run {evidence.run_id} · {evidence.failure_codes.length} reported failures{evidence.target_id !== target.trim() && " · target mismatch: not submitted"}</p>}</>}
      {action === "trust" && <label>Trust decision<select name="posture"><option value="approved">Approve</option><option value="denied">Deny</option></select></label>}
      {action === "bind" && <p>Participant: {workspace.case.participant_ref}. One target, no failover, one attempt. Existing ordered bindings will be replaced.</p>}
    </ApplicationActionDialog>}
  </article>;
}
