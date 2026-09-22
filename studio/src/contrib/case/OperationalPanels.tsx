import { EmptyState } from "../../components/primitives";
import type { PanelViewProps } from "../../workbench/kernel/types";

/** These views consume disclosed facts; history is never a live execution handle. */
export function OutputPanel({ workspace }: PanelViewProps) {
  return <EmptyState title="Execution output is not exposed here" body={workspace.compute.targets.length
    ? "A provider binding is configured. This published Application boundary does not yet expose reconnect-safe execution output. Committed activity is available in Journal."
    : "No provider target is bound. Terminal is a local shell; Journal contains committed Case history. Neither is model output."} />;
}

export function ExecutionPanel({ workspace, actions, selection }: PanelViewProps) {
  const events = workspace.memory.timeline.filter(event => /execution|effect|attempt/.test(event.kind));
  return <section className="operational-panel"><p className="surface-note">Committed execution/effect history from the latest bounded projection. Live execution observation and controls are not exposed by this published boundary.</p>
    {events.slice().reverse().map(event => <button className={`fact-row${selection === event.id ? " selected" : ""}`} key={event.id} onClick={() => actions.inspect(event.id)}><span><strong>{event.kind.replaceAll("_", " ")}</strong><small>{event.component} · generation {event.sequence}</small></span></button>)}
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
