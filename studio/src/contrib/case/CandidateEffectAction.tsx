import { useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { ControlledOperation, EffectProposeInput } from "../../clients/execution";
import { executionKey, rememberExecution } from "../../clients/execution";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";
import { ExecutionReceipt } from "./ExecutionActions";

type Props = Pick<SurfaceRendererProps, "workspace" | "platform"> & { candidateRef: string };
export function CandidateEffectAction(props: Props) {
  return <CandidateEffectBody key={`${props.workspace.case.case_ref}:${props.workspace.case.participant_ref}:${props.candidateRef}`} {...props} />;
}
function CandidateEffectBody({ workspace, platform, candidateRef }: Props) {
  const application = platform.application;
  const { case_ref, participant_ref, generation } = workspace.case;
  const resources = workspace.environment.resources.filter(item => ["filesystem", "process"].includes(item.kind));
  const [input, setInput] = useState<EffectProposeInput>();
  const [proposing, setProposing] = useState(false);
  const [operation, setOperation] = useState<ControlledOperation>();
  const [refusal, setRefusal] = useState<string>();
  const [submitGeneration, setSubmitGeneration] = useState<number>();
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  if (!application || !resources.length) return null;
  const recover = Boolean(input && !operation && !refusal);
  return <section className="candidate-effect" aria-label="Candidate Resource action">
    <Button disabled={!application.supports("effect.propose")} onClick={() => { setInput(undefined); setRefusal(undefined); setProposing(true); }}>Prepare Resource action…</Button>
    {recover && !proposing && <Button onClick={() => setProposing(true)}>Recover exact proposal…</Button>}
    {refusal && <p role="alert">{refusal} No effect was submitted.</p>}
    {proposing && <ApplicationActionDialog title="Prepare Resource action" description="Ask YAI to normalize this exact retained model response for a bound Resource. This may record an Operation or a normalization refusal. It does not execute the action or grant permission." submitLabel={input ? "Recover exact proposal" : "Record proposal"} close={() => setProposing(false)} committed={refresh} resync={refresh} submit={async form => {
      const exact = input ?? { case_ref, participant_ref, candidate_ref: candidateRef, resource_ref: String(form.get("resource")), expected_generation: generation };
      setInput(exact); setOperation(undefined); setRefusal(undefined);
      const response = await application.proposeEffect(exact);
      if (response.result_state === "success" && response.data) {
        if (response.data.posture === "normalization_refused") setRefusal(response.data.failure.detail);
        else {
          const value = response.data.operation;
          if (value.case_id !== case_ref || value.participant_id !== participant_ref || value.resource_attachment_id !== exact.resource_ref || value.origin.provider_result_id !== candidateRef) throw Error("Operation identity mismatch");
          rememberExecution(executionKey(case_ref, participant_ref), { domain: "controlled_effect", operation_ref: value.operation_id });
          setOperation(value);
        }
      }
      return response;
    }}><label>Bound Resource<select name="resource" disabled={Boolean(input)} defaultValue={input?.resource_ref}>{resources.map(item => <option key={item.id} value={item.id}>{item.label ?? item.id} · {item.kind}</option>)}</select></label><p>YAI interprets the original response. Studio does not edit or execute its contents.</p><details><summary>Candidate identity</summary><code>{candidateRef}</code></details></ApplicationActionDialog>}
    {operation && <section aria-label="Recorded Resource action">
      <h4>{operation.kind === "filesystem_write" ? "Recorded file write" : "Recorded process action"}</h4>
      <p>Resource: {resources.find(item => item.id === operation.resource_attachment_id)?.label ?? operation.resource_attachment_id}</p>
      {operation.kind === "filesystem_write" ? <><p>{operation.filesystem_write.relative_path} · {operation.filesystem_write.content_bytes} bytes</p><details><summary>Exact proposed content</summary><pre>{operation.filesystem_write.content}</pre></details></> : <p>Action: {operation.process_signal?.action} · exact bound process identity</p>}
      <Button disabled={!application.supports("effect.submit")} onClick={() => setSubmitGeneration(generation)}>Submit recorded Operation…</Button>
      <ExecutionReceipt workspace={workspace} platform={platform} reference={{ domain: "controlled_effect", operation_ref: operation.operation_id }} />
    </section>}
    {operation && submitGeneration !== undefined && <ApplicationActionDialog title="Submit recorded Operation" description="This may perform the exact external effect shown above. YAI rechecks current policy, Review, Grant, Resource and effect-time fences. If PREPARE already exists, the same reference is observed rather than dispatched again." submitLabel="Submit exact Operation" close={() => setSubmitGeneration(undefined)} enabled={generation === submitGeneration} committed={refresh} resync={refresh} submit={() => {
      rememberExecution(executionKey(case_ref, participant_ref), { domain: "controlled_effect", operation_ref: operation.operation_id });
      return application.submitEffect({ case_ref, participant_ref, operation_ref: operation.operation_id, expected_generation: submitGeneration });
    }}><p>{operation.kind.replaceAll("_", " ")} · {operation.resource_attachment_id}</p>{operation.kind === "filesystem_write" && <p>{operation.filesystem_write.relative_path} · {operation.filesystem_write.content_bytes} bytes</p>}<details><summary>Exact Operation</summary><pre>{JSON.stringify(operation, null, 2)}</pre></details>{generation !== submitGeneration && <p role="alert">The Case changed. Close and inspect the current state before submitting.</p>}</ApplicationActionDialog>}
  </section>;
}
