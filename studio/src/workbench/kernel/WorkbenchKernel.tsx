import { SurfaceTabs } from "../surface/SurfaceTabs";
import { OpenWith } from "../surface/OpenWith";
import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CasePresentation, CaseSearchResult } from "../../clients/dataSource";
import type { MaterialReadProjection, OperationResult } from "../../clients/live";
import { Icon } from "../../components/Icon";
import { Badge, IconButton, PanelHeader } from "../../components/primitives";
import { DesktopWindowControls } from "../../app/DesktopWindowControls";
import { beginDesktopWindowDrag, toggleDesktopWindowMaximize } from "../../platform/desktopWindow";
import { dispatchTerminalCommand } from "../../terminal/commands";
import type { PlatformServices } from "../../platform/services";
import { DisposableStore, toDisposable } from "../../platform/lifecycle";
import { when } from "../../platform/context";
import type { WorkbenchRegistry } from "./registry";
import { type SurfaceInput } from "../surface/model";
import type { NavigationLocation } from "../../platform/navigation";
import { ApplicationMenuBar } from "./ApplicationMenuBar";
import { Splitter } from "./Splitter";
import { WorkbenchSearch, type WorkbenchSearchItem } from "../search/WorkbenchSearch";
import { fileInput, materialInput, rendererChoices } from "../../contrib/surfaces/inputs";
import { WorkbenchSession } from "./session";

export interface WorkbenchKernelProps {
  workspace: CasePresentation;
  stream: "connecting" | "live" | "reconnecting" | "unavailable" | "fixture";
  platform: PlatformServices;
  registry: WorkbenchRegistry;
  readMaterial(input: { case_ref: string; source_ref: string; revision_ref?: string; path: string; expected_generation?: number }): Promise<OperationResult<MaterialReadProjection>>;
  searchCase?: (caseRef: string, query: string) => Promise<OperationResult<readonly CaseSearchResult[]>>;
  refresh(): void;
  openCaseSwitcher(): void;
}

