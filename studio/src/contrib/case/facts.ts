import type { CasePresentation } from "../../clients/dataSource";
import type { IconName } from "../../components/Icon";
import { textLabel } from "./graph";

export function findFact(workspace: CasePresentation, id: string) {
  const values = (...rows: Array<[string, string]>): Array<[string, string]> => rows;
  if (id === workspace.case.case_ref) return { title: workspace.case.display_name, detail: "Durable continuity and application truth owned by YAI.", values: values(["Case", workspace.case.case_ref], ["Lifecycle", workspace.case.case_status], ["Generation", String(workspace.case.generation)], ["Attached participant", workspace.case.participant_ref], ["Data source", workspace.presentation.dataKind]) };
  const fixture = workspace.presentation.materials?.find((item) => item.id === id); if (fixture) return { title: fixture.name, detail: fixture.provenance, values: values(["Media type", fixture.mediaType], ["Path", fixture.path], ["Representation", fixture.body.kind]) };
  const file = workspace.environment.files.find((item) => item.id === id); if (file) { const source = workspace.environment.sources.find((item) => item.id === file.source_ref); return { title: fileName(file.path), detail: file.path, values: values(["Object", file.id], ["Path", file.path], ["Source", file.source_ref], ["Resource", source?.resource_ref ?? "Not exposed"], ["Revision", file.revision_ref], ["Digest", file.digest], ["Size", `${file.bytes} bytes`], ["Media type", file.media_type], ["Backing", typeof file.backing === "string" ? file.backing : JSON.stringify(file.backing)], ["Case generation", String(workspace.case.generation)], ["Mutation", "Unavailable — no admitted participant-origin file write"]) }; }
  const source = workspace.environment.sources.find((item) => item.id === id); if (source) return { title: source.label, detail: source.perimeter, values: values(["Source", source.id], ["Kind", source.kind], ["Roles", source.roles.join(", ")], ["Resource", source.resource_ref], ["Revision", source.revision_ref ?? "Not acquired"], ["Media type", source.media_type], ["Posture", source.posture ?? "declared"]) };
  const resource = workspace.environment.resources.find((item) => item.id === id); if (resource) return { title: resource.label ?? resourceLabel(resource.id), detail: "Operational capability attached to this Case.", values: values(["Resource", resource.id], ["Kind", resource.kind], ["Policy", resource.policy_ref], ["Review", resource.review_requirement], ["Operations", resource.operations.join(", ") || "None disclosed"], ["Read scope", resource.read_prefixes.join(", ") || "None disclosed"]) };
  const document = workspace.knowledge.sources.find(item => item.id === id);
  if (document) return { title: document.path || document.label, detail: "Document in the current Knowledge projection.", values: values(["Object", document.id], ["Backing posture", document.detail], ["Source", document.source_ref], ["Revision", document.revision_ref], ["Digest", document.digest], ["Extractor", document.extractor], ["Status", document.status]) };
  const unit = workspace.knowledge.units.find(item => item.id === id);
  if (unit) return { title: textLabel(unit.text, humanize(unit.kind)), detail: unit.text, values: values(["Object", unit.id], ["Kind", humanize(unit.kind)], ["Posture", unit.posture], ["Source", unit.source_ref], ...(unit.parent_ref ? [["Parent", unit.parent_ref] as [string, string]] : []), ...(unit.predicate ? [["Predicate", unit.predicate] as [string, string]] : []), ...(unit.value != null ? [["Value", typeof unit.value === "string" ? unit.value : JSON.stringify(unit.value)] as [string, string]] : []), ["Topics", unit.topics.join(", ") || "None projected"]) };
  const entity = workspace.knowledge.entities.find(item => item.id === id);
  if (entity) return { title: entity.id, detail: `${entity.definitions.length} qualified definitions`, values: values(["Object", entity.id], ["Definitions", entity.definitions.join(", ")]) };
  const contradiction = workspace.knowledge.contradictions.find(item => item.id === id);
  if (contradiction) return { title: `${contradiction.entity} · ${contradiction.predicate}`, detail: contradiction.posture, values: values(["Object", contradiction.id], ["Entity", contradiction.entity], ["Predicate", contradiction.predicate], ["Members", contradiction.members.join(", ")]) };
  const relation = [...workspace.knowledge.relations, ...workspace.memory.relations, ...workspace.work.edges].find(item => item.id === id);
  if (relation) return { title: humanize(relation.kind), detail: "A qualified relationship between the exact projected endpoints.", values: values(["Relation", relation.id], ["From", relation.from], ["To", relation.to], ["Kind", relation.kind]) };
  const node = workspace.work.nodes.find(item => item.node_id === id);
  if (node) return { title: node.node_id, detail: node.reason, values: values(["Node", node.node_id], ["Kind", node.node_kind], ["Posture", node.posture]) };
  const participant = workspace.overview.participants.find((item) => item.id === id); if (participant) return { title: participant.id, detail: participant.is_current ? "Current attached participant" : "Participant in this Case", values: values(["Roles", participant.roles.join(", ") || "Unavailable"]) };
  const policy = workspace.authority.policies.find((item) => item.id === id); if (policy) return { title: humanize(policy.policy_key), detail: policy.reason, values: values(["Binding", policy.id], ["Owner", policy.owner_ref], ["Version", policy.version], ["Source", policy.source_ref], ["Artifact", policy.artifact_ref]) };
  const event = workspace.memory.timeline.find((item) => item.id === id); if (event) return { title: humanize(event.kind), detail: event.summary ?? "Committed Case transition", values: values(["Transition", event.id], ["Generation", String(event.sequence)], ["Committed", new Date(event.committed_at_unix_ms).toISOString()], ["Owner", event.component], ["Participant", event.participant_ref ?? "Not supplied"]) };
  const review = workspace.authority.reviews.find((item) => item.id === id); if (review) return { title: compact(review.id), detail: "Authority review bound to this Case.", values: values(["Status", review.status], ["Operation", review.operation_ref], ["Policy", review.policy_ref]) };
  const provider = workspace.compute.targets.find((item) => item.id === id); if (provider) return { title: provider.provider_key, detail: provider.model_id, values: values(["Target", provider.id], ["Model", provider.model_id], ["Adapter", provider.adapter], ["Locality", provider.locality], ["Endpoint", provider.endpoint], ...typeof provider.posture === "object" ? [["Trust", provider.posture.trust?.posture ?? "Unreviewed"], ["Health", provider.posture.health.posture], ["Qualification", provider.posture.qualification?.id ?? "Not exposed"]] as Array<[string, string]> : []) };
  return { title: compact(id), detail: "Referenced object. Its details are not included in the current projection.", values: values(["Reference", id], ["Case", workspace.case.case_ref]) };
}
export function factKind(workspace: CasePresentation, id: string) { if ([...workspace.knowledge.relations, ...workspace.memory.relations, ...workspace.work.edges].some(item => item.id === id)) return "relation"; if (workspace.knowledge.units.some(item => item.id === id)) return "knowledge unit"; if (workspace.knowledge.sources.some(item => item.id === id)) return "knowledge document"; if (workspace.knowledge.entities.some(item => item.id === id)) return "entity"; if (workspace.knowledge.contradictions.some(item => item.id === id)) return "contradiction"; if (id === workspace.case.case_ref) return "case"; if (workspace.presentation.materials?.some((item) => item.id === id)) return "material"; if (workspace.environment.files.some((item) => item.id === id)) return "file"; if (workspace.environment.sources.some((item) => item.id === id)) return "source"; if (workspace.environment.resources.some((item) => item.id === id)) return "resource"; if (workspace.overview.participants.some((item) => item.id === id)) return "participant"; if (workspace.authority.policies.some((item) => item.id === id)) return "policy"; if (workspace.authority.reviews.some((item) => item.id === id)) return "review"; if (workspace.compute.targets.some((item) => item.id === id)) return "provider"; if (workspace.work.nodes.some((item) => item.node_id === id)) return "workflow"; if (workspace.memory.timeline.some((item) => item.id === id)) return "event"; return "case fact"; }
export function factIcon(kind: string): IconName { return ({ relation: "graph", "knowledge unit": "knowledge", "knowledge document": "file", entity: "knowledge", contradiction: "review", case: "case", material: "file", file: "file", source: "source", resource: "resource", participant: "people", policy: "authority", review: "review", provider: "compute", workflow: "work", event: "memory" } as Record<string, IconName>)[kind] ?? "overview"; }

