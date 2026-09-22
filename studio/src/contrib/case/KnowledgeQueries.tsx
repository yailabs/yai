import { useEffect, useRef, useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { KnowledgeView, KnowledgeUnit } from "../../clients/knowledge";
import { Badge, Button, SearchInput } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

/** Owner-backed lexical queries, separate from filtering the current projection. */
export function KnowledgeQueries({ workspace, platform, actions }: Pick<SurfaceRendererProps, "workspace" | "platform" | "actions">) {
  const application = platform.application; useApplicationAvailability(application);
  const [query, setQuery] = useState(""); const [source, setSource] = useState("");
  const [view, setView] = useState<KnowledgeView>(); const [units, setUnits] = useState<KnowledgeUnit[]>([]);
  const [resolved, setResolved] = useState<KnowledgeUnit>(); const [navigation, setNavigation] = useState("");
  const [error, setError] = useState(""); const [busy, setBusy] = useState(false);
  const request = { case_id: workspace.case.case_ref, source: source || null, revision: null, max_units: 512 };
  const identity = JSON.stringify([workspace.case.case_ref, workspace.case.generation, source]);
  const current = useRef(identity); current.current = identity;
  // A Case update can revoke backing/disclosure. Clear prior query results rather
  // than preserving them as if they were currently qualified.
  useEffect(() => { setView(undefined); setUnits([]); setResolved(undefined); setNavigation(""); }, [workspace.case.generation, source]);
  const invoke = async (kind: "inspect" | "search" | "navigation" | "resolve", id?: string) => {
    if (!application || busy) return; setBusy(true); setError(""); const stamp = identity;
    try {
      if (kind === "navigation") { const response = await application.navigateKnowledge(request); if (stamp !== current.current) return; if (response.result_state === "success" && response.data) setNavigation(response.data.navigation); else setError(response.error?.safe_message ?? response.result_state); }
      else if (kind === "resolve") { const response = await application.resolveKnowledge(request, id!); if (stamp !== current.current) return; if (response.result_state === "success" && response.data && response.data.unit.id === id && response.data.view.case_id === workspace.case.case_ref) { setResolved(response.data.unit); setView(response.data.view); } else { setResolved(undefined); setError(response.error?.safe_message ?? "Exact unit is not currently disclosed."); } }
      else if (kind === "search") { const response = await application.searchKnowledge(request, query.trim(), 24); if (stamp !== current.current) return; if (response.result_state === "success" && response.data?.view.case_id === workspace.case.case_ref) { setView(response.data.view); const found = new Map(response.data.view.units.map(unit => [unit.id, unit])); setUnits(response.data.hits.flatMap(hit => found.get(hit.document_id) ?? [])); } else { setUnits([]); setError(response.error?.safe_message ?? response.result_state); } }
      else { const response = await application.inspectKnowledge(request); if (stamp !== current.current) return; if (response.result_state === "success" && response.data?.case_id === workspace.case.case_ref) { setView(response.data); setUnits(response.data.units.slice(0, 24)); } else { setUnits([]); setError(response.error?.safe_message ?? response.result_state); } }
    } finally { setBusy(false); }
  };
  return <section className="knowledge-queries work-section"><h2>Search Knowledge</h2><p>Search acquired Sources through YAI, with exact source references. Relevance is not confidence.</p><form className="knowledge-query-form" onSubmit={event => { event.preventDefault(); void invoke("search"); }}><SearchInput aria-label="Knowledge query" value={query} onChange={event => setQuery(event.target.value)} placeholder="Search qualified content" required /><select aria-label="Knowledge Source scope" value={source} onChange={event => setSource(event.target.value)}><option value="">All disclosed Sources</option>{workspace.environment.sources.filter(item => item.roles.includes("knowledge")).map(item => <option key={item.id} value={item.id}>{item.label}</option>)}</select><Button type="submit" disabled={busy || !query.trim() || !application?.supports("knowledge.search")}>Search Knowledge</Button></form><div className="object-action-row"><Button disabled={busy || !application?.supports("knowledge.inspect")} onClick={() => void invoke("inspect")}>Inspect current Knowledge</Button><Button disabled={busy || !application?.supports("knowledge.navigation")} onClick={() => void invoke("navigation")}>Build documentary navigation</Button></div>
    {error && <p role="alert">{error}</p>}{view && <p className="surface-note">{view.units.length} units in the bounded owner response · {view.relations.length} relations · showing {units.length} results</p>}
    {units.map(unit => <button className="fact-row" key={unit.id} onClick={() => void invoke("resolve", unit.id)} disabled={busy || !application?.supports("knowledge.resolve")}><span><strong>{unit.text.slice(0, 120)}</strong><small>{unit.kind} · resolve exact unit</small></span></button>)}
    {resolved && <article className="knowledge-resolved"><Badge>{resolved.posture.replaceAll("_", " ")}</Badge><p>{resolved.text}</p>{workspace.knowledge.units.some(unit => unit.id === resolved.id) && <Button onClick={() => actions.inspect(resolved.id)}>Inspect selected unit</Button>}<details><summary>Exact source closure</summary><code>{resolved.id}</code><p>{view?.sources.find(item => item.id === resolved.source)?.path ?? resolved.source}</p><code>{view?.id}</code></details></article>}
    {navigation && <details open><summary>Navigation returned by YAI</summary><pre>{navigation}</pre></details>}
  </section>;
}
