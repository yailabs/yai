// Local rendering inputs, not YAI DTOs, admission contracts, or canonical state.
export type ScenarioId = "ordinary" | "developer" | "execution";
export type Activity =
  | "Overview"
  | "Environment"
  | "Knowledge"
  | "Memory"
  | "Authority"
  | "Work"
  | "Compute";
export type ContextMode = "Conversation" | "Inspector" | "Activity";
export type MemoryMode = "Timeline" | "Graph";
export type BottomTab =
  "Terminal" | "Output" | "Executions" | "Evidence" | "Problems";
export type Posture =
  "completed" | "running" | "waiting for review" | "failed" | "ready";
export interface ParticipantView {
  id: string;
  name: string;
  initials: string;
  kind: "Human" | "AI participant";
  role: string;
}
export interface DocumentSection {
  title: string;
  body: string;
  points?: readonly string[];
}
export type MaterialBody =
  | {
      kind: "text";
      content: string;
      language?: string;
    }
  | {
      kind: "document";
      eyebrow: string;
      title: string;
      intro: string;
      sections: readonly DocumentSection[];
      references?: readonly string[];
    }
  | {
      kind: "diff";
      title: string;
      description: string;
      before: string;
      after: string;
      lines: readonly { text: string; change?: "add" | "remove" }[];
    }
  | {
      kind: "image";
      source: string;
      alt: string;
      width: number;
      height: number;
      caption?: string;
    }
  | {
      kind: "pdf";
      pageCount?: number;
      source?: string;
      unavailableReason?: string;
    }
  | {
      kind: "table";
      columns: readonly { key: string; label: string }[];
      rows: readonly { id: string; values: Readonly<Record<string, string>> }[];
    }
  | {
      kind: "audio" | "video";
      source?: string;
      unavailableReason?: string;
      durationSeconds?: number;
      caption?: string;
    }
  | { kind: "work"; title: string; description: string }
  | { kind: "provider"; title: string };
export interface MaterialView {
  id: string;
  name: string;
  path: string;
  category: "source" | "artifact" | "work" | "provider";
  format: string;
  mediaType: string;
  provenance: string;
  changed?: boolean;
  body: MaterialBody;
}
export type ActivityEntry =
  | {
      id: string;
      kind: "turn";
      participant: string;
      time: string;
      text: string;
    }
  | {
      id: string;
      kind: "notice" | "execution" | "review";
      time: string;
      title: string;
      text: string;
      material?: string;
    };
export interface ExecutionView {
  id: string;
  title: string;
  detail: string;
  posture: Posture;
  time: string;
}
export interface EvidenceView {
  id: string;
  title: string;
  origin: string;
  detail: string;
  material?: string;
}
export interface ProblemView {
  id: string;
  severity: "warning" | "error";
  title: string;
  detail: string;
  material: string;
}
export interface ProviderView {
  name: string;
  location: string;
  model: string;
  posture: string;
  note: string;
}
export interface ExplorerItem {
  id: string;
  label: string;
  detail: string;
  kind:
    | "source"
    | "resource"
    | "repository"
    | "document"
    | "machine"
    | "knowledge"
    | "memory"
    | "policy"
    | "review"
    | "decision"
    | "workflow"
    | "execution"
    | "artifact"
    | "provider"
    | "model";
  material?: string;
  posture?: Posture | "current" | "derived" | "unavailable";
}
export interface ExplorerGroup {
  label: string;
  note?: string;
  items: readonly ExplorerItem[];
}
export interface TimelineEvent {
  id: string;
  time: string;
  title: string;
  detail: string;
  kind:
    "source" | "participant" | "execution" | "artifact" | "review" | "decision";
  step: number;
  material?: string;
}
export interface GraphNode {
  id: string;
  label: string;
  detail: string;
  kind: "case" | "participant" | "source" | "execution" | "artifact" | "review";
  x: number;
  y: number;
  step: number;
  material?: string;
}
export interface GraphEdge {
  from: string;
  to: string;
  label: string;
  step: number;
}
export interface InspectorView {
  eyebrow: string;
  title: string;
  description: string;
  facts: readonly { label: string; value: string }[];
  note?: string;
}
export interface ProgressionStep {
  id: string;
  label: string;
  time: string;
  summary: string;
  index: number;
}
export interface CaseInformation {
  overview: {
    status: string;
    attention: string;
    highlights: readonly ExplorerItem[];
  };
  environment: readonly ExplorerGroup[];
  knowledge: readonly ExplorerGroup[];
  memory: {
    timeline: readonly TimelineEvent[];
    nodes: readonly GraphNode[];
    edges: readonly GraphEdge[];
  };
  authority: readonly ExplorerGroup[];
  work: readonly ExplorerGroup[];
  compute: readonly ExplorerGroup[];
  inspector: Readonly<Record<string, InspectorView>>;
  progression: {
    initial: string;
    steps: readonly ProgressionStep[];
  };
}
export interface RecentCaseView {
  id: ScenarioId;
  label: string;
  reference: string;
  purpose: string;
  currentWork: string;
  environment: string;
  posture: string;
  updated: string;
}
export interface StartCenterPresentation {
  recentCases: readonly RecentCaseView[];
  recentSources: readonly { label: string; kind: string; caseLabel: string }[];
  environments: readonly { label: string; detail: string; posture: string }[];
  lastCase: ScenarioId;
}
export interface CompositionSection {
  id:
    | "identity"
    | "sources"
    | "participants"
    | "authority"
    | "resources"
    | "compute";
  label: string;
  eyebrow: string;
  description: string;
}
export interface WorkspacePresentation {
  fixture: { id: ScenarioId; label: string; provenance: string };
  case: {
    label: string;
    reference: string;
    context: string;
    purpose: string;
    currentWork: string;
    participant: string;
  };
  participants: readonly ParticipantView[];
  materials: readonly MaterialView[];
  activity: readonly ActivityEntry[];
  executions: readonly ExecutionView[];
  evidence: readonly EvidenceView[];
  problems: readonly ProblemView[];
  output: readonly string[];
  provider: ProviderView;
  information: CaseInformation;
  initial: { tabs: readonly string[]; active: string; bottom: BottomTab };
}

// Authored fixture input consumed by FixtureDataSource. This remains a
// development-data contract and declares no YAI wire protocol or shell owner.
export interface FixturePresentationClient {
  readonly mode: "fixture";
  catalog(): StartCenterPresentation;
  composition(): readonly CompositionSection[];
  scenarios(): readonly { id: ScenarioId; label: string }[];
  workspace(id: ScenarioId): WorkspacePresentation;
}
