import type { CaseAttachment, CaseUpdate, LiveCaseRow, LiveWorkspace, OperationResult } from "./live";
import type { MaterialView, WorkspacePresentation } from "./presentation";

export type CaseDataKind = "live" | "fixture";
export type BackendPosture = "embedded-local" | "resident-host-connected" | "starting" | "reconnecting" | "unavailable" | "not-used";

export interface CasePresentation extends LiveWorkspace {
  presentation: {
    dataKind: CaseDataKind;
    backendPosture: BackendPosture;
    fixture?: { id: string; label: string; provenance: string };
    materials?: readonly MaterialView[];
  };
}

export interface CaseCatalog {
  cases: LiveCaseRow[];
  authority: string;
}

export interface CaseSearchResult {
  id: string;
  label: string;
  detail: string;
  object_ref?: string;
}

export interface CaseDataSource {
  readonly kind: CaseDataKind;
  listCases(): Promise<OperationResult<CaseCatalog>>;
  openCase(caseRef: string): Promise<OperationResult<CaseAttachment>>;
  caseSummary(caseRef: string, expectedGeneration?: number): Promise<OperationResult<CasePresentation>>;
  subscribe?(resumeToken?: string): Promise<OperationResult<unknown>>;
  heartbeat?(caseRef: string): Promise<OperationResult<{ case_ref: string; generation: number; cursor: string; stream_state: string }>>;
  searchCase?(caseRef: string, query: string): Promise<OperationResult<readonly CaseSearchResult[]>>;
  listen?(handler: (update: CaseUpdate) => void): Promise<() => void>;
  composition?(): readonly import("./presentation").CompositionSection[];
}

let fixtureCorrelation = 0;
const fixtureResult = <T>(operation_ref: string, data: T): OperationResult<T> => ({
  operation_ref,
  result_state: "success",
  correlation_ref: `studio:fixture:${++fixtureCorrelation}`,
  data,
});

const missingFixture = <T>(operation_ref: string, caseRef: string): OperationResult<T> => ({
  operation_ref,
  result_state: "error",
  correlation_ref: `studio:fixture:${++fixtureCorrelation}`,
  error: {
    code: "fixture_case_not_found",
    message: `Unknown fixture Case: ${caseRef}`,
    safe_message: "The requested authored fixture Case does not exist.",
    result_state: "error",
  },
});

const posture = (value?: string) => value ?? "fixture";
const itemKind = (kind: string) => kind.replaceAll(" ", "_");

