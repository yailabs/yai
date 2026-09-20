import type { CasePresentation } from "../../clients/dataSource";
import type { IconName } from "../../components/Icon";
import type { SurfaceInput } from "../../workbench/surface/model";

export const surfaceTypes = {
  perspective: "case.perspective",
  markdown: "material.markdown",
  text: "material.text",
  image: "material.image",
  pdf: "material.pdf",
  table: "data.table",
  timeline: "case.timeline",
  graph: "case.graph",
  settings: "studio.settings",
  unavailable: "material.unavailable",
} as const;

export function perspectiveInput(viewId: string, title: string, icon: IconName): SurfaceInput {
  const identity = `perspective:${viewId}`;
  return { id: identity, identity, surfaceType: surfaceTypes.perspective, title, icon, pinned: true, viewId };
}

export function settingsInput(): SurfaceInput {
  return { id: "settings", identity: "settings", surfaceType: surfaceTypes.settings, title: "Settings", icon: "settings", pinned: true };
}

export function timelineInput(pinned = false): SurfaceInput {
  const identity = "case-view:memory-timeline";
  return { id: pinned ? identity : "surface:preview", identity, surfaceType: surfaceTypes.timeline, title: "Case Timeline", icon: "memory", pinned, viewId: "Memory", metadata: { projection: "memory.timeline" } };
}

export function graphInput(projection: "memory" | "knowledge", pinned = false): SurfaceInput {
  const identity = `case-view:${projection}-graph`;
  return { id: pinned ? identity : "surface:preview", identity, surfaceType: surfaceTypes.graph, title: projection === "memory" ? "Experience Graph" : "Knowledge Graph", icon: "graph", pinned, viewId: projection === "memory" ? "Memory" : "Knowledge", metadata: { projection } };
}

export function materialInput(
  workspace: CasePresentation,
  objectRef: string,
  title: string,
  pinned = false,
): SurfaceInput {
  const material = workspace.presentation.materials?.find((item) => item.id === objectRef);
  const file = workspace.environment.files.find((item) => item.id === objectRef);
  const sourceRef = file?.source_ref ?? objectRef;
  const source = workspace.environment.sources.find((item) => item.id === sourceRef);
  const qualified = workspace.knowledge.sources.find((item) => item.id === objectRef || item.source_ref === sourceRef);
  const mediaType = material?.mediaType ?? qualified?.media_type ?? source?.media_type;
  const identity = `material:${objectRef}`;
  return {
    id: pinned ? identity : "surface:preview",
    identity,
    surfaceType: resolveMaterialSurfaceType(mediaType),
    title,
    icon: "file",
    pinned,
    objectRef,
    metadata: mediaType ? { mediaType } : undefined,
  };
}

export function resolveMaterialSurfaceType(mediaType?: string): string {
  if (mediaType === "text/markdown") return surfaceTypes.markdown;
  if (mediaType?.startsWith("text/")) return surfaceTypes.text;
  if (mediaType?.startsWith("image/")) return surfaceTypes.image;
  if (mediaType === "application/pdf") return surfaceTypes.pdf;
  if (mediaType === "application/vnd.yai.table+json") return surfaceTypes.table;
  return surfaceTypes.unavailable;
}
