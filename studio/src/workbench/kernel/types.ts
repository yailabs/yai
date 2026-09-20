import type { ComponentType } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { IconName } from "../../components/Icon";
import type { SurfaceCapability, SurfaceInput } from "../surface/model";

export interface WorkbenchActions {
  inspect(id: string): void;
  openSurface(input: SurfaceInput): void;
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
  surface: SurfaceInput;
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

export interface SurfaceRendererProps extends WorkbenchRenderContext {
  input: SurfaceInput;
}

export interface SurfaceRendererContribution {
  type: string;
  capabilities: readonly SurfaceCapability[];
  component: ComponentType<SurfaceRendererProps>;
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