export function fixtureToCasePresentation(
  source: WorkspacePresentation,
  stepId?: string,
): CasePresentation {
  const step = source.information.progression.steps.find((candidate) => candidate.id === stepId)
    ?? source.information.progression.steps.find((candidate) => candidate.id === source.information.progression.initial)
    ?? source.information.progression.steps[0];
  const generation = step?.index ?? 1;
  const materialById = new Map(source.materials.map((material) => [material.id, material]));
  const environmentItems = source.information.environment.flatMap((group) => group.items);
  const knowledgeItems = source.information.knowledge.flatMap((group) => group.items);
  const authorityItems = source.information.authority.flatMap((group) => group.items);
  const workItems = source.information.work.flatMap((group) => group.items);
  const computeItems = source.information.compute.flatMap((group) => group.items);
  const sourceMaterials = source.materials.filter((material) => material.category === "source");
  const fileMaterials = source.materials.filter((material) => material.category === "source" || material.category === "artifact");
  const environmentSources = environmentItems.filter((item) => item.kind === "source" || item.kind === "document" || item.kind === "repository");
  const sources = (environmentSources.length ? environmentSources : sourceMaterials.map((material) => ({
    id: material.id, label: material.name, detail: material.path, kind: "source" as const,
    material: material.id, posture: "current" as const,
  }))).map((item) => {
    const material = item.material ? materialById.get(item.material) : materialById.get(item.id);
    return {
      id: item.id,
      label: item.label,
      perimeter: material?.path ?? item.detail,
      media_type: material?.mediaType ?? item.kind,
      roles: ["fixture-source"],
      resource_ref: `fixture:resource:${item.id}`,
      posture: posture(item.posture),
      revision_ref: "fixture:authored",
      items: 1,
    };
  });
  const files = fileMaterials.map((material) => ({
    id: material.id,
    source_ref: material.id,
    source_label: material.name,
    revision_ref: "fixture:authored",
    path: material.path,
    digest: "unavailable-in-authored-fixture",
    bytes: 0,
    backing: { posture: "synthetic" },
  }));
  const resources = environmentItems.filter((item) => item.kind === "resource" || item.kind === "machine").map((item) => ({
    id: item.id,
    kind: item.kind,
    policy_ref: "fixture:no-authority",
    review_requirement: posture(item.posture),
  }));
  const timeline = source.information.memory.timeline.filter((entry) => entry.step <= generation).map((entry, index) => ({
    id: entry.id,
    sequence: index + 1,
    committed_at_unix_ms: Date.UTC(2026, 8, 19, 9, index),
    kind: entry.kind,
    component: "fixture-presentation",
    causal_refs: [],
    summary: `${entry.title} · ${entry.detail}`,
  }));
  const relations = source.information.memory.edges.filter((edge) => edge.step <= generation).map((edge, index) => ({
    id: `fixture:edge:${index}`,
    from: edge.from,
    to: edge.to,
    kind: edge.label,
  }));
  const reviews = authorityItems.filter((item) => item.kind === "review" || item.kind === "decision").map((item) => ({
    id: item.id,
    status: posture(item.posture),
    operation_ref: item.detail,
    policy_ref: "fixture:authored-policy",
    required_roles: [],
  }));
  const nodes = workItems.map((item) => ({
    node_id: item.id,
    node_kind: itemKind(item.kind),
    posture: posture(item.posture),
    reason: item.detail,
  }));
  const targets = (computeItems.length ? computeItems : [{
    id: "fixture:provider", label: source.provider.name, detail: source.provider.note, kind: "provider" as const,
  }]).filter((item) => item.kind === "provider" || item.kind === "model").map((item) => ({
    id: item.id,
    provider_key: item.label,
    adapter: "fixture-presentation",
    model_id: source.provider.model,
    locality: source.provider.location,
    endpoint: "not connected",
    posture: item.posture ?? source.provider.posture,
    management: "not exposed",
  }));
  const turns = source.activity.filter((entry) => entry.kind === "turn").map((entry, index) => ({
    id: entry.id,
    thread_ref: `fixture:thread:${source.fixture.id}`,
    participant_ref: entry.kind === "turn" ? entry.participant : "fixture",
    generation: Math.min(generation, index + 1),
    parts: [{ modality: "text", media_type: "text/plain", text: entry.kind === "turn" ? entry.text : "" }],
  }));
  const units = knowledgeItems.map((item) => ({
    id: item.id,
    source_ref: item.material ?? sources[0]?.id ?? "fixture:source:unavailable",
    kind: itemKind(item.kind),
    text: `${item.label}: ${item.detail}`,
    posture: posture(item.posture),
    references: item.material ? [item.material] : [],
    topics: [],
  }));
  return {
    case: {
      case_ref: `fixture:${source.fixture.id}`,
      display_name: source.case.label,
      case_status: "fixture",
      generation,
      tenant_ref: "fixture:development",
      participant_ref: source.case.participant,
    },
    overview: {
      attention: source.information.overview.highlights.map((item) => ({
        kind: item.kind,
        title: item.label,
        detail: item.detail,
        ref: item.id,
      })),
      participants: source.participants.map((participant) => ({
        id: participant.name,
        roles: [participant.role],
        is_current: participant.id === source.case.participant,
      })),
    },
    environment: { sources, files, resources, artifacts: source.materials.filter((item) => item.category === "artifact") },
    knowledge: {
      status: units.length ? "fixture" : "empty",
      message: units.length ? "Authored development projection" : "No authored fixture knowledge.",
      sources: sourceMaterials.map((material) => ({
        id: material.id, source_ref: material.id, label: material.name,
        revision_ref: "fixture:authored", path: material.path,
        digest: "unavailable-in-authored-fixture", media_type: material.mediaType,
        extractor: "fixture-author", status: "synthetic", detail: material.provenance,
      })),
      units,
      entities: knowledgeItems.filter((item) => item.kind === "knowledge").map((item) => ({ id: item.id, definitions: [item.detail] })),
      topics: [], contradictions: [], relations: [],
    },
    memory: { authority: "fixture", timeline, relations, generation },
    authority: { policies: [], reviews, grants: [], empty: authorityItems.length === 0 },
    work: {
      status: nodes.length ? "fixture" : "empty",
      message: nodes.length ? "Authored development work projection" : "No authored fixture workflow.",
      nodes,
      edges: [],
    },
    compute: {
      status: targets.length ? "fixture" : "empty",
      message: "Presentation facts only; no provider connection is implied.",
      targets,
    },
    conversation: { read_only: true, turns },
    freshness: { generation, resync_operation: "fixture.snapshot" },
    presentation: {
      dataKind: "fixture",
      backendPosture: "not-used",
      fixture: source.fixture,
      materials: source.materials,
    },
  };
}

