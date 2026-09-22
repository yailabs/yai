import { useRef, useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { CasePresentation } from "../../clients/dataSource";
import type { PolicyArtifactView, PolicyLifecycleAction, PolicyRule } from "../../clients/policy";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button, PanelHeader } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";
import { PolicyActions } from "./PolicyActions";

const maxSourceBytes = 256 * 1024;
const labels: Record<PolicyLifecycleAction, string> = { validate: "Validate candidate", publish: "Publish policy", retire: "Retire policy", revoke: "Revoke policy" };

/** An explicit upload is sent unchanged to the existing YAI policy compiler.
 * No client classification, compilation, publication or policy simulation.
 * Returned views are transient interaction results, never a policy catalog.
 */
export function PolicyIntake({ application, workspace, refresh }: {
  application?: ApplicationAccess; workspace: CasePresentation; refresh(): void | Promise<void>;
}) {
  useApplicationAvailability(application);
  const [open, setOpen] = useState(false);
  const [views, setViews] = useState<PolicyArtifactView[]>([]);
  const [selected, setSelected] = useState<string>();
  const [dragging, setDragging] = useState(false);
  const [dropped, setDropped] = useState<File[]>([]);
  const [dropError, setDropError] = useState("");
  const available = Boolean(workspace.case.tenant_ref && application?.supports("policy.ingest"));
  const remember = (view: PolicyArtifactView) => {
    setViews(current => [...current.filter(item => item.artifact.artifact_id !== view.artifact.artifact_id), view]);
    setSelected(view.artifact.artifact_id);
  };
  const view = views.find(item => item.artifact.artifact_id === selected);
  return <section className={`policy-intake${dragging ? " is-dragging" : ""}`} aria-label="Policy intake"
    onDragOver={event => { if (event.dataTransfer.types.includes("Files")) { event.preventDefault(); event.dataTransfer.dropEffect = available ? "copy" : "none"; setDragging(available); } }}
    onDragLeave={event => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setDragging(false); }}
    onDrop={event => {
      event.preventDefault(); setDragging(false); if (!available) return;
      if ([...event.dataTransfer.items].some(item => item.webkitGetAsEntry?.()?.isDirectory)) {
        setDropError("Folder intake is not exposed by the connected Application. Select exact policy documents; Studio will not scan the folder or acquire its other files."); return;
      }
      const files = [...event.dataTransfer.files]; if (files.length) { setDropped(files); setDropError(""); setOpen(true); }
    }}>
    <PanelHeader title="Policy documents" detail="Original → candidate → publication → Case binding" actions={<Button disabled={!available} title={application && !available ? application.reason("policy.ingest") : undefined} onClick={() => { setDropped([]); setOpen(true); }}>Import policy documents</Button>} />
    <p>Drop exact policy documents here. YAI extracts and validates typed rules; importing does not activate them in this Case.</p>
    <p className="surface-note">Current profiles: policy JSON, an explicit policy block in Markdown, and supported text-PDF. Ordinary prose and TOML are not automatically interpreted as policy.</p>
    {dropError && <p role="alert" className="policy-refusal">{dropError}</p>}
    {views.length > 0 && <><label className="policy-result-picker">Imported artifact<select value={selected} onChange={event => setSelected(event.target.value)}>{views.map(item => <option key={item.artifact.artifact_id} value={item.artifact.artifact_id}>{item.artifact.policy_key} · {item.artifact.artifact_version} · {item.lifecycle}</option>)}</select></label><p className="surface-note">Results returned in this view. Reopening the view requires reselecting the exact document; the connected Application has no policy catalog read operation.</p></>}
    {view && application && <PolicyResult key={view.artifact.artifact_id} application={application} workspace={workspace} view={view} update={remember} refresh={refresh} />}
    {open && application && <ImportPolicyDialog application={application} tenant={workspace.case.tenant_ref!} initialFiles={dropped} received={remember} close={() => setOpen(false)} />}
  </section>;
}

