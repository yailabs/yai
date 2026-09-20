import { toDisposable, type Disposable } from "../../platform/lifecycle";
import type {
  AuxiliaryViewContribution,
  SurfaceRendererContribution,
  InspectorContribution,
  PanelViewContribution,
  SidebarViewContribution,
  ViewContainerContribution,
} from "./types";

export class WorkbenchRegistry implements Disposable {
  private readonly containers = new Map<string, ViewContainerContribution>();
  private readonly views = new Map<string, SidebarViewContribution>();
  private readonly surfaces = new Map<string, SurfaceRendererContribution>();
  private readonly panels = new Map<string, PanelViewContribution>();
  private readonly auxiliary = new Map<string, AuxiliaryViewContribution>();
  private readonly inspectors = new Map<string, InspectorContribution>();

  registerViewContainer(value: ViewContainerContribution) { return this.insert(this.containers, value.id, value, "view container"); }
  registerView(value: SidebarViewContribution) { return this.insert(this.views, value.id, value, "view"); }
  registerSurfaceRenderer(value: SurfaceRendererContribution) { return this.insert(this.surfaces, value.type, value, "surface renderer"); }
  registerPanelView(value: PanelViewContribution) { return this.insert(this.panels, value.id, value, "panel view"); }
  registerAuxiliaryView(value: AuxiliaryViewContribution) { return this.insert(this.auxiliary, value.id, value, "auxiliary view"); }
  registerInspector(value: InspectorContribution) { return this.insert(this.inspectors, value.kind, value, "inspector"); }

  viewContainers() { return [...this.containers.values()].sort((a, b) => a.order - b.order); }
  viewsFor(containerId: string) { return [...this.views.values()].filter((view) => view.containerId === containerId).sort((a, b) => a.order - b.order); }
  surfaceRenderer(type: string) { return this.surfaces.get(type); }
  panelViews() { return [...this.panels.values()].sort((a, b) => a.order - b.order); }
  auxiliaryViews() { return [...this.auxiliary.values()].sort((a, b) => a.order - b.order); }
  inspector(kind: string) { return this.inspectors.get(kind) ?? this.inspectors.get("default"); }

  dispose() {
    this.containers.clear(); this.views.clear(); this.surfaces.clear();
    this.panels.clear(); this.auxiliary.clear(); this.inspectors.clear();
  }

  private insert<T>(target: Map<string, T>, id: string, value: T, kind: string): Disposable {
    if (target.has(id)) throw new Error(`Duplicate ${kind}: ${id}`);
    target.set(id, value);
    return toDisposable(() => target.delete(id));
  }
}
