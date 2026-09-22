import type { WorkbenchRenderContext } from "../../workbench/kernel/types";
import type { WorkbenchSearchItem } from "../../workbench/kernel/types";
import { fileInput, materialInput, resourceInput, sourceInput } from "../surfaces/inputs";

/** Quick Open indexes only this authorized projection, never persistence or the filesystem. */
export function caseOpenTargets({ workspace, actions }: WorkbenchRenderContext): WorkbenchSearchItem[] {
  const items: WorkbenchSearchItem[] = [];
  const open = (input: ReturnType<typeof fileInput>, category: string, detail?: string) => items.push({ id: input.identity, label: input.title, detail, category, icon: input.icon, run: () => actions.openSurface(input) });
  for (const material of workspace.presentation.materials ?? []) open(materialInput(workspace, material.id, material.name), "Case Material", material.path);
  for (const file of workspace.environment.files) open(fileInput(workspace, file.id), "Exposed Case File", file.path);
  for (const source of workspace.environment.sources) open(sourceInput(workspace, source.id), source.roles.includes("policy") ? "Policy Source" : "Source", `${source.kind} · ${source.roles.join(", ")}`);
  for (const resource of workspace.environment.resources) open(resourceInput(workspace, resource.id), "Resource", resource.kind);
  for (const unit of workspace.knowledge.units) items.push({ id: unit.id, label: unit.text.slice(0, 100), detail: unit.kind, category: "Projected Knowledge", icon: "knowledge", run: () => { actions.openPerspective("Knowledge"); actions.inspect(unit.id); } });
  return items;
}