function ImportPolicyDialog({ application, tenant, initialFiles, received, close }: {
  application: ApplicationAccess; tenant: string; initialFiles: File[]; received(view: PolicyArtifactView): void; close(): void;
}) {
  const [files, setFiles] = useState(initialFiles);
  const completed = useRef(new Set<File>());
  const [accepted, setAccepted] = useState(0);
  const error = files.length > 16 ? "Select at most 16 exact policy documents per import." : files.some(file => !file.size || file.size > maxSourceBytes) ? "Each policy document must contain 1–262,144 bytes, within the current compiler bound." : "";
  return <ApplicationActionDialog title="Import policy documents" description="The original bytes go to the authenticated YAI policy compiler. A successful import retains a Tenant-scoped candidate, without binding it to this Case. YAI can refuse an unsupported or ambiguous document." submitLabel="Import candidates" close={close} committed={() => undefined} enabled={files.length > 0 && !error && application.supports("policy.ingest")} submit={async () => {
    let last;
    for (const file of files) {
      if (completed.current.has(file)) continue;
      const source_bytes = [...new Uint8Array(await file.arrayBuffer())];
      last = await application.ingestPolicy({ tenant_id: tenant, source_bytes });
      if (last.result_state !== "success" || !last.data) return last;
      received(last.data.view); completed.current.add(file); setAccepted(completed.current.size);
    }
    return last ?? { operation_ref: "policy.ingest", correlation_ref: "studio:already-confirmed", result_state: "success" };
  }}>
    <label>Policy documents<input type="file" multiple autoFocus onChange={event => { setFiles([...event.target.files ?? []]); completed.current.clear(); setAccepted(0); }} /></label>
    <ul className="policy-upload-list">{files.map((file, index) => <li key={index}><span>{file.name}</span><small>{file.size.toLocaleString()} bytes{completed.current.has(file) ? " · imported" : ""}</small></li>)}</ul>
    {error && <p role="alert">{error}</p>}
    {accepted > 0 && <p>{accepted} candidate(s) confirmed. A later refusal does not undo earlier imports.</p>}
    <small>JSON is a carrier, not a policy role: the backend must recognize the supported policy schema. No publication or binding runs automatically.</small>
  </ApplicationActionDialog>;
}

