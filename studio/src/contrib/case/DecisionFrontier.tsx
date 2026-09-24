import { useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { WorkingState, FrontierResult, DecisionPreparation, FastSearchPreparation } from "../../clients/memory";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button } from "../../components/primitives";

/** A derived finite frontier is preparation, never an authority Decision. */
export function DecisionFrontier({ state, currentGeneration, application, busy = false }: { state: WorkingState; currentGeneration: number; application?: ApplicationAccess; busy?: boolean }) {
  const [action, setAction] = useState<"frontier" | "request">();
  const [result, setResult] = useState<FrontierResult>();
  const [prepared, setPrepared] = useState<DecisionPreparation>();
  const current = !busy && currentGeneration === state.case_generation;
  return <section className="decision-frontier"><h2>Decision Frontier</h2><p>YAI derives finite alternatives from this exact Working State. At least two qualified alternatives must exist. Preparing a request does not invoke a model, select a winner or authorize an effect.</p>
    <Button disabled={!current || !application?.supports("decision.frontier.prepare")} onClick={() => setAction("frontier")}>Prepare frontier</Button>
    {result && <><p>{result.frontier.visible_candidate_count} visible candidates · {result.frontier.required_candidate_count} required · {result.frontier.omitted_optional_candidates} optional omitted</p>{result.frontier.candidates.map(item => <article className="recall-item" key={item.candidate.candidate_id}><Badge>{item.candidate.candidate_kind}</Badge><p>{item.candidate.description}</p><details><summary>Candidate origins and references</summary><code>{item.candidate.candidate_id}</code><p>{item.candidate.semantic_refs.join(" · ")}</p><pre>{JSON.stringify(item.origins, null, 2)}</pre></details></article>)}<Button disabled={!current || !application?.supports("decision.request.prepare")} onClick={() => setAction("request")}>Prepare decision request</Button></>}
    {prepared && <p className="operation-receipt">Prepared at Case state version {prepared.case_generation}: {prepared.request_id}. No producer execution occurred.</p>}
    {action && application && <ApplicationActionDialog title={action === "frontier" ? "Prepare frontier" : "Prepare decision request"} description="The backend requalifies the exact Working State and current authority. This derived result is not a transferable permission." submitLabel="Prepare" enabled={current} close={() => setAction(undefined)} committed={() => undefined} submit={async form => {
      if (action === "frontier") { const response = await application.prepareFrontier(state, Number(form.get("candidates"))); if (response.result_state === "success" && response.data) { setResult(response.data); setPrepared(undefined); } return response; }
      const response = await application.prepareDecision({ working_state: state, frontier: result!.frontier, decision_kind: String(form.get("kind")).trim(), budget: { max_candidates: result!.frontier.request.max_candidates, max_result_bytes: 65536, max_compute_millis: 1000 } }); if (response.result_state === "success" && response.data) setPrepared(response.data); return response;
    }}>{action === "frontier" ? <label>Maximum candidates<input name="candidates" type="number" min={2} max={32} defaultValue={16} required /></label> : <><label>Decision kind<input name="kind" defaultValue="studio.task.next-step" required maxLength={128} /></label><p>Preparation budget: 64 KiB result · 1,000 ms compute bound. No model is dispatched.</p></>}{!current && <p role="alert">The Case changed. Refresh Working State first.</p>}</ApplicationActionDialog>}
  </section>;
}


/** Inspection of owner-derived navigation; it neither scores nor pages evidence. */
export function FastSearchInspection({ state, currentGeneration, application, busy = false }: { state: WorkingState; currentGeneration: number; application?: ApplicationAccess; busy?: boolean }) {
  const [open, setOpen] = useState(false);
  const [result, setResult] = useState<FastSearchPreparation>();
  const current = !busy && currentGeneration === state.case_generation;
  return <section className="fast-search-inspection" aria-label="Memory navigation preparation">
    <h2>Memory navigation</h2>
    <p>Inspect the exact Recall groups and deferred W4 references prepared by YAI for this Working State. Preparation does not run a model or expand evidence.</p>
    <Button disabled={!current || !application?.supports("semantic.fast_search.prepare")} onClick={() => setOpen(true)}>Inspect memory navigation</Button>
    {!current && <p className="surface-note">Refresh Working State before preparing navigation. Any previous result is a captured snapshot.</p>}
    {result && <>
      <p role="status">{result.availability === "unavailable_producer" ? "System Model unavailable. Standard Recall / Working State remains the execution path." : "No optional navigation choices. Standard Recall / Working State remains the execution path."}</p>
      <p>{result.navigation.choices.length} prepared choices · {result.navigation.omitted_optional_choices} optional choices omitted</p>
      {result.navigation.choices.map(choice => <details className="working-entry" key={choice.candidate.candidate_id}>
        <summary>{choice.origin.kind === "deterministic_path" ? "Standard path" : choice.origin.kind === "resident_recall_group" ? "Resident Recall group" : "Deferred W4 group"}</summary>
        <p>{choice.candidate.description}</p>
        <p className="surface-note">{choice.origin.kind === "deferred_working_group" ? "Reference only. Content is not loaded; use explicit W4 expansion above." : "Prepared by YAI; no model score or selected winner."}</p>
        <dl className="object-facts"><div><dt>Candidate</dt><dd><code>{choice.candidate.candidate_id}</code></dd></div>
          {choice.origin.kind === "resident_recall_group" && <div><dt>Working State entry</dt><dd><code>{choice.origin.entry_id}</code></dd></div>}
          {choice.origin.kind === "deferred_working_group" && <><div><dt>Deferred reference</dt><dd><code>{choice.origin.reference_id}</code></dd></div><div><dt>Group entry</dt><dd><code>{choice.origin.group_entry_id}</code></dd></div></>}
        </dl>
      </details>)}
      <details><summary>Exact preparation identity</summary><dl className="object-facts"><div><dt>Navigation</dt><dd><code>{result.navigation.navigation_id}</code></dd></div><div><dt>Working State</dt><dd><code>{result.navigation.working_state_id}</code></dd></div><div><dt>Actual path</dt><dd>{result.actual_path}</dd></div><div><dt>Fast Search active</dt><dd>{String(result.active)}</dd></div></dl></details>
    </>}
    {open && application && <ApplicationActionDialog title="Inspect memory navigation" description="YAI revalidates current authority and the exact Working State. Only optional navigation choices are bounded; retained evidence is unchanged. After W4 paging, close this dialog and use Refresh exact task before inspection." submitLabel="Inspect" close={() => setOpen(false)} enabled={current} committed={() => undefined} submit={async form => {
      const response = await application.prepareFastSearch(state, Number(form.get("candidates")));
      if (response.result_state === "success" && response.data) {
        if (response.data.navigation.working_state_id !== state.working_state_id) throw new Error("Memory navigation Working State identity mismatch");
        setResult(response.data);
      }
      return response;
    }}><label>Maximum navigation choices<input name="candidates" type="number" min={2} max={32} defaultValue={16} required /></label>{!current && <p role="alert">The Case changed. Refresh Working State first.</p>}</ApplicationActionDialog>}
  </section>;
}