export function WorkbenchKernel({ workspace, stream, platform, registry, readMaterial, searchCase, refresh, openCaseSwitcher }: WorkbenchKernelProps) {
  const containers = useMemo(() => registry.viewContainers(), [registry]);
  const [activeContainer, setActiveContainer] = useState(containers[0]?.id ?? "Overview");
  const [selection, setSelection] = useState(workspace.case.case_ref);
  const [auxiliary, setAuxiliary] = useState("Inspector");
  const [panel, setPanel] = useState("Terminal");
  const [leftOpen, setLeftOpen] = useState(true); const [rightOpen, setRightOpen] = useState(true); const [bottomOpen, setBottomOpen] = useState(true);
  const [leftWidth, setLeftWidth] = useState(() => platform.configuration.get<number>("workbench.sidebar.width") ?? 204); const [rightWidth, setRightWidth] = useState(() => platform.configuration.get<number>("workbench.auxiliary.width") ?? 320);
  const [bottomHeight, setBottomHeight] = useState(() => Math.max(190, Math.floor(window.innerHeight * (platform.configuration.get<number>("workbench.panel.heightRatio") ?? .36))));
  const [bottomLimit, setBottomLimit] = useState(() => Math.max(340, Math.floor(window.innerHeight * .72)));
  const [bottomMaximized, setBottomMaximized] = useState(false);
  const [searchMode, setSearchMode] = useState<"commands" | "open" | "surface" | "case">();
  const [openWith, setOpenWith] = useState(false);
  const [panelToolbarTarget, setPanelToolbarTarget] = useState<HTMLElement | null>(null);
  const [hostState, setHostState] = useState(platform.host.snapshot());
  const restoredBottomHeight = useRef(bottomHeight);
  const windowSession = useMemo(() => new WorkbenchSession(), []);
  const session = windowSession.forCase(workspace.case.case_ref);
  const { surfaces, buffers, navigation } = session;
  const [, invalidateSurfaces] = useState(0);
  const surfaceState = surfaces.snapshot();
  const surfaceArchive = session.archive;
  const applyingHistory = useRef(false);

  useEffect(() => surfaces.subscribe(() => invalidateSurfaces((value) => value + 1)).dispose, [surfaces]);
  useEffect(() => platform.host.subscribe(setHostState).dispose, [platform.host]);
  useEffect(() => () => windowSession.dispose(), [windowSession]);
  const dirtyCount = windowSession.dirtyCount;
  useEffect(() => { void window.__TAURI__?.core.invoke("desktop_set_dirty", { dirty: dirtyCount > 0 }); }, [dirtyCount]);
  useEffect(() => {
    const guard = (event: Event) => {
      if (windowSession.dirtyCount && !window.confirm(`Close Studio and discard ${windowSession.dirtyCount} unsaved file buffer(s)?`)) event.preventDefault();
    };
    const unload = (event: BeforeUnloadEvent) => { if (windowSession.dirtyCount) { event.preventDefault(); event.returnValue = ""; } };
    window.addEventListener("yai:before-window-close", guard);
    window.addEventListener("beforeunload", unload);
    return () => { window.removeEventListener("yai:before-window-close", guard); window.removeEventListener("beforeunload", unload); };
  }, [windowSession]);
  useEffect(() => {
    const current = surfaces.snapshot();
    if (current.inputs.length) {
      const active = current.inputs.find((input) => input.id === current.activeId);
      setSelection(navigation.current()?.selection ?? active?.objectRef ?? workspace.case.case_ref);
      setActiveContainer(navigation.current()?.view ?? active?.viewId ?? "Overview");
      return;
    }
    setSelection(workspace.case.case_ref);
    setActiveContainer("Overview");
    const initial = containers.find((container) => container.id === "Overview")?.surface ?? containers[0]?.surface;
    if (!initial) return;
    surfaceArchive.set(initial.identity, initial);
    surfaces.open(initial);
    navigation.reset({ caseRef: workspace.case.case_ref, view: "Overview", surfaceId: initial.identity, selection: workspace.case.case_ref, auxiliary: "Inspector" });
  }, [containers, navigation, surfaceArchive, surfaces, workspace.case.case_ref]);

  const record = useCallback((view: string, surfaceId: string, nextSelection = selection, nextAuxiliary = auxiliary) => {
    if (applyingHistory.current) return;
    navigation.push({ caseRef: workspace.case.case_ref, view, surfaceId, selection: nextSelection, auxiliary: nextAuxiliary });
  }, [auxiliary, navigation, selection, workspace.case.case_ref]);
  const openSurface = useCallback((input: SurfaceInput, shouldRecord = true) => {
    const capabilities = registry.surfaceRenderer(input.surfaceType)?.capabilities ?? [];
    const previewEnabled = platform.configuration.get<boolean>("workbench.openPreview") !== false;
    const normalized = capabilities.includes("singleton") || !capabilities.includes("previewable") || !previewEnabled
      ? { ...input, id: input.identity, pinned: true }
      : input;
    surfaceArchive.set(normalized.identity, normalized);
    surfaces.open(normalized);
    if (normalized.viewId) setActiveContainer(normalized.viewId);
    const nextSelection = normalized.objectRef ?? (registry.surfaceRenderer(normalized.surfaceType)?.role === "system" ? "settings:general" : workspace.case.case_ref);
    setSelection(nextSelection);
    if (normalized.objectRef) setAuxiliary("Inspector");
    if (shouldRecord) record(normalized.viewId ?? activeContainer, normalized.identity, nextSelection, normalized.objectRef ? "Inspector" : auxiliary);
  }, [activeContainer, auxiliary, platform.configuration, record, registry, surfaces, workspace.case.case_ref, surfaceArchive]);
  const openPerspective = useCallback((id: string, shouldRecord = true) => {
    const container = containers.find((value) => value.id === id);
    if (!container) return;
    setActiveContainer(id);
    openSurface(container.surface, shouldRecord);
  }, [containers, openSurface]);
  const openSettings = useCallback((shouldRecord = true) => {
    const input: SurfaceInput = { id: "settings", identity: "settings", surfaceType: "studio.settings", title: "Settings", icon: "settings", pinned: true };
    surfaceArchive.set(input.identity, input);
    surfaces.open(input);
    setSelection("settings:general");
    if (shouldRecord) record(activeContainer, input.identity, "settings:general");
  }, [activeContainer, record, surfaceArchive, surfaces]);
  const inspect = useCallback((id: string) => {
    setSelection(id); setAuxiliary("Inspector");
    const current = surfaces.snapshot();
    const active = current.inputs.find((input) => input.id === current.activeId);
    record(activeContainer, active?.identity ?? `perspective:${activeContainer}`, id, "Inspector");
  }, [activeContainer, record, surfaces]);
  const surfaceActions = useMemo(() => ({ updateSurface: surfaces.update.bind(surfaces), replaceSurface: surfaces.replace.bind(surfaces) }), [surfaces]);
  const actions = useMemo(() => ({ inspect, openSurface, openPerspective, openSettings, ...surfaceActions }), [inspect, openPerspective, openSettings, openSurface, surfaceActions]);

  const closeSurface = useCallback((id: string) => {
    const input = surfaces.snapshot().inputs.find((candidate) => candidate.id === id);
    if (input?.dirty && !window.confirm(`Discard unsaved changes to ${input.title}?`)) return;
    surfaces.close(id);
    if (input) buffers.discard(input.identity);
    const current = surfaces.snapshot();
    const active = current.inputs.find((candidate) => candidate.id === current.activeId);
    setSelection(active?.objectRef ?? (active?.surfaceType === "studio.settings" ? "settings:general" : workspace.case.case_ref));
  }, [buffers, surfaces, workspace.case.case_ref]);
  const dispatchSurfaceCommand = useCallback((name: string) => window.dispatchEvent(new CustomEvent("yai:surface-command", { detail: { name } })), []);

  const applyLocation = useCallback((location: NavigationLocation | undefined) => {
    if (!location) return;
    applyingHistory.current = true;
    const input = surfaceArchive.get(location.surfaceId);
    if (input) openSurface(input, false);
    else openPerspective(location.view, false);
    setSelection(location.selection ?? workspace.case.case_ref);
    setAuxiliary(location.auxiliary ?? "Inspector");
    applyingHistory.current = false;
  }, [openPerspective, openSurface, surfaceArchive, workspace.case.case_ref]);

  useEffect(() => {
    const registrations = new DisposableStore();
    const command = (id: string, title: string, handler: () => void, predicate?: Parameters<typeof platform.commands.registerCommand>[0]["when"]) => registrations.add(platform.commands.registerCommand({ id, title, handler, when: predicate }));
    command("studio.file.openCase", "Open Case…", openCaseSwitcher);
    command("studio.file.openWith", "Open With…", () => setOpenWith(true), when.truthy("surface.openWith"));
    command("studio.file.save", "Save", () => undefined, when.truthy("surface.saveAvailable"));
    command("studio.file.revert", "Revert File", () => dispatchSurfaceCommand("revert"), when.all(when.truthy("surface.editable"), when.truthy("surface.dirty")));
    command("studio.file.close", "Close Surface", () => surfaceState.activeId && closeSurface(surfaceState.activeId), when.truthy("surface.active"));
    command("studio.file.closeOthers", "Close Others", () => surfaceState.inputs.filter((input) => input.id !== surfaceState.activeId).forEach((input) => closeSurface(input.id)), when.truthy("surface.active"));
    command("studio.file.closeAll", "Close All", () => surfaceState.inputs.forEach((input) => closeSurface(input.id)), when.truthy("surface.active"));
    command("studio.file.settings", "Settings…", () => openSettings());
    command("studio.search.commands", "Show Command Palette", () => setSearchMode("commands"));
    command("studio.search.quickOpen", "Quick Open", () => setSearchMode("open"));
    command("studio.search.currentSurface", "Find in Current Surface", () => {
      const current = surfaces.snapshot();
      const active = current.inputs.find((input) => input.id === current.activeId);
      if (active && registry.surfaceRenderer(active.surfaceType)?.findInRenderer) dispatchSurfaceCommand("find");
      else setSearchMode("surface");
    }, when.truthy("surface.searchable"));
    command("studio.search.case", "Search Current Case", () => setSearchMode("case"));
    command("studio.window.close", "Close Window", platform.host.closeWindow);
    command("studio.edit.undo", "Undo", () => dispatchSurfaceCommand("undo"), when.truthy("surface.editable")); command("studio.edit.redo", "Redo", () => dispatchSurfaceCommand("redo"), when.truthy("surface.editable"));
    command("studio.edit.cut", "Cut", () => document.execCommand("cut")); command("studio.edit.copy", "Copy", () => document.execCommand("copy")); command("studio.edit.paste", "Paste", () => document.execCommand("paste"));
    command("studio.edit.replace", "Replace", () => dispatchSurfaceCommand("replace"), when.truthy("surface.editable")); command("studio.edit.selectAll", "Select All", () => dispatchSurfaceCommand("selectAll"), when.truthy("surface.active"));
    command("studio.view.toggleExplorer", "Explorer", () => setLeftOpen((value) => !value));
    command("studio.view.toggleContext", "Context Panel", () => setRightOpen((value) => !value));
    command("studio.view.toggleBottomPanel", "Bottom Panel", () => setBottomOpen((value) => !value));
    command("studio.view.resetLayout", "Reset Layout", () => { setLeftOpen(true); setRightOpen(true); setBottomOpen(true); platform.configuration.update("workbench.sidebar.width", 204); platform.configuration.update("workbench.auxiliary.width", 320); platform.configuration.update("workbench.panel.heightRatio", .36); setLeftWidth(204); setRightWidth(320); const height = Math.max(260, Math.floor(window.innerHeight * .36)); restoredBottomHeight.current = height; setBottomHeight(height); setBottomMaximized(false); });
    command("studio.go.back", "Back", () => applyLocation(navigation.back()), () => navigation.canBack());
    command("studio.go.forward", "Forward", () => applyLocation(navigation.forward()), () => navigation.canForward());
    const adjacentSurface = (delta: number) => {
      const current = surfaces.snapshot();
      const index = current.inputs.findIndex((input) => input.id === current.activeId);
      const target = current.inputs[(index + delta + current.inputs.length) % current.inputs.length];
      if (target) openSurface(target);
    };
    command("studio.go.previousTab", "Previous Tab", () => adjacentSurface(-1)); command("studio.go.nextTab", "Next Tab", () => adjacentSurface(1));
    containers.forEach((container) => command(`studio.case.${container.id.toLowerCase()}`, container.title, () => openPerspective(container.id)));
    command("studio.case.refresh", "Refresh / Resync Case", refresh, when.equals("studio.data.live", true));
    const terminal = (name: "new" | "kill" | "clear" | "focus") => { setPanel("Terminal"); setBottomOpen(true); requestAnimationFrame(() => dispatchTerminalCommand(name)); };
    command("studio.terminal.new", "New Terminal", () => terminal("new"), when.truthy("terminal.available"));
    command("studio.terminal.kill", "Kill Terminal", () => terminal("kill"), when.truthy("terminal.available"));
    command("studio.terminal.clear", "Clear", () => terminal("clear"), when.truthy("terminal.available"));
    command("studio.terminal.focus", "Focus Terminal", () => terminal("focus"), when.truthy("terminal.available"));
    command("studio.help.about", "About YAI Studio", () => openSettings());
    const menu = (location: Parameters<typeof platform.menus.registerMenuItem>[0]["location"], group: string, order: number, commandId: string, checkedWhen?: Parameters<typeof platform.menus.registerMenuItem>[0]["checkedWhen"]) => registrations.add(platform.menus.registerMenuItem({ id: `${location}:${commandId}`, location, group, order, command: commandId, checkedWhen }));
    menu("YAI", "1", 0, "studio.file.settings"); menu("YAI", "2", 0, "studio.window.close");
    menu("File", "1", 0, "studio.file.openCase"); menu("File", "1", 1, "studio.search.quickOpen"); menu("File", "2", 0, "studio.file.openWith"); menu("File", "2", 1, "studio.file.save"); menu("File", "2", 2, "studio.file.revert"); menu("File", "3", 0, "studio.file.close"); menu("File", "3", 1, "studio.file.closeOthers"); menu("File", "3", 2, "studio.file.closeAll"); menu("File", "4", 0, "studio.file.settings"); menu("File", "5", 0, "studio.window.close");
    ["undo", "redo", "cut", "copy", "paste"].forEach((name, order) => menu("Edit", "1", order, `studio.edit.${name}`));
    menu("Edit", "2", 0, "studio.search.currentSurface"); menu("Edit", "2", 1, "studio.edit.replace"); menu("Edit", "2", 2, "studio.edit.selectAll"); menu("Edit", "3", 0, "studio.search.case");
    menu("View", "0", 0, "studio.search.commands"); menu("View", "0", 1, "studio.search.quickOpen"); menu("View", "1", 0, "studio.view.toggleExplorer", when.truthy("sidebar.visible")); menu("View", "1", 1, "studio.view.toggleContext", when.truthy("auxiliary.visible")); menu("View", "1", 2, "studio.view.toggleBottomPanel", when.truthy("panel.visible")); menu("View", "2", 0, "studio.view.resetLayout");
    menu("Go", "1", 0, "studio.go.back"); menu("Go", "1", 1, "studio.go.forward"); menu("Go", "2", 0, "studio.go.previousTab"); menu("Go", "2", 1, "studio.go.nextTab");
    containers.forEach((container, order) => menu("Case", "1", order, `studio.case.${container.id.toLowerCase()}`, when.equals("view.active", container.id))); menu("Case", "2", 0, "studio.case.refresh"); menu("Case", "3", 0, "studio.file.openCase");
    ["new", "kill", "clear", "focus"].forEach((name, order) => menu("Terminal", "1", order, `studio.terminal.${name}`)); menu("Terminal", "2", 0, "studio.view.toggleBottomPanel"); menu("Help", "1", 0, "studio.help.about");
    const binding = (id: string, commandId: string, key: string, allowInTerminal = false) => registrations.add(platform.keybindings.registerKeybinding({ id, command: commandId, key, allowInTerminal, precedence: ["case-search", "quick-open", "command-palette", "toggle-panel", "terminal-focus", "terminal-new"].includes(id) ? "workbench" : undefined }));
    binding("close-surface", "studio.file.close", "Mod+w"); binding("settings", "studio.file.settings", "Mod+,");
    binding("toggle-panel", "studio.view.toggleBottomPanel", "Mod+j", true); binding("terminal-focus", "studio.terminal.focus", "Mod+`", true); binding("terminal-new", "studio.terminal.new", "Mod+Shift+`", true); binding("open-case", "studio.file.openCase", "Mod+o"); binding("toggle-sidebar", "studio.view.toggleExplorer", "Mod+b"); binding("toggle-auxiliary", "studio.view.toggleContext", "Mod+Shift+b"); binding("go-back", "studio.go.back", "Alt+ArrowLeft"); binding("go-forward", "studio.go.forward", "Alt+ArrowRight"); binding("command-palette", "studio.search.commands", "Mod+Shift+p"); binding("quick-open", "studio.search.quickOpen", "Mod+p"); binding("surface-find", "studio.search.currentSurface", "Mod+f"); binding("case-search", "studio.search.case", "Mod+Shift+f");
    containers.forEach((container, order) => binding(`perspective-${container.id}`, `studio.case.${container.id.toLowerCase()}`, `Mod+${order + 1}`));
    const focus = (event: FocusEvent) => {
      const candidate = event.type === "focusout" ? event.relatedTarget : event.target;
      platform.context.update("terminal.focused", Boolean((candidate as Element | null)?.closest?.(".xterm")));
    };
    document.addEventListener("focusin", focus); document.addEventListener("focusout", focus);
    registrations.add(toDisposable(() => { document.removeEventListener("focusin", focus); document.removeEventListener("focusout", focus); }));
    registrations.add(platform.keybindings.attach());
    return () => registrations.dispose();
  }, [applyLocation, closeSurface, containers, dispatchSurfaceCommand, openCaseSwitcher, openPerspective, openSettings, platform, refresh, surfaceState.activeId, surfaceState.inputs, surfaces]);

  useEffect(() => {
    platform.context.update("studio.data.live", workspace.presentation.dataKind === "live");
    platform.context.update("studio.data.fixture", workspace.presentation.dataKind === "fixture");
    platform.context.update("case.attached", true); platform.context.update("view.active", activeContainer);
    platform.context.update("panel.active", panel); platform.context.update("sidebar.visible", leftOpen);
    platform.context.update("auxiliary.visible", rightOpen); platform.context.update("panel.visible", bottomOpen);
    platform.context.update("selection.kind", selection === workspace.case.case_ref ? "case" : "object");
    platform.context.update("case.hasWorkflow", workspace.work.status !== "empty"); platform.context.update("case.hasProvider", workspace.compute.targets.length > 0);
    const currentInput = surfaceState.inputs.find((input) => input.id === surfaceState.activeId);
    const capabilities = currentInput ? registry.surfaceRenderer(currentInput.surfaceType)?.capabilities ?? [] : [];
    platform.context.update("surface.active", Boolean(currentInput));
    platform.context.update("surface.searchable", capabilities.includes("searchable"));
    platform.context.update("surface.editable", capabilities.includes("editable"));
    platform.context.update("surface.dirty", Boolean(currentInput?.dirty));
    platform.context.update("surface.saveAvailable", false);
    platform.context.update("surface.openWith", Boolean(currentInput?.metadata?.mediaType && rendererChoices(currentInput.metadata.mediaType).length > 1));
  }, [activeContainer, bottomOpen, leftOpen, panel, platform.context, registry, rightOpen, selection, surfaceState.activeId, surfaceState.inputs, workspace]);
  useEffect(() => { const resize = () => { const limit = Math.max(340, Math.floor(window.innerHeight * .72)); setBottomLimit(limit); setBottomHeight((value) => Math.max(190, Math.min(limit, value))); }; window.addEventListener("resize", resize); return () => window.removeEventListener("resize", resize); }, []);
  useEffect(() => {
    const apply = () => document.documentElement.toggleAttribute("data-reduce-motion", platform.configuration.get<boolean>("appearance.reducedMotion") === true);
    apply();
    const subscription = platform.configuration.subscribe((key) => key === "appearance.reducedMotion" && apply());
    return () => subscription.dispose();
  }, [platform.configuration]);

  useEffect(() => platform.configuration.subscribe(key => {
    if (key === "workbench.sidebar.width") setLeftWidth(platform.configuration.get<number>(key) ?? 204);
    if (key === "workbench.auxiliary.width") setRightWidth(platform.configuration.get<number>(key) ?? 320);
    if (key === "workbench.panel.heightRatio") {
      const height = Math.max(190, Math.floor(window.innerHeight * (platform.configuration.get<number>(key) ?? .36)));
      restoredBottomHeight.current = height; setBottomHeight(height); setBottomMaximized(false);
    }
  }).dispose, [platform.configuration]);

  const activeInput = surfaceState.inputs.find((input) => input.id === surfaceState.activeId);
  const surfaceRenderer = activeInput ? registry.surfaceRenderer(activeInput.surfaceType) : undefined;
  const sidebarViews = registry.viewsFor(activeContainer);
  const panelViews = registry.panelViews(); const activePanel = panelViews.find((view) => view.id === panel) ?? panelViews[0];
  const auxiliaryViews = registry.auxiliaryViews(); const activeAuxiliary = auxiliaryViews.find((view) => view.id === auxiliary) ?? auxiliaryViews[0];
  const inspector = auxiliary === "Inspector" ? registry.inspector("default") : undefined;
  const renderContext = { workspace, selection, actions, platform, settings: registry.settings, readMaterial, buffers };
  const SurfaceComponent = surfaceRenderer?.component;

  const AuxiliaryComponent = inspector?.component ?? activeAuxiliary?.component;
  const commandItems = (): WorkbenchSearchItem[] => platform.commands.entries().map((command) => ({ id: command.id, label: command.title, detail: platform.keybindings.shortcutFor(command.id), category: "Command", icon: "arrow", disabled: !command.enabled, run: () => void platform.commands.executeCommand(command.id) }));
  const quickItems = (): WorkbenchSearchItem[] => {
    const known = new Map<string, WorkbenchSearchItem>();
    surfaceState.inputs.forEach((input) => known.set(input.identity, { id: `open:${input.identity}`, label: input.title, detail: input.surfaceType, category: "Open Surface", icon: input.icon, run: () => openSurface(input) }));
    for (const material of workspace.presentation.materials ?? []) {
      const input = materialInput(workspace, material.id, material.name);
      if (!known.has(input.identity)) known.set(input.identity, { id: input.identity, label: material.name, detail: material.path, category: "Case Material", icon: input.icon, run: () => openSurface(input) });
    }
    for (const file of workspace.environment.files) {
      const input = fileInput(workspace, file.id);
      if (!known.has(input.identity)) known.set(input.identity, { id: input.identity, label: input.title, detail: file.path, category: "Exposed Case File", icon: input.icon, run: () => openSurface(input) });
    }
    return [...known.values()];
  };
  const surfaceItems = async (query: string): Promise<WorkbenchSearchItem[]> => {
    if (!activeInput || !surfaceRenderer?.search) return [];
    const results = await surfaceRenderer.search(renderContext, activeInput, query);
    return results.map((result) => ({ id: result.id, label: result.label, detail: result.detail, category: activeInput.title, icon: activeInput.icon, run: () => result.objectRef && inspect(result.objectRef) }));
  };
  const caseItems = async (query: string): Promise<WorkbenchSearchItem[]> => {
    if (!searchCase) return [{ id: "case-search-unavailable", label: "Case search unavailable", detail: "The current YAI application boundary exposes no qualified Case search query.", category: "Case Search", icon: "warning", disabled: true, run() {} }];
    if (!query.trim()) return [];
    const result = await searchCase(workspace.case.case_ref, query);
    if (result.result_state !== "success") return [{ id: "case-search-refused", label: "Case search refused", detail: result.error?.safe_message ?? result.result_state, category: "Case Search", icon: "warning", disabled: true, run() {} }];
    return (result.data ?? []).map((item) => ({ id: item.id, label: item.label, detail: item.detail, category: "Case Search", icon: "search", run: () => item.object_ref && openSurface(materialInput(workspace, item.object_ref, item.label)) }));
  };
  return <div className="live-case-shell workbench-kernel" data-case-source={workspace.presentation.dataKind} data-host={platform.host.capabilities.kind} style={{ "--left-width": `${leftWidth}px`, "--right-width": `${rightWidth}px`, "--bottom-height": `${bottomHeight}px` } as React.CSSProperties}>
    <nav className="case-titlebar" aria-label="Application and workspace controls" data-tauri-drag-region onPointerDown={beginDesktopWindowDrag} onDoubleClick={toggleDesktopWindowMaximize}><ApplicationMenuBar platform={platform} /><div className="titlebar-center" data-tauri-drag-region><IconButton aria-label="Back" disabled={!navigation.canBack()} onClick={() => applyLocation(navigation.back())}><Icon name="back" /></IconButton><IconButton aria-label="Forward" disabled={!navigation.canForward()} onClick={() => applyLocation(navigation.forward())}><Icon name="forward" /></IconButton><button className="titlebar-case" title={workspace.case.case_ref} onClick={openCaseSwitcher}><Icon name="case" size={14} /><span>{workspace.case.display_name}</span><Icon name="chevron" size={12} /></button></div><div className="titlebar-layout">{workspace.presentation.dataKind === "fixture" && <Badge tone="warning">Fixture data</Badge>}{stream !== "live" && stream !== "fixture" && <Badge tone={stream === "unavailable" ? "error" : "warning"}>{stream}</Badge>}<IconButton aria-label="Refresh Case" onClick={refresh}><Icon name="refresh" /></IconButton><IconButton aria-label="Toggle Case sidebar" onClick={() => setLeftOpen((value) => !value)}><Icon name="left" /></IconButton><IconButton aria-label="Toggle bottom panel" onClick={() => setBottomOpen((value) => !value)}><Icon name="bottom" /></IconButton><IconButton aria-label="Toggle context panel" onClick={() => setRightOpen((value) => !value)}><Icon name="right" /></IconButton><DesktopWindowControls /></div></nav>
    <div className="live-workbench">
      <aside className="live-rail" aria-label="Case perspectives">{containers.map((container) => <button key={container.id} aria-label={container.title} aria-pressed={activeContainer === container.id} onClick={() => openPerspective(container.id)}><Icon name={container.icon} size={20} /><span role="tooltip">{container.title}</span></button>)}</aside>
      {leftOpen && <><aside className="live-sidebar" id="case-sidebar"><PanelHeader title={containers.find((value) => value.id === activeContainer)?.title ?? activeContainer} />{sidebarViews.map((view) => { const ViewComponent = view.component; return <ViewComponent key={view.id} {...renderContext} containerId={activeContainer} />; })}</aside><Splitter label="Resize Case explorer" axis="x" value={leftWidth} min={170} max={320} set={value => { setLeftWidth(value); platform.configuration.update("workbench.sidebar.width", value); }} /></>}
      <section className="live-center"><SurfaceTabs inputs={surfaceState.inputs} activeId={surfaceState.activeId} describe={(input) => { const renderer = registry.surfaceRenderer(input.surfaceType); return {role: renderer?.role ?? "content", pinnable: renderer?.capabilities.includes("pinnable") ?? false}; }} activate={openSurface} pin={(id) => surfaces.pin(id)} close={closeSurface} /><main className="live-surface">{activeInput && SurfaceComponent ? <Suspense fallback={<div className="surface-loading">Loading {activeInput.title}…</div>}><SurfaceComponent {...renderContext} input={activeInput} /></Suspense> : <div className="empty-surface"><h2>{activeInput ? "Surface unavailable" : "Your workspace is ready"}</h2><p>{activeInput ? `No trusted renderer is registered for ${activeInput.title}.` : "Open a Case perspective from the sidebar or find a file with Quick Open."}</p>{!activeInput && <button onClick={() => setSearchMode("open")}>Quick Open</button>}</div>}</main>
        {bottomOpen && <Splitter label="Resize bottom panel" axis="y" reverse value={bottomHeight} min={190} max={bottomLimit} set={(value) => { setBottomMaximized(false); restoredBottomHeight.current = value; setBottomHeight(value); platform.configuration.update("workbench.panel.heightRatio", Math.max(.2, Math.min(.72, value / window.innerHeight))); }} />}
        <section hidden={!bottomOpen} className="live-bottom" id="case-tools" style={{ height: bottomHeight }} aria-label="Bottom tools"><header>{panelViews.map((view) => <button key={view.id} aria-pressed={panel === view.id} onClick={() => setPanel(view.id)}>{view.title}</button>)}<span /><div className="panel-contribution-toolbar" ref={setPanelToolbarTarget} /><IconButton aria-label={bottomMaximized ? "Restore bottom panel" : "Maximize bottom panel"} onClick={() => { if (bottomMaximized) { setBottomHeight(restoredBottomHeight.current); setBottomMaximized(false); } else { restoredBottomHeight.current = bottomHeight; setBottomHeight(bottomLimit); setBottomMaximized(true); } }}><Icon name={bottomMaximized ? "restore" : "maximize"} size={14} /></IconButton><IconButton aria-label="Close bottom panel" onClick={() => setBottomOpen(false)}><Icon name="close" size={14} /></IconButton></header><div className="tool-content">{panelViews.map((view) => { const Component = view.component; const visible = bottomOpen && activePanel?.id === view.id; return <div key={view.id} className="tool-pane" hidden={!visible}><Component {...renderContext} visible={visible} available={platform.host.capabilities.terminalAvailable} toolbarTarget={visible ? panelToolbarTarget : null} closePanel={() => setBottomOpen(false)} /></div>; })}</div></section>
      </section>
      {rightOpen && <><Splitter label="Resize context panel" axis="x" reverse value={rightWidth} min={290} max={470} set={value => { setRightWidth(value); platform.configuration.update("workbench.auxiliary.width", value); }} /><aside className="live-context" id="case-conversation"><header><div className="segmented">{auxiliaryViews.map((view) => <button key={view.id} aria-pressed={auxiliary === view.id} onClick={() => setAuxiliary(view.id)}>{view.title}</button>)}</div><IconButton aria-label="Close context panel" onClick={() => setRightOpen(false)}><Icon name="right" /></IconButton></header>{AuxiliaryComponent && <AuxiliaryComponent {...renderContext} />}</aside></>}
    </div>
    <footer className="kernel-status" aria-label="Workbench status">
      <div><span className="case-status" data-status={workspace.case.case_status}>{workspace.case.case_status}</span><span>Generation {workspace.case.generation}</span><span>{activeInput?.title ?? activeContainer}</span></div>
      <div><button className="host-status" data-state={hostState.state} onClick={() => { openSettings(); setSelection("settings:yai-host"); }} title="Open Settings > YAI Host">YAI {hostState.state === "live" ? "●" : hostState.state}</button>{workspace.presentation.dataKind === "fixture" && <span>Fixture data</span>}<span>{workspace.case.participant_ref}</span></div>
    </footer>
    {searchMode === "commands" && <WorkbenchSearch title="Command Palette" placeholder="Type a command" items={commandItems()} onClose={() => setSearchMode(undefined)} />}
    {searchMode === "open" && <WorkbenchSearch title="Quick Open" placeholder="Search open Surfaces and exposed Case material" items={quickItems()} onClose={() => setSearchMode(undefined)} />}
    {searchMode === "surface" && <WorkbenchSearch title={`Find in ${activeInput?.title ?? "Surface"}`} placeholder="Search current Surface" resolveItems={surfaceItems} onClose={() => setSearchMode(undefined)} />}
    {searchMode === "case" && <WorkbenchSearch title="Search Current Case" placeholder={searchCase ? "Search qualified Case presentation" : "Case search unavailable"} resolveItems={caseItems} onClose={() => setSearchMode(undefined)} />}
    {openWith && activeInput && <OpenWith title={activeInput.title} selected={activeInput.surfaceType} choices={rendererChoices(activeInput.metadata?.mediaType)} close={() => setOpenWith(false)} choose={(type) => { if (activeInput.objectRef) surfaces.replace(activeInput.id, { ...fileInput(workspace, activeInput.objectRef, activeInput.pinned, type), id: activeInput.id, dirty: activeInput.dirty }); }} />}
  </div>;
}
