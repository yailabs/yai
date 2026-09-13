// Local rendering inputs, not YAI DTOs, admission contracts, or canonical state.
export type ScenarioId = "ordinary" | "developer" | "execution";
export type Activity = "Case" | "Sources" | "Files" | "Work" | "Providers";
export type BottomTab =
  | "Terminal"
  | "Output"
  | "Executions"
  | "Evidence"
  | "Problems";
export type Posture =
  | "completed"
  | "running"
  | "waiting for review"
  | "failed"
  | "ready";
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
  | { kind: "work"; title: string; description: string }
  | { kind: "provider"; title: string };
export interface MaterialView {
  id: string;
  name: string;
  path: string;
  category: "source" | "artifact" | "work" | "provider";
  format: string;
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
  initial: { tabs: readonly string[]; active: string; bottom: BottomTab };
}

// A synchronous presentation seam for this offline shell only. Future live
// mapping must follow a qualified YAI contract; this declares no wire protocol.
export interface StudioClient {
  readonly mode: "fixture";
  scenarios(): readonly { id: ScenarioId; label: string }[];
  workspace(id: ScenarioId): WorkspacePresentation;
}
