import { useEffect, useRef, useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { CognitiveCapability, CognitivePlan, CognitivePrepareInput } from "../../clients/cognitive";
import { executionKey, readExecutionRefs, rememberExecution } from "../../clients/execution";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button } from "../../components/primitives";
import { ExecutionReceipt } from "./ExecutionActions";
import { useApplicationAvailability } from "./applicationActions";

type Props = Pick<SurfaceRendererProps, "workspace" | "platform">;
export function CognitiveExecution(props: Props) {
  return <CognitiveExecutionBody key={`${props.workspace.case.case_ref}:${props.workspace.case.participant_ref}`} {...props} />;
}

function CognitiveExecutionBody({ workspace, platform }: Props) {
  const application = platform.application;
  useApplicationAvailability(application);
  const { case_ref, participant_ref, generation } = workspace.case;
  const key = executionKey(case_ref, participant_ref);
  const [turn, setTurn] = useState("");
  const [capability, setCapability] = useState<CognitiveCapability>("primary_conversation");
  const [prepared, setPrepared] = useState<{ identity: string; input: CognitivePrepareInput; plan: CognitivePlan }>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [confirm, setConfirm] = useState(false);
  const [refs, setRefs] = useState(() => readExecutionRefs(key));
  const identity = JSON.stringify([case_ref, participant_ref, generation, turn, capability]);
  const current = useRef(identity); current.current = identity;
  const serial = useRef(0);
  const selected = prepared?.identity === identity ? prepared : undefined;
  useEffect(() => {
    const update = () => setRefs(readExecutionRefs(key));
    window.addEventListener("yai:execution-reference", update);
    return () => { current.current = "unmounted"; window.removeEventListener("yai:execution-reference", update); };
  }, [key]);
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  const prepare = async () => {
    if (!application || busy || !turn) return;
    const request = ++serial.current; const stamp = identity;
    const input: CognitivePrepareInput = { case_ref, participant_ref, source_turn_ref: turn, source_part_refs: [], capability };
    setBusy(true); setError(undefined); setPrepared(undefined);
    try {
      const response = await application.prepareCognition(input);
      if (current.current !== stamp || request !== serial.current) return;
      if (response.result_state !== "success" || !response.data) { setError(response.error?.safe_message ?? "YAI could not prepare this input."); return; }
      const plan = response.data;
      if (plan.case_id !== case_ref || plan.participant_id !== participant_ref || plan.case_generation !== generation || plan.capability !== capability) {
        setError("The Case or plan identity changed. Refresh before preparing again."); return;
      }
      setPrepared({ identity: stamp, input, plan });
    } catch { if (current.current === stamp) setError("Plan preparation is unavailable. No execution was submitted."); }
    finally { if (request === serial.current) setBusy(false); }
  };
  const plans = refs.filter(ref => ref.domain === "cognitive_realization").slice(-3).reverse();
  return <section className="compute-section cognitive-execution" aria-label="Cognitive execution">
    <h2>Execute retained input</h2>
    <p>Prepare a current model route for a committed Turn. This is an explicit execution of retained input, separate from sending a new conversation message.</p>
    <div className="cognitive-plan-fields">
      <label>Committed Turn<select aria-label="Committed Turn" value={turn} disabled={busy} onChange={event => { setTurn(event.target.value); setError(undefined); }}>
        <option value="">Select retained input</option>
        {workspace.conversation.turns.map((item, index) => <option value={item.id} key={item.id}>Message {index + 1} · {item.parts.map(part => part.text ?? `[${part.modality}]`).join(" ").slice(0, 120)}</option>)}
      </select></label>
      <label>Capability<select aria-label="Cognitive capability" value={capability} disabled={busy} onChange={event => { setCapability(event.target.value as CognitiveCapability); setError(undefined); }}>
        <option value="primary_conversation">Primary conversation</option><option value="speech_to_text">Speech to text</option><option value="image_understanding">Image understanding</option>
      </select></label>
      <Button disabled={busy || !turn || !application?.supports("cognitive.realization.prepare")} onClick={() => void prepare()}>{busy ? "Preparing route…" : "Prepare execution plan"}</Button>
    </div>
    <p className="surface-note">All original parts of the selected Turn are resolved by YAI. No model is called during preparation; incompatible input or missing bindings are refused.</p>
    {error && <p role="alert">{error}</p>}
    {prepared && !selected && <p role="status">The input or Case changed. Prepare a fresh plan before executing.</p>}
    {selected && <section className="cognitive-plan" aria-label="Prepared execution plan">
      <header><strong>Prepared route</strong><Badge tone={selected.plan.route === "unresolved" ? "warning" : "info"}>{selected.plan.route}</Badge></header>
      <dl className="object-facts"><div><dt>Role</dt><dd>{selected.plan.role}</dd></div><div><dt>Selected model</dt><dd>{workspace.compute.targets.find(item => item.id === selected.plan.selected_target_id)?.model_id ?? selected.plan.selected_target_id ?? "No target selected"}</dd></div></dl>
      {selected.plan.unresolved_reason && <p role="status">{selected.plan.unresolved_reason.replaceAll("_", " ")}</p>}
      {selected.plan.arbitration?.candidates.map((candidate, index) => <p key={`${candidate.target_id}:${index}`}>{workspace.compute.targets.find(item => item.id === candidate.target_id)?.model_id ?? candidate.target_id}: {candidate.exclusions.length ? candidate.exclusions.map(value => value.replaceAll("_", " ")).join(", ") : "No recorded exclusion"}</p>)}
      <details><summary>Exact plan and evidence</summary><pre>{JSON.stringify(selected.plan, null, 2)}</pre></details>
      <Button disabled={selected.plan.route === "unresolved" || !selected.plan.selected_target_id || !application?.supports("cognitive.realize") || plans.some(ref => ref.plan_ref === selected.plan.plan_id)} onClick={() => setConfirm(true)}>Execute prepared plan…</Button>
    </section>}
    {confirm && application && <ApplicationActionDialog title="Execute prepared input" description="Submit this exact retained plan through YAI. It may invoke the provider; current authority, trust, input shape and capacity are checked again. The result is candidate material. Lost confirmation is observed through the retained plan, never automatically resent." submitLabel="Submit exact plan" close={() => setConfirm(false)} enabled={Boolean(selected)} submit={() => {
      if (!selected) throw Error("Plan changed");
      rememberExecution(key, { domain: "cognitive_realization", plan_ref: selected.plan.plan_id });
      return application.realizeCognition({ ...selected.input, plan_ref: selected.plan.plan_id });
    }} committed={refresh} resync={refresh}>
      <p>{selected ? "All original parts of the selected committed Turn." : "The Case or input changed. Close and prepare again."}</p>
      {selected && <code>{selected.plan.plan_id}</code>}
    </ApplicationActionDialog>}
    {plans.length > 0 && <div className="cognitive-recent"><h3>Recent executions in this window</h3>{plans.map(reference => <ExecutionReceipt key={reference.plan_ref} workspace={workspace} platform={platform} reference={reference} />)}</div>}
  </section>;
}
