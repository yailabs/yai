import { useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button } from "../../components/primitives";
import type { SemanticEvidence } from "../../clients/compute";

export function ConversationModelSetup({ workspace, platform }: Pick<SurfaceRendererProps, "workspace" | "platform">) {
  const application = platform.application;
  const [action, setAction] = useState<"attest" | "bind">();
  const [target, setTarget] = useState("");
  const [evidence, setEvidence] = useState<SemanticEvidence>();
  const current = workspace.compute.cognitive_bindings?.find(item => item.participant_id === workspace.case.participant_ref && item.role === "primary");
  const selected = workspace.compute.targets.find(item => item.id === target);
  const evidenceRows = [...(selected?.semantic_evidence ?? []), ...(evidence?.target_id === target && !selected?.semantic_evidence?.some(item => item.evidence_id === evidence.evidence_id) ? [evidence] : [])];
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  return <section className="compute-section conversation-model-setup"><h2>Conversation model <Badge tone={current ? "success" : "warning"}>{current ? "Assigned" : "Setup required"}</Badge></h2>
    <p>{current ? workspace.compute.targets.find(item => item.id === current.target_id)?.model_id ?? current.target_id : "After qualification, trust and Case binding, assign a target to your Participant’s primary conversation role."}</p>
    <p>Suitability is your explicit assessment of this model for conversation. It is separate from the measured API contract and grants no Resource authority.</p>
    <div className="object-action-row"><Button disabled={!workspace.compute.targets.length || !application?.supports("provider.suitability.record")} onClick={() => { setTarget(workspace.compute.targets[0]?.id ?? ""); setAction("attest"); }}>Attest conversation suitability</Button><Button disabled={!workspace.compute.targets.length || !application?.supports("cognitive.binding.set")} onClick={() => { setTarget(current?.target_id ?? workspace.compute.targets[0]?.id ?? ""); setAction("bind"); }}>{current ? "Change conversation model" : "Assign conversation model"}</Button></div>
    {current && <details><summary>Current assignment</summary><code>{current.binding_id}</code><p>Participant: {current.participant_id}</p><code>{current.semantic_evidence_id}</code></details>}
    {action && application && <ApplicationActionDialog title={action === "attest" ? "Attest conversation suitability" : "Assign conversation model"} description={action === "attest" ? "Record your own assessment of this exact model. This is operator-attested suitability, not a model-quality benchmark or permission to execute." : "YAI validates current mechanical qualification, trust, Case provider binding and exact semantic evidence. Each subsequent message is admitted again."} close={() => setAction(undefined)} submitLabel={action === "attest" ? "Record my attestation" : current ? "Replace assignment" : "Assign model"} enabled={application.supports(action === "attest" ? "provider.suitability.record" : "cognitive.binding.set") && Boolean(selected) && (action === "attest" || evidenceRows.length > 0)} committed={refresh} resync={refresh} submit={async form => {
      if (action === "attest") {
        const result = await application.attestProvider({ target_ref: target, capability: "primary_conversation", suite_ref: "studio.operator.conversation.v1", run_ref: `studio-attestation:${crypto.randomUUID()}`, evidence_refs: [String(form.get("basis")).trim()] });
        if (result.result_state === "success" && result.data) setEvidence(result.data);
        return result;
      }
      return application.bindCognition({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, role: "primary", capability: "primary_conversation", candidates: [{ target_ref: target, semantic_evidence_ref: String(form.get("evidence")) }], replace: Boolean(current) });
    }}>
      <label>Case provider target<select value={target} onChange={event => setTarget(event.target.value)}>{workspace.compute.targets.map(item => <option value={item.id} key={item.id}>{item.provider_key} · {item.model_id}</option>)}</select></label>
      {action === "attest" ? <><label>Assessment / evidence reference<input required name="basis" placeholder="operator-assessment:local-model-conversation" /></label><label className="confirmation"><input required type="checkbox" />I attest this exact target’s suitability for primary conversation.</label></> : <><label>Recorded suitability<select required name="evidence" key={target}>{evidenceRows.map(item => <option key={item.evidence_id} value={item.evidence_id}>{item.posture} · {item.evidence_id}</option>)}</select></label>{!evidenceRows.length && <p>Record a suitability attestation for this target first.</p>}{current && <p>This explicitly replaces the current primary assignment.</p>}</>}
    </ApplicationActionDialog>}
  </section>;
}
