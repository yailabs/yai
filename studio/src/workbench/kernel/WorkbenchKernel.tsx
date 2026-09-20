import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CasePresentation, CaseSearchResult } from "../../clients/dataSource";
import type { OperationResult } from "../../clients/live";
import { Icon } from "../../components/Icon";
import { Badge, IconButton, PanelHeader } from "../../components/primitives";
import { DesktopWindowControls } from "../../app/DesktopWindowControls";
import { beginDesktopWindowDrag, toggleDesktopWindowMaximize } from "../../platform/desktopWindow";
import { dispatchTerminalCommand } from "../../terminal/TerminalPanel";
import type { PlatformServices } from "../../platform/services";
import { DisposableStore, toDisposable } from "../../platform/lifecycle";
import { when } from "../../platform/context";
import type { WorkbenchRegistry } from "./registry";
import { SurfaceGroupService, type SurfaceInput } from "../surface/model";
import type { NavigationLocation } from "../../platform/navigation";
import { ApplicationMenuBar } from "./ApplicationMenuBar";
import { Splitter } from "./Splitter";
import { WorkbenchSearch, type WorkbenchSearchItem } from "../search/WorkbenchSearch";
import { materialInput } from "../../contrib/surfaces/inputs";

export interface WorkbenchKernelProps {
  workspace: CasePresentation;
  stream: "connecting" | "live" | "reconnecting" | "unavailable" | "fixture";
  platform: PlatformServices;
  registry: WorkbenchRegistry;
  searchCase?: (caseRef: string, query: string) => Promise<OperationResult<readonly CaseSearchResult[]>>;
  refresh(): void;
  openCaseSwitcher(): void;
}

