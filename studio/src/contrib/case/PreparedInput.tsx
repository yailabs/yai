import { useState } from "react";
import type { PreparedFrame } from "../../clients/executionContext";

/** Browse the archived, disclosed frame. Filtering changes only this view. */
export function PreparedInput({ frame }: { frame: PreparedFrame }) {
  const [kind, setKind] = useState("");
  const [query, setQuery] = useState("");
  const counts = new Map<string, number>();
  for (const entry of frame.entries) counts.set(entry.value.kind, (counts.get(entry.value.kind) ?? 0) + 1);
  const entries = frame.entries.filter(entry => (!kind || entry.value.kind === kind)
    && (!query.trim() || JSON.stringify(entry).toLocaleLowerCase().includes(query.trim().toLocaleLowerCase())));
  return <details className="prepared-input">
    <summary>Prepared model input · {frame.entries.length} entries</summary>
    <p>This is the archived input prepared by YAI, not proof that the provider received it. Filtering here does not change the request or its context.</p>
    <section aria-label="Prepared task"><strong>Task</strong><p className="prepared-input-task">{frame.task}</p></section>
    <div className="prepared-input-filters">
      <label>Entry kind<select aria-label="Entry kind" value={kind} onChange={event => setKind(event.target.value)}>
        <option value="">All kinds ({frame.entries.length})</option>
        {[...counts].map(([name, count]) => <option key={name} value={name}>{name.replaceAll("_", " ")} ({count})</option>)}
      </select></label>
      <label>Find in prepared input<input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder="Text or exact reference" /></label>
    </div>
    <p role="status">{entries.length} of {frame.entries.length} entries shown</p>
    {entries.map(entry => <details className="prepared-input-entry" key={entry.entry_id}>
      <summary>{entry.value.kind.replaceAll("_", " ")} <small>{entry.posture.replaceAll("_", " ")}</small></summary>
      <code>{entry.entry_id}</code>
      <pre aria-label="Exact prepared value">{JSON.stringify(entry.value.value, null, 2)}</pre>
      <details><summary>Provenance</summary><ul>{entry.provenance.map((source, index) => <li key={index}>{source.kind.replaceAll("_", " ")}<code>{source.source_ref}</code></li>)}</ul></details>
    </details>)}
    <details><summary>Instructions and constraints supplied by YAI</summary>
      <ul>{frame.semantic_instructions.map((text, index) => <li key={`instruction:${index}`}>{text}</li>)}</ul>
      <ul>{frame.model_independent_constraints.map((text, index) => <li key={`constraint:${index}`}>{text}</li>)}</ul>
    </details>
  </details>;
}
