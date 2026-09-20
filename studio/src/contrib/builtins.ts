import type { StudioContribution } from "../workbench/kernel/contributions";
import { ActivityView, CaseSidebarView, ConversationView, GraphSurface, InspectorView, perspectiveMeta, perspectives, PerspectiveSurface, SettingsSurface, TimelineSurface } from "./case/CaseViews";
import { EmptyToolView, OutputPanelView, TerminalPanelView } from "./terminal/TerminalContribution";
import { ImageSurface, MarkdownSurface, PdfSurface, TableSurface, TextSurface, UnavailableMaterialSurface } from "./surfaces/MaterialSurfaces";
import { perspectiveInput, surfaceTypes } from "./surfaces/inputs";

export const builtInContributions: readonly StudioContribution[] = [
  {
    id: "yai.case-perspectives",
    register({ workbench }) {
      return perspectives.flatMap((perspective, order) => [
        workbench.registerViewContainer({ id: perspective, title: perspective, icon: perspectiveMeta[perspective].icon, order, surface: perspectiveInput(perspective, perspective, perspectiveMeta[perspective].icon) }),
        workbench.registerView({ id: `${perspective}.explorer`, containerId: perspective, title: perspective, order: 0, component: CaseSidebarView }),
      ]);
    },
  },
  {
    id: "yai.work-surfaces",
    register({ workbench }) {
      return [
        workbench.registerSurfaceRenderer({ type: surfaceTypes.perspective, capabilities: ["read", "select", "navigate"], component: PerspectiveSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.markdown, capabilities: ["read", "select", "navigate", "search"], component: MarkdownSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.text, capabilities: ["read", "select", "navigate", "search"], component: TextSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.image, capabilities: ["read", "select", "navigate", "zoom"], component: ImageSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.pdf, capabilities: ["read", "select", "navigate", "zoom", "search"], component: PdfSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.table, capabilities: ["read", "select", "navigate"], component: TableSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.timeline, capabilities: ["read", "select", "navigate", "zoom"], component: TimelineSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.graph, capabilities: ["read", "select", "navigate", "zoom", "search"], component: GraphSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.settings, capabilities: ["read", "navigate"], component: SettingsSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.unavailable, capabilities: ["read"], component: UnavailableMaterialSurface }),
      ];
    },
  },
  {
    id: "yai.auxiliary",
    register({ workbench }) {
      return [
        workbench.registerAuxiliaryView({ id: "Conversation", title: "Conversation", order: 0, component: ConversationView }),
        workbench.registerAuxiliaryView({ id: "Inspector", title: "Inspector", order: 1, component: InspectorView }),
        workbench.registerAuxiliaryView({ id: "Activity", title: "Activity", order: 2, component: ActivityView }),
        workbench.registerInspector({ kind: "default", component: InspectorView }),
      ];
    },
  },
  {
    id: "yai.panel",
    register({ workbench }) {
      return [
        workbench.registerPanelView({ id: "Terminal", title: "Terminal", order: 0, component: TerminalPanelView }),
        workbench.registerPanelView({ id: "Output", title: "Output", order: 1, component: OutputPanelView }),
        workbench.registerPanelView({ id: "Executions", title: "Executions", order: 2, component: EmptyToolView }),
        workbench.registerPanelView({ id: "Evidence", title: "Evidence", order: 3, component: EmptyToolView }),
        workbench.registerPanelView({ id: "Problems", title: "Problems", order: 4, component: EmptyToolView }),
      ];
    },
  },
];