function compact(value: string) { return value.length > 38 ? `${value.slice(0, 20)}…${value.slice(-12)}` : value; }
function fileName(value: string) { return value.split(/[\\/]/).filter(Boolean).at(-1) ?? value; }
function resourceLabel(value: string) { return value.replace(/^resource:/, "").replaceAll("-", " ").replace(/^./, (char) => char.toUpperCase()); }
function humanize(value: string) { return value.replaceAll("_", " ").replace(/^./, (char) => char.toUpperCase()); }

/** Only explicitly projected object references; this is navigation, not inferred graph edges. */
export function factReferences(workspace: CasePresentation, id: string): string[] {
  const relation = [...workspace.knowledge.relations, ...workspace.memory.relations, ...workspace.work.edges].find(item => item.id === id);
  if (relation) return [relation.from, relation.to];
  const unit = workspace.knowledge.units.find(item => item.id === id);
  if (unit) return [...new Set([unit.source_ref, unit.parent_ref, unit.entity_ref, ...unit.references].filter((ref): ref is string => Boolean(ref)))];
  const document = workspace.knowledge.sources.find(item => item.id === id);
  if (document) return [document.source_ref, ...workspace.environment.files.filter(file => file.source_ref === document.source_ref && file.revision_ref === document.revision_ref && file.path === document.path && file.digest === document.digest).map(file => file.id)];
  const entity = workspace.knowledge.entities.find(item => item.id === id);
  if (entity) return entity.definitions;
  const contradiction = workspace.knowledge.contradictions.find(item => item.id === id);
  if (contradiction) return [contradiction.entity, ...contradiction.members];
  const file = workspace.environment.files.find(item => item.id === id);
  if (file) return [file.source_ref];
  const source = workspace.environment.sources.find(item => item.id === id);
  if (source) return [source.resource_ref];
  const event = workspace.memory.timeline.find(item => item.id === id);
  if (event) return event.causal_refs;
  const work = workspace.work.resolution?.nodes.find(item => item.node_id === id);
  return work?.evidence_refs ?? [];
}
