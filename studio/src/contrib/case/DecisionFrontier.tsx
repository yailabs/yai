import { useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { WorkingState, FrontierResult, DecisionPreparation } from "../../clients/memory";
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
    {prepared && <p className="operation-receipt">Prepared at generation {prepared.case_generation}: {prepared.request_id}. No producer execution occurred.</p>}
    {action && application && <ApplicationActionDialog title={action === "frontier" ? "Prepare frontier" : "Prepare decision request"} description="The backend requalifies the exact Working State and current authority. This derived result is not a transferable permission." submitLabel="Prepare" enabled={current} close={() => setAction(undefined)} committed={() => undefined} submit={async form => {
      if (action === "frontier") { const response = await application.prepareFrontier(state, Number(form.get("candidates"))); if (response.result_state === "success" && response.data) { setResult(response.data); setPrepared(undefined); } return response; }
      const response = await application.prepareDecision({ working_state: state, frontier: result!.frontier, decision_kind: String(form.get("kind")).trim(), budget: { max_candidates: result!.frontier.request.max_candidates, max_result_bytes: 65536, max_compute_millis: 1000 } }); if (response.result_state === "success" && response.data) setPrepared(response.data); return response;
    }}>{action === "frontier" ? <label>Maximum candidates<input name="candidates" type="number" min={2} max={32} defaultValue={16} required /></label> : <><label>Decision kind<input name="kind" defaultValue="studio.task.next-step" required maxLength={128} /></label><p>Preparation budget: 64 KiB result · 1,000 ms compute bound. No model is dispatched.</p></>}{!current && <p role="alert">The Case changed. Refresh Working State first.</p>}</ApplicationActionDialog>}
  </section>;
}