function PolicyResult({ application, workspace, view, update, refresh }: {
  application: ApplicationAccess; workspace: CasePresentation; view: PolicyArtifactView; update(view: PolicyArtifactView): void; refresh(): void | Promise<void>;
}) {
  const [action, setAction] = useState<PolicyLifecycleAction>();
  const artifact = view.artifact;
  const bindings = workspace.authority.policies.filter(item => item.artifact_ref === artifact.artifact_id);
  const offered: PolicyLifecycleAction[] = view.lifecycle === "candidate" ? ["validate", "revoke"] : view.lifecycle === "validated" ? ["publish", "revoke"] : view.lifecycle === "published" ? ["retire", "revoke"] : view.lifecycle === "superseded" || view.lifecycle === "retired" ? ["revoke"] : [];
  return <article className="policy-result" aria-label={`Policy ${artifact.policy_key}`}>
    <header><div><small>Typed policy · version {artifact.artifact_version}</small><h2>{artifact.policy_key}</h2></div><Badge tone={view.lifecycle === "revoked" ? "error" : view.runtime_consumable ? "success" : "warning"}>{view.lifecycle}</Badge></header>
    <p>{bindings.length ? "Bound to this Case in the current projection. YAI evaluates current validity and all applicable policy at admission." : "Not bound to this Case. Publishing an artifact alone grants no Case authority."}</p>
    <div className="policy-lifecycle" aria-label="Policy lifecycle">{["candidate", "validated", "published"].map(step => <span key={step} aria-current={view.lifecycle === step ? "step" : undefined}>{step}</span>)}<span aria-current={bindings.length ? "step" : undefined}>Case binding</span></div>
    <div className="object-action-row">{offered.map(item => <Button key={item} disabled={!application.supports(`policy.${item}`)} onClick={() => setAction(item)}>{labels[item]}</Button>)}</div>
    {view.lifecycle === "published" && !bindings.length && <PolicyActions application={application} workspace={workspace} artifactRef={artifact.artifact_id} refresh={refresh} />}
    <PanelHeader title="Rules returned by YAI" detail={`${artifact.policy_ir.rules.length} rules · validation ${artifact.validation.status}`} />
    {artifact.validation.blockers.length > 0 && <div className="policy-refusal" role="status"><strong>Validation blockers</strong><ul>{artifact.validation.blockers.map((item, i) => <li key={i}>{item}</li>)}</ul></div>}
    <div className="policy-rules">{artifact.policy_ir.rules.map(rule => <PolicyRuleRow key={rule.rule_id} rule={rule} />)}</div>
    {artifact.policy_ir.conflicts.length > 0 && <section><h3>Conflicts reported by YAI</h3>{artifact.policy_ir.conflicts.map((conflict, i) => <p key={i}>{conflict.code} · {conflict.selector} · {conflict.rule_refs.join(", ")}</p>)}</section>}
    {artifact.policy_ir.unresolved.length > 0 && <p className="policy-refusal">{artifact.policy_ir.unresolved.length} unresolved item(s). Unresolved content does not become an enforceable rule.</p>}
    <details className="policy-technical"><summary>Provenance and lifecycle</summary><dl className="object-facts"><div><dt>Artifact</dt><dd>{artifact.artifact_id}</dd></div><div><dt>Source</dt><dd>{artifact.source_id}</dd></div><div><dt>Digest</dt><dd>{artifact.source_digest}</dd></div><div><dt>Owner</dt><dd>{artifact.owner_ref}</dd></div><div><dt>Validity</dt><dd>{artifact.validity.mode}{artifact.validity.valid_from_unix_ms != null && ` · from ${new Date(artifact.validity.valid_from_unix_ms).toISOString()}`}{artifact.validity.expires_at_unix_ms != null && ` · until ${new Date(artifact.validity.expires_at_unix_ms).toISOString()}`}</dd></div></dl><ol>{view.lifecycle_events.map(event => <li key={event.event_id}><strong>{event.action}</strong> · {event.reason}<small>{new Date(event.committed_at_unix_ms).toLocaleString()} · {event.actor_ref}</small></li>)}</ol></details>
    <p className="surface-note">This is the compiler’s last returned representation, not a policy simulation. A current EffectivePolicy explanation or what-if test requires an authorized Application projection; Studio does not evaluate combined rules.</p>
    {action && <ApplicationActionDialog title={labels[action]} description={`${artifact.policy_key}: ${action === "publish" ? "publish this validated artifact. Case binding is a separate action." : action === "validate" ? "ask YAI to validate this candidate." : "change the Tenant policy artifact lifecycle. This can affect every Case using it; it does not delete the original or history."}`} submitLabel={labels[action]} close={() => setAction(undefined)} enabled={application.supports(`policy.${action}`)} submit={async form => {
      const result = await application.policyLifecycle(action, { artifact_ref: artifact.artifact_id, reason: String(form.get("reason")).trim() });
      if (result.result_state === "success" && result.data) update(result.data.view);
      return result;
    }} committed={refresh}><label>Reason<textarea name="reason" autoFocus required rows={3} /></label><small>YAI verifies current lifecycle and Tenant authority before accepting this action.</small></ApplicationActionDialog>}
  </article>;
}

function PolicyRuleRow({ rule }: { rule: PolicyRule }) {
  const title = rule.kind === "operation_restriction" ? rule.effect === "deny" ? "Deny" : "Allow"
    : rule.kind === "review_requirement" ? rule.required ? "Review required" : "Review not required"
    : rule.kind === "authority_requirement" ? `${rule.subject === "reviewer" ? "Reviewer" : "Proposer"} role required`
    : "Evidence required";
  return <section className="policy-rule"><header><Badge tone={rule.effect === "deny" ? "error" : rule.effect === "allow" ? "success" : "warning"}>{title}</Badge><strong>{rule.operation_kind}</strong>{rule.resource_kind && <span>{rule.resource_kind}</span>}</header>
    {rule.required_role && <p>Role: <strong>{rule.required_role}</strong></p>}{rule.obligation && <p>Evidence: {rule.obligation.replaceAll("_", " ")}</p>}<p>{rule.reason}</p>
    <details><summary>Rule provenance</summary><code>{rule.rule_id}</code><p>{rule.provenance.source_locations.join(" · ")}</p></details>
  </section>;
}
