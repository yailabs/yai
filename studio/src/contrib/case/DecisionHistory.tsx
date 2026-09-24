import { useEffect, useRef, useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { DecisionCorpus, DecisionEvaluation, DecisionTrajectory } from "../../clients/work";
import { Badge, Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";
import { factKind } from "./facts";

type Props = Pick<SurfaceRendererProps, "workspace" | "platform" | "actions">;
const human = (value: string) => value.replaceAll("_", " ");

/** The parent keys this captured read by Case, Participant and observed version. */
export function DecisionHistory({ workspace, platform, actions }: Props) {
  const application = platform.application;
  useApplicationAvailability(application);
  const [limit, setLimit] = useState(16);
  const [reference, setReference] = useState("");
  const [corpus, setCorpus] = useState<DecisionCorpus>();
  const [selected, setSelected] = useState<DecisionTrajectory>();
  const [evaluation, setEvaluation] = useState<DecisionEvaluation>();
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const alive = useRef(true);
  const pending = useRef(false);
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
  const identity = { case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref };
  const matches = (value: { case_id: string; participant_id: string }) => value.case_id === identity.case_ref && value.participant_id === identity.participant_ref;
  const run = async (kind: "corpus" | "inspect" | "evaluate", decisionRef = reference.trim()) => {
    if (!application || pending.current) return;
    pending.current = true; setBusy(true); setError(undefined);
    // A refused re-read must not leave earlier disclosures on screen.
    setSelected(undefined); setCorpus(undefined); setEvaluation(undefined);
    try {
      if (kind === "corpus") {
        const result = await application.decisionCorpus({ ...identity, max_decisions: limit });
        if (!alive.current) return;
        if (result.result_state !== "success" || !result.data) throw Error(result.error?.safe_message ?? result.result_state);
        if (result.data.schema !== "yai.cognitive_decision_corpus.v1" || !matches(result.data) || result.data.trajectories.some(item => !matches(item))) throw Error("Decision history identity mismatch.");
        setCorpus(result.data);
      } else if (kind === "inspect") {
        const result = await application.inspectDecision({ ...identity, decision_ref: decisionRef });
        if (!alive.current) return;
        if (result.result_state !== "success" || !result.data) throw Error(result.error?.safe_message ?? result.result_state);
        if (result.data.schema !== "yai.cognitive_decision_trajectory.v1" || !matches(result.data) || result.data.decision.decision_id !== decisionRef) throw Error("Decision history identity mismatch.");
        setReference(decisionRef); setSelected(result.data);
      } else {
        const result = await application.evaluateDecisions({ ...identity, max_decisions: limit });
        if (!alive.current) return;
        if (result.result_state !== "success" || !result.data) throw Error(result.error?.safe_message ?? result.result_state);
        if (result.data.schema !== "yai.cognitive_decision_trajectory_evaluation.v1") throw Error("Unsupported Decision evaluation response.");
        setEvaluation(result.data);
      }
    } catch (failure) { if (alive.current) setError(failure instanceof Error ? failure.message : "Decision history unavailable."); }
    finally { pending.current = false; if (alive.current) setBusy(false); }
  };
  const link = (ref: string) => factKind(workspace, ref) === "case fact"
    ? <code key={ref}>{ref}</code>
    : <Button key={ref} onClick={() => actions.inspect(ref)}>{ref}</Button>;
  return <section className="work-section decision-history" aria-label="Decision history">
    <h2>Decisions and consequences</h2>
    <p>Reconstruct what was known before a committed Decision and its qualified consequences. Current disclosure still applies. Historical evidence grants no new authority.</p>
    <div className="object-action-row"><label>Decision limit <input aria-label="Decision history limit" type="number" min={1} max={128} value={limit} onChange={event => setLimit(Number(event.target.value))} /></label>
      <Button disabled={busy || !Number.isInteger(limit) || limit < 1 || limit > 128 || !application?.supports("decision.trajectory.corpus")} onClick={() => void run("corpus")}>Load Decision history</Button>
      <Button disabled={busy || !Number.isInteger(limit) || limit < 1 || limit > 128 || !application?.supports("decision.trajectory.evaluate")} onClick={() => void run("evaluate")}>Evaluate reconstruction</Button></div>
    <form className="object-action-row" onSubmit={event => { event.preventDefault(); void run("inspect"); }}><label>Exact Decision reference <input aria-label="Exact Decision reference" value={reference} onChange={event => setReference(event.target.value)} required /></label><Button type="submit" disabled={busy || !reference.trim() || !application?.supports("decision.trajectory.inspect")}>Inspect Decision</Button></form>
    {busy && <p role="status">Reading qualified history…</p>}
    {error && <p role="alert">{error}</p>}
    {corpus && <div className="decision-corpus"><p>{corpus.trajectories.length} reconstructed · {corpus.omitted_visible_decisions} visible Decisions omitted by the selected bound.</p>{!corpus.trajectories.length && <p>No visible Decisions in this bounded reconstruction.</p>}{corpus.trajectories.map(item => <button className="fact-row" key={item.trajectory_id} onClick={() => void run("inspect", item.decision.decision_id)} disabled={busy || !application?.supports("decision.trajectory.inspect")}><Badge tone={item.decision.outcome === "deny" ? "error" : item.decision.outcome === "require_review" ? "warning" : "neutral"}>{human(item.decision.outcome)}</Badge><span><strong>{item.decision.reason}</strong><small>{item.decision.decision_id}</small></span></button>)}</div>}
    {selected && <article className="decision-reconstruction"><header><h3>{human(selected.decision.outcome)}</h3><p>{selected.decision.reason}</p></header>
      {selected.task_context && <p>{selected.task_context}</p>}
      <p>Candidate reconstruction: {human(selected.candidate_posture)}. {selected.candidate_posture !== "exact_reconstructed" && "The selected operation does not establish all alternatives available at the time."}</p>
      <dl className="object-facts">{Object.entries(selected.readiness).map(([name, available]) => <div key={name}><dt>{human(name.replace(/_available$/, ""))}</dt><dd>{available ? "Available" : "Not reconstructed"}</dd></div>)}</dl>
      <h4>Decision basis</h4><div className="object-action-row">{selected.decision.basis_refs.map(link)}</div>
      <h4>Related evidence</h4><div className="object-action-row">{selected.related_evidence.map(item => link(item.transition_id))}</div>
      <h4>Missing or unsupported</h4><ul>{[...selected.missingness, ...selected.pre_decision.unsupported_families].map((item, index) => <li key={index}>{human(item)}</li>)}</ul>
      <details><summary>Exact historical cut and provenance</summary><dl className="object-facts"><div><dt>State version before Decision</dt><dd>{selected.pre_decision.cut_generation}</dd></div><div><dt>Committed state version</dt><dd>{selected.decision_generation}</dd></div><div><dt>Trajectory</dt><dd>{selected.trajectory_id}</dd></div><div><dt>Operation</dt><dd>{selected.decision.operation_id}</dd></div></dl>{selected.pre_decision.content_backing.map((item, index) => <p key={index}>{item.source_ref} · {item.posture}</p>)}{selected.correction_decisions.map(link)}</details>
    </article>}
    {evaluation && <div className="decision-evaluation"><h3>Reconstruction coverage</h3><p>Structural evidence checks, not model quality, confidence or permission to act.</p><dl className="object-facts">{Object.entries(evaluation).filter(([key]) => key !== "schema").map(([key, value]) => <div key={key}><dt>{human(key)}</dt><dd>{value}</dd></div>)}</dl></div>}
  </section>;
}
