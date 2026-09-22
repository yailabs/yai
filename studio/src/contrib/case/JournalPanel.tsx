import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Button, EmptyState, SearchInput } from "../../components/primitives";
import type { PanelViewProps } from "../../workbench/kernel/types";
import { timelineInput } from "../surfaces/inputs";

export function JournalPanel({ workspace, actions, selection, toolbarTarget, visible }: PanelViewProps) {
  const [paused, setPaused] = useState(false);
  const [retained, setRetained] = useState({ caseRef: workspace.case.case_ref, events: workspace.memory.timeline });
  const snapshot = retained.caseRef === workspace.case.case_ref ? retained.events : workspace.memory.timeline;
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("");
  const [component, setComponent] = useState("");
  const list = useRef<HTMLDivElement>(null);
  useEffect(() => { setPaused(false); setQuery(""); setKind(""); setComponent(""); }, [workspace.case.case_ref]);
  useEffect(() => { if (!paused) setRetained({ caseRef: workspace.case.case_ref, events: workspace.memory.timeline }); }, [paused, workspace.memory.timeline, workspace.case.case_ref]);
  const rows = useMemo(() => snapshot.filter(event => (!kind || event.kind === kind) && (!component || event.component === component) && `${event.kind} ${event.summary ?? ""} ${event.component} ${event.participant_ref ?? ""} ${event.causal_refs.join(" ")}`.toLowerCase().includes(query.toLowerCase())), [snapshot, query, kind, component]);
  useEffect(() => { if (!paused && visible && list.current) list.current.scrollTop = list.current.scrollHeight; }, [rows, paused, visible]);
  const latest = workspace.memory.timeline.at(-1)?.sequence;
  const shown = snapshot.at(-1)?.sequence;
  return <section className="case-journal" aria-label="Case Journal">
    {toolbarTarget && createPortal(<><Button aria-pressed={!paused} onClick={() => setPaused(value => !value)}>{paused ? "Resume follow" : "Pause follow"}</Button><Button onClick={() => actions.openSurface(timelineInput(true))}>Timeline</Button></>, toolbarTarget)}
    <div className="journal-filter"><SearchInput aria-label="Search Journal" placeholder="Filter committed events" value={query} onChange={event => setQuery(event.target.value)} /><select aria-label="Journal event kind" value={kind} onChange={event => setKind(event.target.value)}><option value="">All kinds</option>{[...new Set(snapshot.map(event => event.kind))].sort().map(value => <option key={value}>{value}</option>)}</select><select aria-label="Journal component" value={component} onChange={event => setComponent(event.target.value)}><option value="">All components</option>{[...new Set(snapshot.map(event => event.component))].sort().map(value => <option key={value}>{value}</option>)}</select></div>
    <div className="journal-events" ref={list} aria-label="Committed Case events">{rows.map(event => <button key={event.id} className={`journal-event${selection === event.id ? " selected" : ""}`} title={event.id} onClick={() => actions.inspect(event.id)}><time dateTime={new Date(event.committed_at_unix_ms).toISOString()}>{new Date(event.committed_at_unix_ms).toLocaleTimeString()}</time><span className="journal-sequence">{event.sequence}</span><span><strong>{event.kind.replaceAll("_", " ")}</strong><small>{event.summary ?? event.component}{event.participant_ref ? ` · ${event.participant_ref}` : ""}</small></span></button>)}{!rows.length && <EmptyState title="No matching committed events" body={snapshot.length ? "Adjust the Journal filters." : "No Transition history is projected for this Case."} />}</div>
    <footer role="status">{paused ? `Paused at sequence ${shown ?? "—"}${latest !== shown ? " · newer projected events available" : ""}` : "Following latest"} · {rows.length} / {snapshot.length} projected events · latest 160 maximum; older history is not pageable here.</footer>
  </section>;
}
