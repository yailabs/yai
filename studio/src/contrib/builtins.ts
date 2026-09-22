import { caseOpenTargets } from "./case/navigation";
import type { StudioContribution } from "../workbench/kernel/contributions";
import { lazy } from "react";
import { ActivityView, CaseSidebarView, ConversationView, GraphSurface, InspectorView, perspectiveMeta, perspectives, PerspectiveSurface, ResourceSurface, SourceSurface, TimelineSurface, searchGraphSurface, searchTimelineSurface } from "./case/CaseViews";
import { TerminalPanelView } from "./terminal/TerminalContribution";
import { OutputPanel, ExecutionPanel, EvidencePanel, ProblemsPanel } from "./case/OperationalPanels";
import { AudioSurface, ImageSurface, MarkdownSurface, StructuredTextSurface, TextEditorSurface, TextSurface, UnavailableMaterialSurface, VideoSurface, searchMaterialSurface, searchPdfSurface } from "./surfaces/MaterialSurfaces";
import { perspectiveInput, surfaceTypes } from "./surfaces/inputs";
import { JournalPanel } from "./case/JournalPanel";
import { IdentityAccess, ManageAccess } from "./settings/WorkbenchAccess";

const SettingsSurface = lazy(() => import("./settings/SettingsSurface").then(module => ({ default: module.SettingsSurface })));
const TableSurface = lazy(() => import("./surfaces/TableSurface"));
const PdfSurface = lazy(() => import("./surfaces/PdfSurface"));
const RecallSurface = lazy(() => import("./case/MemorySurfaces").then(module => ({ default: module.RecallSurface })));
const WorkingStateSurface = lazy(() => import("./case/MemorySurfaces").then(module => ({ default: module.WorkingStateSurface })));

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
    register({ workbench, platform }) {
      return [
        workbench.registerQuickOpen({ id: "case.projected-objects", items: caseOpenTargets }),
        workbench.registerActivityFooter({ id: "studio.identity", order: 0, component: IdentityAccess }),
        workbench.registerActivityFooter({ id: "studio.manage", order: 1, component: ManageAccess }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.perspective, role: "projection", capabilities: ["pinnable", "navigable", "selectable"], component: PerspectiveSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.source, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable"], component: SourceSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.resource, role: "system", capabilities: ["previewable", "pinnable", "navigable", "selectable"], component: ResourceSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.textEditor, role: "content", capabilities: ["previewable", "pinnable", "editable", "dirty-aware", "navigable", "selectable", "searchable"], component: TextEditorSurface, findInRenderer: true, search: searchMaterialSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.markdown, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable", "searchable"], component: MarkdownSurface, search: searchMaterialSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.text, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable", "searchable"], component: TextSurface, search: searchMaterialSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.structuredText, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable", "searchable"], component: StructuredTextSurface, search: searchMaterialSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.image, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable", "zoomable"], component: ImageSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.pdf, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable", "zoomable", "searchable"], component: PdfSurface, search: searchPdfSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.audio, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable"], component: AudioSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.video, role: "content", capabilities: ["previewable", "pinnable", "navigable", "selectable"], component: VideoSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.table, role: "projection", capabilities: ["previewable", "pinnable", "navigable", "selectable", "searchable"], component: TableSurface, search: searchMaterialSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.timeline, role: "projection", capabilities: ["previewable", "pinnable", "navigable", "selectable", "zoomable", "searchable"], component: TimelineSurface, search: searchTimelineSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.graph, role: "projection", capabilities: ["previewable", "pinnable", "navigable", "selectable", "zoomable", "searchable"], component: GraphSurface, search: searchGraphSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.recall, role: "projection", capabilities: ["pinnable", "navigable", "selectable"], component: RecallSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.workingState, role: "projection", capabilities: ["pinnable", "navigable", "selectable"], component: WorkingStateSurface }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.settings, role: "system", capabilities: ["singleton", "navigable", "searchable"], component: SettingsSurface, search: async (context, input, query) => (await import("./settings/SettingsSurface")).searchSettingsSurface(context, input, query) }),
        workbench.registerSurfaceRenderer({ type: surfaceTypes.unavailable, role: "content", capabilities: ["previewable", "pinnable", "navigable"], component: UnavailableMaterialSurface }),
        workbench.settings.register({ id: "workbench.openPreview", title: "Open material in preview", description: "Single-click reuses one preview tab. Double-click pins the Surface.", section: "Workbench", scope: "local", control: "boolean", defaultValue: true, available: true }),
        workbench.settings.register({ id: "workbench.sidebar.width", title: "Explorer width", description: "Width in pixels. Dragging the Explorer divider updates the same preference.", section: "Workbench", scope: "local", control: "number", defaultValue: 204, available: true }),
        workbench.settings.register({ id: "workbench.auxiliary.width", title: "Context panel width", description: "Width in pixels for Conversation, Inspector and Activity.", section: "Workbench", scope: "local", control: "number", defaultValue: 320, available: true }),
        workbench.settings.register({ id: "workbench.panel.heightRatio", title: "Bottom panel height", description: "Fraction of window height, from 0.20 to 0.72. Maximize remains temporary.", section: "Workbench", scope: "local", control: "number", defaultValue: .36, available: true }),
        workbench.settings.register({ id: "general.caseContinuity", title: "Case continuity", description: "Back and Forward traverse local Studio navigation. Closing a tab or window does not close the durable Case.", section: "General", scope: "local", control: "information", available: true }),
        workbench.settings.register({ id: "appearance.reducedMotion", title: "Reduce motion", description: "Minimize nonessential Workbench transitions.", section: "Appearance", scope: "local", control: "boolean", defaultValue: false, available: true }),
        workbench.settings.register({ id: "terminal.scrollback", title: "Terminal scrollback", description: "Maximum number of lines retained by a local terminal renderer.", section: "Terminal", scope: "local", control: "number", defaultValue: 5000, available: platform.host.capabilities.terminalAvailable, unavailableReason: "Requires the desktop host" }),
        workbench.settings.register({ id: "host.currentTopology", title: "Current YAI topology", description: "Studio attaches to the resident same-user YAI Local Host. RuntimeInstance supervision remains separate.", section: "YAI Host", scope: "host", control: "information", available: true }),
        workbench.settings.register({ id: "editor.localBuffers", title: "Local file buffers", description: "The text editor retains local undo, find/replace and unsaved drafts. Saving remains unavailable until YAI exposes governed file mutation.", section: "Editor", scope: "local", control: "information", available: true }),
        workbench.settings.register({ id: "security.authority", title: "Case authority", description: "Authority remains YAI-owned and cannot be changed through local Studio preferences.", section: "Security", scope: "case", control: "information", available: false, unavailableReason: "Use qualified YAI authority operations" }),
        workbench.settings.register({ id: "advanced.persistence", title: "Preference storage", description: "Versioned local preferences use browser/WebView local storage; no Case database is created.", section: "Advanced", scope: "local", control: "information", available: true }),
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
        workbench.registerPanelView({ id: "Journal", title: "Journal", order: 1, component: JournalPanel }),
        workbench.registerPanelView({ id: "Output", title: "Output", order: 1.5, component: OutputPanel }),
        workbench.registerPanelView({ id: "Executions", title: "Executions", order: 2, component: ExecutionPanel }),
        workbench.registerPanelView({ id: "Evidence", title: "Evidence", order: 3, component: EvidencePanel }),
        workbench.registerPanelView({ id: "Problems", title: "Problems", order: 4, component: ProblemsPanel }),
      ];
    },
  },
];