export class FixtureDataSource implements CaseDataSource {
  readonly kind = "fixture" as const;
  constructor(private readonly fixture: import("./fixture").FixtureClient) {}

  async listCases() {
    return fixtureResult("case.list", {
      authority: "explicit-development-fixture",
      cases: this.fixture.catalog().recentCases.map((item, index) => {
        const workspace = this.fixture.workspace(item.id);
        return {
          case_ref: `fixture:${item.id}`,
          display_name: item.label,
          case_status: item.posture,
          generation: workspace.information.progression.steps.at(-1)?.index ?? index + 1,
          participant_count: workspace.participants.length,
          source_count: workspace.information.environment.flatMap((group) => group.items).length,
          resource_count: 0,
          pending_review_count: workspace.information.authority.flatMap((group) => group.items).filter((entry) => entry.kind === "review").length,
        };
      }),
    });
  }

  async searchCase(caseRef: string, query: string) {
    const id = caseRef.replace(/^fixture:/, "") as import("./presentation").ScenarioId;
    let workspace: WorkspacePresentation;
    try { workspace = this.fixture.workspace(id); } catch { return missingFixture<readonly CaseSearchResult[]>("studio.fixture.search_case", caseRef); }
    const needle = query.trim().toLocaleLowerCase();
    const results = needle ? workspace.materials.filter((material) => `${material.name} ${material.path} ${material.mediaType} ${material.provenance}`.toLocaleLowerCase().includes(needle)).map((material) => ({ id: material.id, label: material.name, detail: `${material.mediaType} · ${material.path}`, object_ref: material.id })) : [];
    return fixtureResult<readonly CaseSearchResult[]>("studio.fixture.search_case", results);
  }

  async openCase(caseRef: string) {
    const id = caseRef.replace(/^fixture:/, "") as import("./presentation").ScenarioId;
    let workspace: WorkspacePresentation;
    try { workspace = this.fixture.workspace(id); }
    catch { return missingFixture<CaseAttachment>("case.open", caseRef); }
    return fixtureResult("case.open", {
      case_ref: `fixture:${id}`,
      case_status: "fixture",
      generation: workspace.information.progression.steps.at(-1)?.index ?? 1,
      authenticated_principal: "fixture:developer",
      participant_ref: workspace.case.participant,
      thread_ref: `fixture:thread:${id}`,
      attachment: "ephemeral" as const,
    });
  }

  async caseSummary(caseRef: string, expectedGeneration?: number) {
    const id = caseRef.replace(/^fixture:/, "") as import("./presentation").ScenarioId;
    let workspace: WorkspacePresentation;
    try { workspace = this.fixture.workspace(id); }
    catch { return missingFixture<CasePresentation>("case.summary", caseRef); }
    const step = expectedGeneration
      ? workspace.information.progression.steps.find((candidate) => candidate.index === expectedGeneration)?.id
      : undefined;
    return fixtureResult("case.summary", fixtureToCasePresentation(workspace, step));
  }

  composition() { return this.fixture.composition(); }
}

export class LiveDataSource implements CaseDataSource {
  readonly kind = "live" as const;
  constructor(private readonly client: import("./live").LiveClient) {}
  listCases() { return this.client.listCases(); }
  openCase(caseRef: string) { return this.client.openCase(caseRef); }
  async caseSummary(caseRef: string, expectedGeneration?: number) {
    const result = await this.client.caseSummary(caseRef, expectedGeneration);
    if (!result.data) {
      return {
        operation_ref: result.operation_ref,
        result_state: result.result_state,
        correlation_ref: result.correlation_ref,
        error: result.error,
      } satisfies OperationResult<CasePresentation>;
    }
    return {
      ...result,
      data: { ...result.data, presentation: { dataKind: "live" as const, backendPosture: "embedded-local" as const } },
    } satisfies OperationResult<CasePresentation>;
  }
  subscribe(resumeToken?: string) { return this.client.subscribe(resumeToken); }
  heartbeat(caseRef: string) { return this.client.heartbeat(caseRef); }
  listen(handler: (update: CaseUpdate) => void) { return this.client.listen(handler); }
}
