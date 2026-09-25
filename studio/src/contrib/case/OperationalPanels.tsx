import { lazy, Suspense, useEffect, useState } from "react";
const ExecutionHistory = lazy(() => import("./ExecutionActions").then(module => ({ default: module.ExecutionHistory })));
import { Badge, Button, EmptyState } from "../../components/primitives";
import type { ConversationExecution } from "../../clients/conversation";
import { conversationExecutionMessage } from "../../clients/conversation";
import { useApplicationAvailability } from "./applicationActions";
import type { PanelViewProps } from "../../workbench/kernel/types";

/** These views consume disclosed facts; history is never a live execution handle. */
export function OutputPanel(props: PanelViewProps) {
  return props.visible ? <RetainedConversationOutput {...props} /> : null;
}

function RetainedConversationOutput({ workspace, platform }: PanelViewProps) {
  const application = platform.application;
  const availability = useApplicationAvailability(application);
  const [selected, setSelected] = useState<string>();
  const [revision, setRevision] = useState(0);
  const turns = workspace.conversation.turns.filter(turn => turn.execution_request_ref).slice(-32).reverse();
  const turn = turns.find(item => item.id === selected) ?? turns[0];
  const identity = JSON.stringify([workspace.case.case_ref, workspace.case.participant_ref, workspace.case.generation, turn?.id, turn?.execution_request_ref, revision]);
  const [read, setRead] = useState<{ identity: string; catalog: typeof availability.catalog; execution?: ConversationExecution; error?: string }>();
  const supported = Boolean(application?.supports("execution.get"));
  useEffect(() => {
    if (!application || !supported || !turn?.execution_request_ref) return;
    let active = true;
    void application.observeConversation({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref,
      execution: { domain: "cognitive_composition", request_ref: turn.execution_request_ref } }).then(response => {
      if (!active) return;
      const value = response.data;
      const exact = response.result_state === "success" && value?.case_ref === workspace.case.case_ref
        && value.participant_ref === workspace.case.participant_ref && value.turn_ref === turn.id
        && value.request_ref === turn.execution_request_ref;
      setRead({ identity, catalog: availability.catalog, execution: exact ? value : undefined,
        error: exact ? undefined : response.error?.safe_message ?? "No authorized output for this exact Turn." });
    }).catch(() => { if (active) setRead({ identity, catalog: availability.catalog, error: "Output observation is unavailable. Reconnect and refresh; no request was sent again." }); });
    return () => { active = false; };
  }, [application, supported, availability.catalog, identity]);
  const current = read?.identity === identity && read.catalog === availability.catalog && availability.state === "available" ? read : undefined;
  const execution = current?.execution;
  const result = execution?.primary_result;
  return <section className="operational-panel retained-output" aria-label="Retained model output">
    <header className="output-toolbar"><label>Conversation Turn<select aria-label="Output Turn" value={turn?.id ?? ""} disabled={!turns.length} onChange={event => setSelected(event.target.value)}>
      {!turns.length && <option value="">No retained execution</option>}
      {turns.map(item => <option key={item.id} value={item.id}>{item.parts.map(part => part.text ?? "").join(" ").slice(0,100) || item.id}</option>)}
    </select></label><Button disabled={!supported || !turn} onClick={() => setRevision(value => value + 1)}>Refresh output</Button></header>
    <p className="surface-note" title="Up to 32 Turns from the current authorized projection. Refresh only observes; it never dispatches inference.">{turns.length} projected Turns · retained responses</p>
    {!turn ? <EmptyState title="No retained Conversation output" body="Send a message from Conversation. Completed responses can then be inspected here without sending them again." />
      : !supported ? <p role="status">The connected Host does not expose execution observation.</p>
      : current?.error ? <p role="alert">{current.error}</p>
      : !execution ? <p role="status">Reading authorized output…</p>
      : <><div className="output-posture"><Badge tone={result ? "info" : /failed|refused/.test(execution.posture) ? "error" : "warning"}>{result ? "Retained response" : execution.posture.replaceAll("_", " ")}</Badge><span>Model output is candidate material.</span></div>
        {result ? <pre className="retained-output-text" data-result-ref={result.result_id}>{result.output}</pre> : <p>{conversationExecutionMessage(execution)}</p>}
        <details><summary>Execution evidence</summary><p>Read-only retained model response. This is not a live token stream, process stdout or YVEX server log.</p><dl className="object-facts">{Object.entries({ Turn: execution.turn_ref, Request: execution.request_ref, Result: result?.result_id, Invocation: result?.invocation_id, Target: result?.selection.selected_target_id }).filter(([,value]) => value).map(([label,value]) => <div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl></details>
      </>}
  </section>;
}

export function ExecutionPanel({ workspace, actions, selection, platform }: PanelViewProps) {
  const events = workspace.memory.timeline.filter(event => /execution|effect|attempt/.test(event.kind));
  return <section className="operational-panel"><Suspense fallback={<p>Loading execution observation…</p>}><ExecutionHistory workspace={workspace} platform={platform} /></Suspense><p className="surface-note">Committed execution/effect history from the latest bounded projection. History remains distinct from the exact operational observations above.</p>
    {events.slice().reverse().map(event => <button className={`fact-row${selection === event.id ? " selected" : ""}`} key={event.id} onClick={() => actions.inspect(event.id)}><span><strong>{event.kind.replaceAll("_", " ")}</strong><small>{event.component}</small></span></button>)}
    {!events.length && <EmptyState title="No projected execution events" body="Studio has not invented an execution from a Workflow node or a configured provider." />}
  </section>;
}

export function EvidencePanel({ workspace, actions, selection }: PanelViewProps) {
  const events = workspace.memory.timeline.filter(event => /receipt|observation|content_admitted|effect_finalized|evidence/.test(event.kind));
  return <section className="operational-panel"><p className="surface-note">Committed observation and receipt references. Select a Transition to inspect exact causal references; full receipt objects are not projected here.</p>
    {events.slice().reverse().map(event => <button className={`fact-row${selection === event.id ? " selected" : ""}`} key={event.id} onClick={() => actions.inspect(event.id)}><span><strong>{event.kind.replaceAll("_", " ")}</strong><small>Generation {event.sequence} · {event.causal_refs.length} exact references</small></span></button>)}
    {!events.length && <EmptyState title="No evidence events projected" body="No receipt or observation has been substituted with fixture data." />}
  </section>;
}

export function ProblemsPanel({ workspace, actions, selection }: PanelViewProps) {
  const items = [
    ...workspace.environment.sources.filter(source => source.posture && !["acquired", "revoked"].includes(source.posture)).map(source => ({ id: source.id, title: source.label, detail: `Source · ${source.posture}` })),
    ...workspace.knowledge.sources.filter(source => !["qualified", "fixture"].includes(source.status)).map(source => ({ id: source.id, title: source.label, detail: `Knowledge · ${source.status} · ${source.detail}` })),
    ...workspace.authority.reviews.filter(review => review.status === "pending").map(review => ({ id: review.id, title: review.operation_ref, detail: "Review pending" })),
  ];
  return <section className="operational-panel"><p className="surface-note">Current projected Source, Knowledge and Review attention items. This is not a system-wide diagnostic scan.</p>
    {items.map(item => <button key={item.id} className={`fact-row${selection === item.id ? " selected" : ""}`} onClick={() => actions.inspect(item.id)}><span><strong>{item.title}</strong><small>{item.detail}</small></span></button>)}
    {!items.length && <EmptyState title="No attention items in this projection" body="Absence here does not assert that every backend subsystem is healthy." />}
  </section>;
}
