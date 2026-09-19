import type { StudioContribution } from "../workbench/kernel/contributions";
import { ActivityView, CaseSidebarView, ConversationView, InspectorView, MaterialEditor, perspectiveMeta, perspectives, PerspectiveEditor, SettingsEditor } from "./case/CaseViews";
import { EmptyToolView, OutputPanelView, TerminalPanelView } from "./terminal/TerminalContribution";

export const builtInContributions: readonly StudioContribution[] = [
  {
    id: "yai.case-perspectives",
    register({ workbench }) {
      return perspectives.flatMap((perspective, order) => [
        workbench.registerViewContainer({ id: perspective, title: perspective, icon: perspectiveMeta[perspective].icon, order }),
        workbench.registerView({ id: `${perspective}.explorer`, containerId: perspective, title: perspective, order: 0, component: CaseSidebarView }),
      ]);
    },
  },
  {
    id: "yai.editors",
    register({ workbench }) {
      return [
        workbench.registerEditor({ type: "perspective", component: PerspectiveEditor }),
        workbench.registerEditor({ type: "material", component: MaterialEditor }),
        workbench.registerEditor({ type: "settings", component: SettingsEditor }),
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
