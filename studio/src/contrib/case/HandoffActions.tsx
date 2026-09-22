import { useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

type Action = "offer" | "accept" | "decline" | "result" | "reconcile";
const labels = { offer: "Offer Handoff", accept: "Accept Handoff", decline: "Decline Handoff", result: "Record Handoff result", reconcile: "Reconcile Handoff" };
export function HandoffActions({ workspace, platform }: Pick<SurfaceRendererProps, "workspace" | "platform">) {
  const application = platform.application; useApplicationAvailability(application);
  const [action, setAction] = useState<Action>();
  const [receipt, setReceipt] = useState<{ identity: string; kind: string; generation: number }>();
  const operation = (value: Action) => value === "result" ? "handoff.result.record" : `handoff.${value}`;
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  return <section className="work-section"><h2>Handoffs</h2><p>Bounded work information between Cases in the same Tenant. Resources, roles and authority are not transferred.</p>
    <div className="object-action-row">{(["offer", "accept", "decline", "result", "reconcile"] as const).map(value => <Button key={value} disabled={!application?.supports(operation(value))} onClick={() => setAction(value)}>{labels[value]}</Button>)}</div>
    {receipt && <p className="operation-receipt" role="status">{receipt.kind.replaceAll("_", " ")} · generation {receipt.generation} · <code>{receipt.identity}</code></p>}
    <p className="surface-note">The current summary does not expose a Handoff inbox or payload history. These actions use exact references returned by YAI. The manual offer contract provides one offer slot per source Case.</p>
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
      {action === "offer" ? <><label>Target Case reference<input name="target" required autoFocus placeholder="case:target" /></label><label>Request<textarea name="request" rows={5} required /></label><label>Required target roles, comma separated<input name="roles" /></label><small>At most 16 KiB of text. YAI enforces bounds and the same-Tenant condition.</small></> : <label>Handoff reference<input name="handoff" required autoFocus defaultValue={receipt?.identity} /></label>}
      {(action === "accept" || action === "decline") && <label>Source Case reference<input name="source" required placeholder="case:source" /></label>}
      {action === "decline" && <label>Reason<textarea name="reason" required rows={3} /></label>}
      {action === "result" && <><label>Outcome<select name="outcome"><option value="succeeded">Succeeded</option><option value="failed">Failed</option><option value="cancelled">Cancelled</option></select></label><label>Result<textarea name="result" required rows={4} /></label><label>Evidence references, one per line<textarea name="evidence" rows={3} /></label></>}
    </ApplicationActionDialog>}
  </section>;
}
