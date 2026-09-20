import type { ComponentType } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { MaterialReadProjection, OperationResult } from "../../clients/live";
import type { IconName } from "../../components/Icon";
import type { PlatformServices } from "../../platform/services";
import type { SettingsRegistry } from "../settings/registry";
import type { SurfaceCapability, SurfaceInput, SurfaceRole } from "../surface/model";
import type { SurfaceBufferService } from "../surface/buffers";

export interface WorkbenchActions {
  inspect(id: string): void;
  openSurface(input: SurfaceInput): void;
  openPerspective(id: string): void;
  openSettings(): void;
  updateSurface(id: string, patch: Partial<Omit<SurfaceInput, "id" | "identity">>): void;
  replaceSurface(id: string, input: SurfaceInput): void;
}

export interface WorkbenchRenderContext {
  workspace: CasePresentation;
  selection: string;
  actions: WorkbenchActions;
  platform: PlatformServices;
  settings: SettingsRegistry;
  readMaterial(input: { case_ref: string; source_ref: string; revision_ref?: string; path: string; expected_generation?: number }): Promise<OperationResult<MaterialReadProjection>>;
  buffers: SurfaceBufferService;
}

export interface SurfaceSearchResult {
  id: string;
  label: string;
  detail?: string;
  objectRef?: string;
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
  role: SurfaceRole;
  capabilities: readonly SurfaceCapability[];
  component: ComponentType<SurfaceRendererProps>;
  search?: (context: WorkbenchRenderContext, input: SurfaceInput, query: string) => readonly SurfaceSearchResult[] | Promise<readonly SurfaceSearchResult[]>;
}

export interface PanelViewProps extends WorkbenchRenderContext {
  available: boolean;
  toolbarTarget: HTMLElement | null;
  closePanel(): void;
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
