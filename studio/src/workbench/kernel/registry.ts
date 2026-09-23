import { toDisposable, type Disposable } from "../../platform/lifecycle";
import type {
  QuickOpenContribution,
  ActivityFooterContribution,
  AuxiliaryViewContribution,
  SurfaceRendererContribution,
  InspectorContribution,
  PanelViewContribution,
  SidebarViewContribution,
  ViewContainerContribution,
} from "./types";
import { SettingsRegistry } from "../settings/registry";

export class WorkbenchRegistry implements Disposable {
  private readonly openProviders = new Map<string, QuickOpenContribution>();
  registerQuickOpen(value: QuickOpenContribution) { return this.insert(this.openProviders, value.id, value, "quick open provider"); }
  quickOpenProviders() { return [...this.openProviders.values()]; }
  readonly settings = new SettingsRegistry();
  private readonly containers = new Map<string, ViewContainerContribution>();
  private readonly views = new Map<string, SidebarViewContribution>();
  private readonly surfaces = new Map<string, SurfaceRendererContribution>();
  private readonly panels = new Map<string, PanelViewContribution>();
  private readonly auxiliary = new Map<string, AuxiliaryViewContribution>();
  private readonly inspectors = new Map<string, InspectorContribution>();
  private readonly footer = new Map<string, ActivityFooterContribution>();

  registerViewContainer(value: ViewContainerContribution) { return this.insert(this.containers, value.id, value, "view container"); }
  registerView(value: SidebarViewContribution) { return this.insert(this.views, value.id, value, "view"); }
  registerSurfaceRenderer(value: SurfaceRendererContribution) { return this.insert(this.surfaces, value.type, value, "surface renderer"); }
  registerPanelView(value: PanelViewContribution) { return this.insert(this.panels, value.id, value, "panel view"); }
  registerAuxiliaryView(value: AuxiliaryViewContribution) { return this.insert(this.auxiliary, value.id, value, "auxiliary view"); }
  registerInspector(value: InspectorContribution) { return this.insert(this.inspectors, value.kind, value, "inspector"); }
  registerActivityFooter(value: ActivityFooterContribution) { return this.insert(this.footer, value.id, value, "activity footer"); }

  viewContainers() { return [...this.containers.values()].sort((a, b) => a.order - b.order); }
  railContainers(preference: { hidden: readonly string[]; order: readonly string[] }) {
    const section = { core: 0, platform: 1, pinned: 2 };
    return this.viewContainers().filter(item => item.rail?.fixed !== false ||
      (!preference.hidden.includes(item.id) && (item.rail.defaultPinned !== false || preference.order.includes(item.id))))
      .sort((a, b) => {
        const group = section[a.rail?.section ?? "core"] - section[b.rail?.section ?? "core"];
        if (group) return group;
        const fixedA = a.rail?.fixed !== false, fixedB = b.rail?.fixed !== false;
        if (fixedA !== fixedB) return fixedA ? -1 : 1;
        if (fixedA) return a.order - b.order;
        const position = (id: string, fallback: number) => {
          const index = preference.order.indexOf(id); return index < 0 ? preference.order.length + fallback : index;
        };
        return position(a.id, a.order) - position(b.id, b.order);
      });
  }
  viewsFor(containerId: string) { return [...this.views.values()].filter((view) => view.containerId === containerId).sort((a, b) => a.order - b.order); }
  surfaceRenderer(type: string) { return this.surfaces.get(type); }
  panelViews() { return [...this.panels.values()].sort((a, b) => a.order - b.order); }
  auxiliaryViews() { return [...this.auxiliary.values()].sort((a, b) => a.order - b.order); }
  inspector(kind: string) { return this.inspectors.get(kind) ?? this.inspectors.get("default"); }
  activityFooter() { return [...this.footer.values()].sort((a, b) => a.order - b.order); }

  dispose() {
    this.containers.clear(); this.views.clear(); this.surfaces.clear();
    this.panels.clear(); this.auxiliary.clear(); this.inspectors.clear();
    this.settings.dispose();
    this.footer.clear(); this.openProviders.clear();
  }

  private insert<T>(target: Map<string, T>, id: string, value: T, kind: string): Disposable {
    if (target.has(id)) throw new Error(`Duplicate ${kind}: ${id}`);
    target.set(id, value);
    return toDisposable(() => target.delete(id));
  }
}
