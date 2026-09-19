import type { ComponentType } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { IconName } from "../../components/Icon";
import type { EditorInput } from "../editor/model";

export interface WorkbenchActions {
  inspect(id: string): void;
  openMaterial(id: string, label: string, pinned?: boolean): void;
  openPerspective(id: string): void;
  openSettings(): void;
}

export interface WorkbenchRenderContext {
  workspace: CasePresentation;
  selection: string;
  actions: WorkbenchActions;
}

export interface ViewContainerContribution {
  id: string;
  title: string;
  icon: IconName;
  order: number;
}

export interface SidebarViewProps extends WorkbenchRenderContext {
  containerId: string;
}

export interface SidebarViewContribution {
  id: string;
  containerId: string;
  title: string;
  order: number;
  component: ComponentType<SidebarViewProps>;
}

export interface EditorRendererProps extends WorkbenchRenderContext {
  input: EditorInput;
}

export interface EditorContribution {
  type: EditorInput["type"];
  component: ComponentType<EditorRendererProps>;
}

export interface PanelViewProps extends WorkbenchRenderContext {
  available: boolean;
}

export interface PanelViewContribution {
  id: string;
  title: string;
  order: number;
  component: ComponentType<PanelViewProps>;
}

export interface AuxiliaryViewProps extends WorkbenchRenderContext {}

export interface AuxiliaryViewContribution {
  id: string;
  title: string;
  order: number;
  component: ComponentType<AuxiliaryViewProps>;
}

export interface InspectorContribution {
  kind: string;
  component: ComponentType<AuxiliaryViewProps>;
}