export function WorkbenchKernel({ workspace, stream, platform, registry, searchCase, refresh, openCaseSwitcher }: WorkbenchKernelProps) {
  const containers = useMemo(() => registry.viewContainers(), [registry]);
  const [activeContainer, setActiveContainer] = useState(containers[0]?.id ?? "Overview");
  const [selection, setSelection] = useState(workspace.case.case_ref);
  const [auxiliary, setAuxiliary] = useState("Inspector");
  const [panel, setPanel] = useState("Terminal");
  const [leftOpen, setLeftOpen] = useState(true); const [rightOpen, setRightOpen] = useState(true); const [bottomOpen, setBottomOpen] = useState(true);
  const [leftWidth, setLeftWidth] = useState(204); const [rightWidth, setRightWidth] = useState(320);
  const [bottomHeight, setBottomHeight] = useState(() => Math.max(260, Math.floor(window.innerHeight * .36)));
  const [bottomLimit, setBottomLimit] = useState(() => Math.max(340, Math.floor(window.innerHeight * .72)));
  const [bottomMaximized, setBottomMaximized] = useState(false);
  const [searchMode, setSearchMode] = useState<"commands" | "open" | "surface" | "case">();
  const [panelToolbarTarget, setPanelToolbarTarget] = useState<HTMLElement | null>(null);
  const restoredBottomHeight = useRef(bottomHeight);
  const surfaces = useMemo(() => new SurfaceGroupService(), []);
  const [surfaceState, setSurfaceState] = useState(surfaces.snapshot());
  const surfaceArchive = useRef(new Map<string, SurfaceInput>());
  const applyingHistory = useRef(false);

  useEffect(() => surfaces.subscribe(() => setSurfaceState(surfaces.snapshot())).dispose, [surfaces]);
  useEffect(() => () => surfaces.dispose(), [surfaces]);
  useEffect(() => {
    const initial = containers.find((container) => container.id === "Overview")?.surface ?? containers[0]?.surface;
    if (!initial) return;
    surfaceArchive.current.set(initial.identity, initial);
    surfaces.open(initial);
    platform.navigation.reset({ caseRef: workspace.case.case_ref, view: "Overview", surfaceId: initial.identity, selection: workspace.case.case_ref, auxiliary: "Inspector" });
  }, [containers, platform.navigation, surfaces, workspace.case.case_ref]);

  const record = useCallback((view: string, surfaceId: string, nextSelection = selection, nextAuxiliary = auxiliary) => {
    if (applyingHistory.current) return;
    platform.navigation.push({ caseRef: workspace.case.case_ref, view, surfaceId, selection: nextSelection, auxiliary: nextAuxiliary });
  }, [auxiliary, platform.navigation, selection, workspace.case.case_ref]);
  const openSurface = useCallback((input: SurfaceInput, shouldRecord = true) => {
    const capabilities = registry.surfaceRenderer(input.surfaceType)?.capabilities ?? [];
    const previewEnabled = platform.configuration.get<boolean>("workbench.openPreview") !== false;
    const normalized = capabilities.includes("singleton") || !capabilities.includes("previewable") || !previewEnabled
      ? { ...input, id: input.identity, pinned: true }
      : input;
    surfaceArchive.current.set(normalized.identity, normalized);
    surfaces.open(normalized);
    if (normalized.viewId) setActiveContainer(normalized.viewId);
    if (normalized.objectRef) { setSelection(normalized.objectRef); setAuxiliary("Inspector"); }
    if (shouldRecord) record(normalized.viewId ?? activeContainer, normalized.identity, normalized.objectRef ?? selection, normalized.objectRef ? "Inspector" : auxiliary);
  }, [activeContainer, auxiliary, platform.configuration, record, registry, selection, surfaces]);
  const openPerspective = useCallback((id: string, shouldRecord = true) => {
    const container = containers.find((value) => value.id === id);
    if (!container) return;
    setActiveContainer(id);
    openSurface(container.surface, shouldRecord);
  }, [containers, openSurface]);
  const openSettings = useCallback((shouldRecord = true) => {
    const input: SurfaceInput = { id: "settings", identity: "settings", surfaceType: "studio.settings", title: "Settings", icon: "settings", pinned: true };
    surfaceArchive.current.set(input.identity, input);
    surfaces.open(input);
    if (shouldRecord) record("Settings", input.identity, "settings:general");
  }, [record, surfaces]);
  const inspect = useCallback((id: string) => {
    setSelection(id); setAuxiliary("Inspector");
    const active = surfaceState.inputs.find((input) => input.id === surfaceState.activeId);
    record(activeContainer, active?.identity ?? `perspective:${activeContainer}`, id, "Inspector");
  }, [activeContainer, record, surfaceState.activeId, surfaceState.inputs]);
  const actions = useMemo(() => ({ inspect, openSurface, openPerspective, openSettings }), [inspect, openPerspective, openSettings, openSurface]);

  const applyLocation = useCallback((location: NavigationLocation | undefined) => {
    if (!location) return;
    applyingHistory.current = true;
    setSelection(location.selection ?? workspace.case.case_ref);
    setAuxiliary(location.auxiliary ?? "Inspector");
    const input = surfaceArchive.current.get(location.surfaceId);
    if (input) openSurface(input, false);
    else openPerspective(location.view, false);
    applyingHistory.current = false;
  }, [openPerspective, openSurface, workspace.case.case_ref]);

  useEffect(() => {
    const registrations = new DisposableStore();
    const command = (id: string, title: string, handler: () => void, predicate?: Parameters<typeof platform.commands.registerCommand>[0]["when"]) => registrations.add(platform.commands.registerCommand({ id, title, handler, when: predicate }));
    command("studio.file.openCase", "Open Case…", openCaseSwitcher);
    command("studio.file.settings", "Settings…", () => openSettings());
    command("studio.search.commands", "Show Command Palette", () => setSearchMode("commands"));
    command("studio.search.quickOpen", "Quick Open", () => setSearchMode("open"));
    command("studio.search.currentSurface", "Find in Current Surface", () => setSearchMode("surface"), when.truthy("surface.searchable"));
    command("studio.search.case", "Search Current Case", () => setSearchMode("case"));
    command("studio.window.close", "Close Window", platform.host.closeWindow);
    command("studio.edit.cut", "Cut", () => document.execCommand("cut")); command("studio.edit.copy", "Copy", () => document.execCommand("copy")); command("studio.edit.paste", "Paste", () => document.execCommand("paste"));
    command("studio.view.toggleExplorer", "Explorer", () => setLeftOpen((value) => !value));
    command("studio.view.toggleContext", "Context Panel", () => setRightOpen((value) => !value));
    command("studio.view.toggleBottomPanel", "Bottom Panel", () => setBottomOpen((value) => !value));
    command("studio.view.resetLayout", "Reset Layout", () => { setLeftOpen(true); setRightOpen(true); setBottomOpen(true); setLeftWidth(204); setRightWidth(320); const height = Math.max(260, Math.floor(window.innerHeight * .36)); restoredBottomHeight.current = height; setBottomHeight(height); setBottomMaximized(false); });
    command("studio.go.back", "Back", () => applyLocation(platform.navigation.back()), () => platform.navigation.canBack());
    command("studio.go.forward", "Forward", () => applyLocation(platform.navigation.forward()), () => platform.navigation.canForward());
    command("studio.go.previousTab", "Previous Tab", () => surfaces.move(-1)); command("studio.go.nextTab", "Next Tab", () => surfaces.move(1));
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
    menu("File", "1", 0, "studio.file.openCase"); menu("File", "2", 0, "studio.file.settings"); menu("File", "3", 0, "studio.window.close");
    ["cut", "copy", "paste"].forEach((name, order) => menu("Edit", "1", order, `studio.edit.${name}`));
    menu("Edit", "2", 0, "studio.search.currentSurface"); menu("Edit", "2", 1, "studio.search.case");
    menu("View", "0", 0, "studio.search.commands"); menu("View", "0", 1, "studio.search.quickOpen"); menu("View", "1", 0, "studio.view.toggleExplorer", when.truthy("sidebar.visible")); menu("View", "1", 1, "studio.view.toggleContext", when.truthy("auxiliary.visible")); menu("View", "1", 2, "studio.view.toggleBottomPanel", when.truthy("panel.visible")); menu("View", "2", 0, "studio.view.resetLayout");
    menu("Go", "1", 0, "studio.go.back"); menu("Go", "1", 1, "studio.go.forward"); menu("Go", "2", 0, "studio.go.previousTab"); menu("Go", "2", 1, "studio.go.nextTab");
    containers.forEach((container, order) => menu("Case", "1", order, `studio.case.${container.id.toLowerCase()}`, when.equals("view.active", container.id))); menu("Case", "2", 0, "studio.case.refresh"); menu("Case", "3", 0, "studio.file.openCase");
    ["new", "kill", "clear", "focus"].forEach((name, order) => menu("Terminal", "1", order, `studio.terminal.${name}`)); menu("Terminal", "2", 0, "studio.view.toggleBottomPanel"); menu("Help", "1", 0, "studio.help.about");
    const binding = (id: string, commandId: string, key: string, allowInTerminal = false) => registrations.add(platform.keybindings.registerKeybinding({ id, command: commandId, key, allowInTerminal }));
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
  }, [applyLocation, containers, openCaseSwitcher, openPerspective, openSettings, platform, refresh, surfaces]);

  useEffect(() => {
    platform.context.update("studio.data.live", workspace.presentation.dataKind === "live");
    platform.context.update("studio.data.fixture", workspace.presentation.dataKind === "fixture");
    platform.context.update("case.attached", true); platform.context.update("view.active", activeContainer);
    platform.context.update("panel.active", panel); platform.context.update("sidebar.visible", leftOpen);
    platform.context.update("auxiliary.visible", rightOpen); platform.context.update("panel.visible", bottomOpen);
    platform.context.update("selection.kind", selection === workspace.case.case_ref ? "case" : "object");
    platform.context.update("case.hasWorkflow", workspace.work.status !== "empty"); platform.context.update("case.hasProvider", workspace.compute.targets.length > 0);
    const currentInput = surfaceState.inputs.find((input) => input.id === surfaceState.activeId);
    platform.context.update("surface.searchable", Boolean(currentInput && registry.surfaceRenderer(currentInput.surfaceType)?.capabilities.includes("searchable")));
  }, [activeContainer, bottomOpen, leftOpen, panel, platform.context, registry, rightOpen, selection, surfaceState.activeId, surfaceState.inputs, workspace]);
  useEffect(() => { const resize = () => { const limit = Math.max(340, Math.floor(window.innerHeight * .72)); setBottomLimit(limit); setBottomHeight((value) => Math.max(190, Math.min(limit, value))); }; window.addEventListener("resize", resize); return () => window.removeEventListener("resize", resize); }, []);
  useEffect(() => {
    const apply = () => document.documentElement.toggleAttribute("data-reduce-motion", platform.configuration.get<boolean>("appearance.reducedMotion") === true);
    apply();
    const subscription = platform.configuration.subscribe((key) => key === "appearance.reducedMotion" && apply());
    return () => subscription.dispose();
  }, [platform.configuration]);

  const activeInput = surfaceState.inputs.find((input) => input.id === surfaceState.activeId);
  const surfaceRenderer = activeInput ? registry.surfaceRenderer(activeInput.surfaceType) : undefined;
  const sidebarViews = registry.viewsFor(activeContainer);
  const panelViews = registry.panelViews(); const activePanel = panelViews.find((view) => view.id === panel) ?? panelViews[0];
  const auxiliaryViews = registry.auxiliaryViews(); const activeAuxiliary = auxiliaryViews.find((view) => view.id === auxiliary) ?? auxiliaryViews[0];
  const inspector = auxiliary === "Inspector" ? registry.inspector("default") : undefined;
  const renderContext = { workspace, selection, actions, platform, settings: registry.settings };
  const SurfaceComponent = surfaceRenderer?.component;
  const PanelComponent = activePanel?.component;
  const AuxiliaryComponent = inspector?.component ?? activeAuxiliary?.component;
  const commandItems = (): WorkbenchSearchItem[] => platform.commands.entries().map((command) => ({ id: command.id, label: command.title, detail: platform.keybindings.shortcutFor(command.id), category: "Command", icon: "arrow", disabled: !command.enabled, run: () => void platform.commands.executeCommand(command.id) }));
  const quickItems = (): WorkbenchSearchItem[] => {
    const known = new Map<string, WorkbenchSearchItem>();
    surfaceState.inputs.forEach((input) => known.set(input.identity, { id: `open:${input.identity}`, label: input.title, detail: input.surfaceType, category: "Open Surface", icon: input.icon, run: () => openSurface(input) }));
    for (const material of workspace.presentation.materials ?? []) known.set(material.id, { id: `material:${material.id}`, label: material.name, detail: `${material.mediaType} · ${material.path}`, category: "Case Material", icon: "file", run: () => openSurface(materialInput(workspace, material.id, material.name)) });
    for (const file of workspace.environment.files) if (!known.has(file.id)) known.set(file.id, { id: `file:${file.id}`, label: file.path.split("/").at(-1) ?? file.path, detail: file.path, category: "Exposed Case File", icon: "file", run: () => openSurface(materialInput(workspace, file.id, file.path.split("/").at(-1) ?? file.path)) });
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
    <nav className="case-titlebar" aria-label="Application and workspace controls" data-tauri-drag-region onPointerDown={beginDesktopWindowDrag} onDoubleClick={toggleDesktopWindowMaximize}><ApplicationMenuBar platform={platform} /><div className="titlebar-center" data-tauri-drag-region><IconButton aria-label="Back" disabled={!platform.navigation.canBack()} onClick={() => applyLocation(platform.navigation.back())}><Icon name="back" /></IconButton><IconButton aria-label="Forward" disabled={!platform.navigation.canForward()} onClick={() => applyLocation(platform.navigation.forward())}><Icon name="forward" /></IconButton><button className="titlebar-case" title={workspace.case.case_ref} onClick={openCaseSwitcher}><Icon name="case" size={14} /><span>{workspace.case.display_name}</span><Icon name="chevron" size={12} /></button></div><div className="titlebar-layout">{workspace.presentation.dataKind === "fixture" && <Badge tone="warning">Fixture data</Badge>}{stream !== "live" && stream !== "fixture" && <Badge tone={stream === "unavailable" ? "error" : "warning"}>{stream}</Badge>}<IconButton aria-label="Refresh Case" onClick={refresh}><Icon name="refresh" /></IconButton><IconButton aria-label="Toggle Case sidebar" onClick={() => setLeftOpen((value) => !value)}><Icon name="left" /></IconButton><IconButton aria-label="Toggle bottom panel" onClick={() => setBottomOpen((value) => !value)}><Icon name="bottom" /></IconButton><IconButton aria-label="Toggle context panel" onClick={() => setRightOpen((value) => !value)}><Icon name="right" /></IconButton><DesktopWindowControls /></div></nav>
    <div className="live-workbench">
      <aside className="live-rail" aria-label="Case perspectives">{containers.map((container) => <button key={container.id} aria-label={container.title} aria-pressed={activeContainer === container.id} onClick={() => openPerspective(container.id)}><Icon name={container.icon} size={20} /><span role="tooltip">{container.title}</span></button>)}</aside>
      {leftOpen && <><aside className="live-sidebar" id="case-sidebar"><PanelHeader title={containers.find((value) => value.id === activeContainer)?.title ?? activeContainer} />{sidebarViews.map((view) => { const ViewComponent = view.component; return <ViewComponent key={view.id} {...renderContext} containerId={activeContainer} />; })}</aside><Splitter label="Resize Case explorer" axis="x" value={leftWidth} min={170} max={320} set={setLeftWidth} /></>}
      <section className="live-center"><div className="surface-tabs" role="tablist">{surfaceState.inputs.map((input) => { const descriptor = registry.surfaceRenderer(input.surfaceType); return <button key={input.id} className={`surface-role-${descriptor?.role ?? "content"} surface-posture-${input.posture ?? "ready"}`} role="tab" aria-selected={surfaceState.activeId === input.id} title={`${input.title}${input.pinned ? "" : " · Preview"}${input.posture === "read-only" ? " · Read only" : ""}`} onClick={() => { surfaces.activate(input.id); if (input.viewId) setActiveContainer(input.viewId); }} onDoubleClick={() => descriptor?.capabilities.includes("pinnable") && surfaces.pin(input.id)}><Icon name={input.icon} size={14} /><span className="surface-tab-title">{input.title}</span>{input.dirty ? <i className="dirty-mark" aria-label="Unsaved changes">●</i> : !input.pinned ? <i className="preview-mark">Preview</i> : null}<span role="button" aria-label={`Close ${input.title}`} onClick={(event) => { event.stopPropagation(); surfaces.close(input.id); }}>×</span></button>; })}</div><main className="live-surface">{activeInput && SurfaceComponent ? <Suspense fallback={<div className="surface-loading">Loading {activeInput.title}…</div>}><SurfaceComponent {...renderContext} input={activeInput} /></Suspense> : <div className="empty-surface">No Surface renderer is registered for this input.</div>}</main>
        {bottomOpen && <Splitter label="Resize bottom panel" axis="y" reverse value={bottomHeight} min={190} max={bottomLimit} set={(value) => { setBottomMaximized(false); restoredBottomHeight.current = value; setBottomHeight(value); }} />}
        {bottomOpen && <section className="live-bottom" id="case-tools" style={{ height: bottomHeight }} aria-label="Bottom tools"><header>{panelViews.map((view) => <button key={view.id} aria-pressed={panel === view.id} onClick={() => setPanel(view.id)}>{view.title}</button>)}<span /><div className="panel-contribution-toolbar" ref={setPanelToolbarTarget} /><IconButton aria-label={bottomMaximized ? "Restore bottom panel" : "Maximize bottom panel"} onClick={() => { if (bottomMaximized) { setBottomHeight(restoredBottomHeight.current); setBottomMaximized(false); } else { restoredBottomHeight.current = bottomHeight; setBottomHeight(bottomLimit); setBottomMaximized(true); } }}><Icon name={bottomMaximized ? "restore" : "maximize"} size={14} /></IconButton><IconButton aria-label="Close bottom panel" onClick={() => setBottomOpen(false)}><Icon name="close" size={14} /></IconButton></header><div className="tool-content">{PanelComponent && <div className="tool-pane"><PanelComponent {...renderContext} available={platform.host.capabilities.terminalAvailable} toolbarTarget={panelToolbarTarget} closePanel={() => setBottomOpen(false)} /></div>}</div></section>}
      </section>
      {rightOpen && <><Splitter label="Resize context panel" axis="x" reverse value={rightWidth} min={290} max={470} set={setRightWidth} /><aside className="live-context" id="case-conversation"><header><div className="segmented">{auxiliaryViews.map((view) => <button key={view.id} aria-pressed={auxiliary === view.id} onClick={() => setAuxiliary(view.id)}>{view.title}</button>)}</div><IconButton aria-label="Close context panel" onClick={() => setRightOpen(false)}><Icon name="right" /></IconButton></header>{AuxiliaryComponent && <AuxiliaryComponent {...renderContext} />}</aside></>}
    </div>
    <footer className="kernel-status" aria-label="Workbench status">
      <div><span className="case-status" data-status={workspace.case.case_status}>{workspace.case.case_status}</span><span>Generation {workspace.case.generation}</span><span>{activeInput?.title ?? activeContainer}</span></div>
      <div>{workspace.presentation.dataKind === "fixture" && <span>Fixture data</span>}<span>{workspace.case.participant_ref}</span></div>
    </footer>
    {searchMode === "commands" && <WorkbenchSearch title="Command Palette" placeholder="Type a command" items={commandItems()} onClose={() => setSearchMode(undefined)} />}
    {searchMode === "open" && <WorkbenchSearch title="Quick Open" placeholder="Search open Surfaces and exposed Case material" items={quickItems()} onClose={() => setSearchMode(undefined)} />}
    {searchMode === "surface" && <WorkbenchSearch title={`Find in ${activeInput?.title ?? "Surface"}`} placeholder="Search current Surface" resolveItems={surfaceItems} onClose={() => setSearchMode(undefined)} />}
    {searchMode === "case" && <WorkbenchSearch title="Search Current Case" placeholder={searchCase ? "Search qualified Case presentation" : "Case search unavailable"} resolveItems={caseItems} onClose={() => setSearchMode(undefined)} />}
  </div>;
}
