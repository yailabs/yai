import { useEffect, useRef, useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import type { HandoffInspection, HandoffOffer } from "../../clients/work";
import { Badge, Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

type Action = "offer" | "accept" | "decline" | "result" | "reconcile";
const labels = { offer: "Offer Handoff", accept: "Accept Handoff", decline: "Decline Handoff", result: "Record Handoff result", reconcile: "Reconcile Handoff" };
export function HandoffActions({ workspace, platform }: Pick<SurfaceRendererProps, "workspace" | "platform">) {
  const application = platform.application; const availability = useApplicationAvailability(application);
  const [revision, setRevision] = useState(0); const [shown, setShown] = useState(32);
  const context = JSON.stringify([workspace.case.case_ref, workspace.case.participant_ref]);
  const identity = JSON.stringify([context, workspace.case.generation, revision, availability]);
  const active = useRef(identity); active.current = identity;
  const [inbox, setInbox] = useState<{ identity: string; offers: HandoffOffer[]; error?: string }>();
  const [selection, setSelection] = useState<{ context: string; ref: string }>();
  const [inspection, setInspection] = useState<{ identity: string; ref: string; value?: HandoffInspection; error?: string }>();
  const selected = selection?.context === context ? selection.ref : undefined;
  const current = inbox?.identity === identity ? inbox : undefined;
  const detail = inspection?.identity === identity && inspection.ref === selected ? inspection : undefined;
  const choose = (ref: string) => setSelection({ context, ref });
  useEffect(() => {
    if (!application?.supports("handoff.pending")) return;
    let disposed = false; const stamp = identity;
    void application.pendingHandoffs({ case_ref: workspace.case.case_ref }).then(response => {
      if (disposed || active.current !== stamp) return; const value = response.data;
      if (response.result_state === "success" && value?.schema === "yai.handoff_pending_projection.v1" && value.case_ref === workspace.case.case_ref && value.generation === workspace.case.generation && Array.isArray(value.offers) && value.offers.every(offer => offer.target_case_id === workspace.case.case_ref && typeof offer.handoff_id === "string")) setInbox({ identity: stamp, offers: value.offers });
      else setInbox({ identity: stamp, offers: [], error: response.error?.safe_message ?? "Handoff inbox changed. Refresh the Case." });
    }).catch(() => { if (!disposed && active.current === stamp) setInbox({ identity: stamp, offers: [], error: "Handoff inbox is unavailable." }); });
    return () => { disposed = true; };
  }, [application, identity]);
  useEffect(() => {
    if (!selected || !application?.supports("handoff.inspect")) return;
    let disposed = false; const stamp = identity; const ref = selected;
    void application.inspectHandoff({ case_ref: workspace.case.case_ref, handoff_ref: ref }).then(response => {
      if (disposed || active.current !== stamp) return; const value = response.data;
      if (response.result_state === "success" && value?.schema === "yai.handoff_inspection.v1" && value.case_ref === workspace.case.case_ref && value.generation === workspace.case.generation && value.offer.handoff_id === ref && [value.offer.source_case_id, value.offer.target_case_id].includes(workspace.case.case_ref)) setInspection({ identity: stamp, ref, value });
      else setInspection({ identity: stamp, ref, error: response.error?.safe_message ?? "No authorized detail for this exact Handoff." });
    }).catch(() => { if (!disposed && active.current === stamp) setInspection({ identity: stamp, ref, error: "Handoff observation is unavailable." }); });
    return () => { disposed = true; };
  }, [application, identity, selected]);
  const [action, setAction] = useState<Action>();
  const [receipt, setReceipt] = useState<{ identity: string; kind: string; generation: number }>();
  const operation = (value: Action) => value === "result" ? "handoff.result.record" : `handoff.${value}`;
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  return <section className="work-section"><h2>Handoffs</h2><p>Bounded work information between Cases in the same Tenant. Resources, roles and authority are not transferred.</p>
    <header className="handoff-inbox-toolbar"><h3>Incoming</h3><Button disabled={!application?.supports("handoff.pending")} onClick={() => setRevision(value => value + 1)}>Refresh Handoffs</Button></header>
    {application?.supports("handoff.pending") ? !current ? <p role="status">Reading incoming offers…</p> : current.error ? <p role="alert">{current.error}</p> : <>
      <div className="handoff-inbox" aria-label="Incoming Handoffs">{current.offers.slice(0, shown).map(offer => <button type="button" className="handoff-inbox-row" key={offer.handoff_id} aria-pressed={selected === offer.handoff_id} onClick={() => choose(offer.handoff_id)}><strong>{offer.source_case_id}</strong><span>{offer.request.value}</span><small>Awaiting response</small></button>)}</div>
      {shown < current.offers.length && <Button onClick={() => setShown(value => value + 32)}>Show 32 more offers</Button>}
      {!current.offers.length && <p>No pending offers addressed to this Case.</p>}
      <small>{Math.min(current.offers.length, shown)} of {current.offers.length} offers returned by YAI. Bounded authorized Tenant scan; accepted items are not pending.</small>
    </> : <p>This Host does not expose a Handoff inbox.</p>}
    <details className="handoff-exact-inspection"><summary>Inspect by exact reference</summary><form className="handoff-inspect-form" onSubmit={event => { event.preventDefault(); choose(String(new FormData(event.currentTarget).get("reference")).trim()); }}><label>Inspect retained Handoff<input name="reference" required placeholder="Exact Handoff reference" /></label><Button type="submit" disabled={!application?.supports("handoff.inspect")}>Inspect Handoff</Button></form></details>
    {selected && (!detail ? <p role="status">Reading selected Handoff…</p> : detail.error ? <p role="alert">{detail.error}</p> : detail.value && <section className="handoff-detail" aria-label="Selected Handoff">
      <header><h3>Selected Handoff</h3><Badge>{detail.value.reconciliation ? "Reconciled" : detail.value.result ? "Result recorded" : detail.value.decline ? "Declined" : detail.value.acceptance ? "Accepted" : "Offered"}</Badge></header>
      <dl className="object-facts"><div><dt>From</dt><dd>{detail.value.offer.source_case_id}</dd></div><div><dt>To</dt><dd>{detail.value.offer.target_case_id}</dd></div><div><dt>Required target roles</dt><dd>{detail.value.offer.required_target_roles.join(", ") || "None declared"}</dd></div></dl>
      <h4>Request</h4><pre>{detail.value.offer.request.value}</pre>
      {detail.value.decline && <p>Decline reason: {detail.value.decline.reason}</p>}
      {detail.value.result && <><h4>Result · {detail.value.result.outcome}</h4><pre>{detail.value.result.result.value}</pre><details><summary>Evidence references</summary>{detail.value.result.evidence_refs.map(ref => <code key={ref}>{ref}</code>)}</details></>}
      {detail.value.reconciliation && <><h4>Reconciled outcome · {detail.value.reconciliation.outcome}</h4>{detail.value.reconciliation.result && <pre>{detail.value.reconciliation.result.value}</pre>}</>}
      <details><summary>Exact identity</summary><code>{selected}</code></details><small>Current Case-local protocol facts. Viewing an offer does not import the target result or transfer authority.</small>
    </section>)}
    <div className="object-action-row">{(["offer", "accept", "decline", "result", "reconcile"] as const).map(value => <Button key={value} disabled={!application?.supports(operation(value))} onClick={() => setAction(value)}>{labels[value]}</Button>)}</div>
    {receipt && <p className="operation-receipt" role="status">{receipt.kind.replaceAll("_", " ")} · Case state version {receipt.generation} · <code>{receipt.identity}</code></p>}
    <p className="surface-note">Select an incoming offer to fill its exact references below. Retained items can be inspected by reference. The manual offer contract provides one offer slot per source Case.</p>
    {action && application && <ApplicationActionDialog title={labels[action]} description={action === "offer" ? "Create a retained offer from this Case to an exact same-Tenant Case. YAI checks current lifecycle and authority; acceptance by the target is separate." : action === "reconcile" ? "Reconcile the target's authoritative result or decline back into this source Case. This does not execute an effect." : `Act as ${workspace.case.participant_ref} in this target Case. YAI validates the exact offer, target, required roles and current state.`} submitLabel={action === "result" ? "Record result" : action[0].toUpperCase() + action.slice(1)} close={() => setAction(undefined)} enabled={application.supports(operation(action))} submit={async form => {
      const handoff_ref = String(form.get("handoff") ?? "").trim();
      const target_case_ref = workspace.case.case_ref; const participant_ref = workspace.case.participant_ref;
      const source_case_ref = String(form.get("source") ?? "").trim();
      const result = action === "offer" ? await application.offerHandoff({ source_case_ref: workspace.case.case_ref, target_case_ref: String(form.get("target")).trim(), request: { kind: "text", value: String(form.get("request")) }, required_target_roles: String(form.get("roles")).split(",").map(value => value.trim()).filter(Boolean) })
        : action === "accept" ? await application.acceptHandoff({ target_case_ref, source_case_ref, handoff_ref, participant_ref })
        : action === "decline" ? await application.declineHandoff({ target_case_ref, source_case_ref, handoff_ref, participant_ref, reason: String(form.get("reason")) })
        : action === "result" ? await application.resultHandoff({ target_case_ref, handoff_ref, participant_ref, outcome: String(form.get("outcome")) as "succeeded" | "failed" | "cancelled", result: { kind: "text", value: String(form.get("result")) }, evidence_refs: String(form.get("evidence")).split(/\r?\n/).map(value => value.trim()).filter(Boolean) })
        : await application.reconcileHandoff({ source_case_ref: workspace.case.case_ref, handoff_ref });
      if (result.result_state === "success" && result.data) { const payload = result.data.transition.payload; setReceipt({ identity: payload.data.offer?.handoff_id ?? handoff_ref, kind: payload.kind, generation: result.data.state.generation }); }
      return result;
    }} committed={refresh} resync={refresh}>
      {action === "offer" ? <><label>Target Case reference<input name="target" required autoFocus placeholder="case:target" /></label><label>Request<textarea name="request" rows={5} required /></label><label>Required target roles, comma separated<input name="roles" /></label><small>At most 16 KiB of text. YAI enforces bounds and the same-Tenant condition.</small></> : <label>Handoff reference<input name="handoff" required autoFocus defaultValue={detail?.value?.offer.handoff_id ?? receipt?.identity} /></label>}
      {(action === "accept" || action === "decline") && <label>Source Case reference<input name="source" required placeholder="case:source" defaultValue={detail?.value?.offer.source_case_id} /></label>}
      {action === "decline" && <label>Reason<textarea name="reason" required rows={3} /></label>}
      {action === "result" && <><label>Outcome<select name="outcome"><option value="succeeded">Succeeded</option><option value="failed">Failed</option><option value="cancelled">Cancelled</option></select></label><label>Result<textarea name="result" required rows={4} /></label><label>Evidence references, one per line<textarea name="evidence" rows={3} /></label></>}
    </ApplicationActionDialog>}
  </section>;
}
