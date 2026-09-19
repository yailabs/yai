import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import { Icon } from "../../components/Icon";
import { Badge, IconButton, PanelHeader } from "../../components/primitives";
import { DesktopWindowControls } from "../../app/DesktopWindowControls";
import { dispatchTerminalCommand } from "../../terminal/TerminalPanel";
import type { PlatformServices } from "../../platform/services";
import { DisposableStore, toDisposable } from "../../platform/lifecycle";
import { when } from "../../platform/context";
import type { WorkbenchRegistry } from "./registry";
import { EditorGroupService } from "../editor/model";
import type { NavigationLocation } from "../../platform/navigation";
import { ApplicationMenuBar } from "./ApplicationMenuBar";
import { Splitter } from "./Splitter";

export interface WorkbenchKernelProps {
  workspace: CasePresentation;
  stream: "connecting" | "live" | "reconnecting" | "unavailable" | "fixture";
  platform: PlatformServices;
  registry: WorkbenchRegistry;
  refresh(): void;
  openCaseSwitcher(): void;
}

export function WorkbenchKernel({ workspace, stream, platform, registry, refresh, openCaseSwitcher }: WorkbenchKernelProps) {
  const containers = registry.viewContainers();
  const [activeContainer, setActiveContainer] = useState(containers[0]?.id ?? "Overview");
  const [selection, setSelection] = useState(workspace.case.case_ref);
  const [auxiliary, setAuxiliary] = useState("Inspector");
  const [panel, setPanel] = useState("Terminal");
  const [leftOpen, setLeftOpen] = useState(true); const [rightOpen, setRightOpen] = useState(true); const [bottomOpen, setBottomOpen] = useState(true);
  const [leftWidth, setLeftWidth] = useState(204); const [rightWidth, setRightWidth] = useState(320);
  const [bottomHeight, setBottomHeight] = useState(() => Math.max(260, Math.floor(window.innerHeight * .36)));
  const [bottomLimit, setBottomLimit] = useState(() => Math.max(340, Math.floor(window.innerHeight * .72)));
  const [bottomMaximized, setBottomMaximized] = useState(false);
  const restoredBottomHeight = useRef(bottomHeight);
  const editors = useMemo(() => new EditorGroupService(), []);
  const [editorState, setEditorState] = useState(editors.snapshot());
  const applyingHistory = useRef(false);

  useEffect(() => editors.subscribe(() => setEditorState(editors.snapshot())).dispose, [editors]);
  useEffect(() => () => editors.dispose(), [editors]);
  useEffect(() => {
    editors.open({ id: "perspective:Overview", type: "perspective", title: "Overview", perspective: "Overview", pinned: true });
    platform.navigation.reset({ caseRef: workspace.case.case_ref, view: "Overview", editorId: "perspective:Overview", selection: workspace.case.case_ref, auxiliary: "Inspector" });
  }, [editors, platform.navigation, workspace.case.case_ref]);

  const record = useCallback((view: string, editorId: string, nextSelection = selection, nextAuxiliary = auxiliary) => {
    if (applyingHistory.current) return;
    platform.navigation.push({ caseRef: workspace.case.case_ref, view, editorId, selection: nextSelection, auxiliary: nextAuxiliary });
  }, [auxiliary, platform.navigation, selection, workspace.case.case_ref]);
  const openPerspective = useCallback((id: string, shouldRecord = true) => {
    const container = containers.find((value) => value.id === id);
    if (!container) return;
    setActiveContainer(id);
    const input = { id: `perspective:${id}`, type: "perspective" as const, title: container.title, perspective: id, pinned: true };
    editors.open(input);
    if (shouldRecord) record(id, input.id);
  }, [containers, editors, record]);
  const openMaterial = useCallback((resourceRef: string, title: string, pinned = false, shouldRecord = true) => {
    const id = pinned ? `material:${resourceRef}` : "preview:material";
    editors.open({ id, type: "material", title, resourceRef, pinned });
    setSelection(resourceRef); setAuxiliary("Inspector");
    if (shouldRecord) record(activeContainer, id, resourceRef, "Inspector");
  }, [activeContainer, editors, record]);
  const openSettings = useCallback((shouldRecord = true) => {
    const input = { id: "settings", type: "settings" as const, title: "Settings", pinned: true };
    editors.open(input);
    if (shouldRecord) record("Settings", input.id, "settings:general");
  }, [editors, record]);
  const inspect = useCallback((id: string) => {
    setSelection(id); setAuxiliary("Inspector");
    record(activeContainer, editorState.activeId ?? `perspective:${activeContainer}`, id, "Inspector");
  }, [activeContainer, editorState.activeId, record]);
  const actions = useMemo(() => ({ inspect, openMaterial, openPerspective, openSettings }), [inspect, openMaterial, openPerspective, openSettings]);

  const applyLocation = useCallback((location: NavigationLocation | undefined) => {
    if (!location) return;
    applyingHistory.current = true;
    setSelection(location.selection ?? workspace.case.case_ref);
    setAuxiliary(location.auxiliary ?? "Inspector");
    if (location.editorId === "settings") openSettings(false);
    else if (location.editorId.startsWith("material:") || location.editorId === "preview:material") openMaterial(location.selection ?? "", location.selection ?? "Material", location.editorId.startsWith("material:"), false);
    else openPerspective(location.view, false);
    applyingHistory.current = false;
  }, [openMaterial, openPerspective, openSettings, platform.navigation, workspace.case.case_ref]);

  useEffect(() => {
    const registrations = new DisposableStore();
    const command = (id: string, title: string, handler: () => void, predicate?: Parameters<typeof platform.commands.registerCommand>[0]["when"]) => registrations.add(platform.commands.registerCommand({ id, title, handler, when: predicate }));
    command("studio.file.openCase", "Open Case…", openCaseSwitcher);
    command("studio.file.settings", "Settings…", () => openSettings());
    command("studio.window.close", "Close Window", platform.host.closeWindow);
    command("studio.edit.cut", "Cut", () => document.execCommand("cut")); command("studio.edit.copy", "Copy", () => document.execCommand("copy")); command("studio.edit.paste", "Paste", () => document.execCommand("paste"));
    command("studio.view.toggleExplorer", "Explorer", () => setLeftOpen((value) => !value));
    command("studio.view.toggleContext", "Context Panel", () => setRightOpen((value) => !value));
    command("studio.view.toggleBottomPanel", "Bottom Panel", () => setBottomOpen((value) => !value));
    command("studio.view.resetLayout", "Reset Layout", () => { setLeftOpen(true); setRightOpen(true); setBottomOpen(true); setLeftWidth(204); setRightWidth(320); const height = Math.max(260, Math.floor(window.innerHeight * .36)); restoredBottomHeight.current = height; setBottomHeight(height); setBottomMaximized(false); });
    command("studio.go.back", "Back", () => applyLocation(platform.navigation.back()), () => platform.navigation.canBack());
    command("studio.go.forward", "Forward", () => applyLocation(platform.navigation.forward()), () => platform.navigation.canForward());
    command("studio.go.previousTab", "Previous Tab", () => editors.move(-1)); command("studio.go.nextTab", "Next Tab", () => editors.move(1));
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
    menu("View", "1", 0, "studio.view.toggleExplorer", when.truthy("sidebar.visible")); menu("View", "1", 1, "studio.view.toggleContext", when.truthy("auxiliary.visible")); menu("View", "1", 2, "studio.view.toggleBottomPanel", when.truthy("panel.visible")); menu("View", "2", 0, "studio.view.resetLayout");
    menu("Go", "1", 0, "studio.go.back"); menu("Go", "1", 1, "studio.go.forward"); menu("Go", "2", 0, "studio.go.previousTab"); menu("Go", "2", 1, "studio.go.nextTab");
    containers.forEach((container, order) => menu("Case", "1", order, `studio.case.${container.id.toLowerCase()}`, when.equals("view.active", container.id))); menu("Case", "2", 0, "studio.case.refresh"); menu("Case", "3", 0, "studio.file.openCase");
    ["new", "kill", "clear", "focus"].forEach((name, order) => menu("Terminal", "1", order, `studio.terminal.${name}`)); menu("Terminal", "2", 0, "studio.view.toggleBottomPanel"); menu("Help", "1", 0, "studio.help.about");
    const binding = (id: string, commandId: string, key: string, allowInTerminal = false) => registrations.add(platform.keybindings.registerKeybinding({ id, command: commandId, key, allowInTerminal }));
    binding("toggle-panel", "studio.view.toggleBottomPanel", "Mod+j", true); binding("terminal-focus", "studio.terminal.focus", "Mod+`", true); binding("terminal-new", "studio.terminal.new", "Mod+Shift+`", true); binding("open-case", "studio.file.openCase", "Mod+o"); binding("toggle-sidebar", "studio.view.toggleExplorer", "Mod+b"); binding("toggle-auxiliary", "studio.view.toggleContext", "Mod+Shift+b"); binding("go-back", "studio.go.back", "Alt+ArrowLeft"); binding("go-forward", "studio.go.forward", "Alt+ArrowRight");
    containers.forEach((container, order) => binding(`perspective-${container.id}`, `studio.case.${container.id.toLowerCase()}`, `Mod+${order + 1}`));
    const focus = (event: FocusEvent) => {
      const candidate = event.type === "focusout" ? event.relatedTarget : event.target;
      platform.context.update("terminal.focused", Boolean((candidate as Element | null)?.closest?.(".xterm")));
    };
    document.addEventListener("focusin", focus); document.addEventListener("focusout", focus);
    registrations.add(toDisposable(() => { document.removeEventListener("focusin", focus); document.removeEventListener("focusout", focus); }));
    registrations.add(platform.keybindings.attach());
    return () => registrations.dispose();
  }, [applyLocation, containers, editors, openCaseSwitcher, openPerspective, openSettings, platform, refresh]);

  useEffect(() => {
    platform.context.update("studio.data.live", workspace.presentation.dataKind === "live");
    platform.context.update("studio.data.fixture", workspace.presentation.dataKind === "fixture");
    platform.context.update("case.attached", true); platform.context.update("view.active", activeContainer);
    platform.context.update("panel.active", panel); platform.context.update("sidebar.visible", leftOpen);
    platform.context.update("auxiliary.visible", rightOpen); platform.context.update("panel.visible", bottomOpen);
    platform.context.update("selection.kind", selection === workspace.case.case_ref ? "case" : "object");
    platform.context.update("case.hasWorkflow", workspace.work.status !== "empty"); platform.context.update("case.hasProvider", workspace.compute.targets.length > 0);
  }, [activeContainer, bottomOpen, leftOpen, panel, platform.context, rightOpen, selection, workspace]);
  useEffect(() => { const resize = () => { const limit = Math.max(340, Math.floor(window.innerHeight * .72)); setBottomLimit(limit); setBottomHeight((value) => Math.max(190, Math.min(limit, value))); }; window.addEventListener("resize", resize); return () => window.removeEventListener("resize", resize); }, []);

  const activeInput = editorState.inputs.find((input) => input.id === editorState.activeId);
  const editorContribution = activeInput ? registry.editor(activeInput.type) : undefined;
  const sidebarViews = registry.viewsFor(activeContainer);
  const panelViews = registry.panelViews(); const activePanel = panelViews.find((view) => view.id === panel) ?? panelViews[0];
  const auxiliaryViews = registry.auxiliaryViews(); const activeAuxiliary = auxiliaryViews.find((view) => view.id === auxiliary) ?? auxiliaryViews[0];
  const inspector = auxiliary === "Inspector" ? registry.inspector("default") : undefined;
  const renderContext = { workspace, selection, actions };
  const EditorComponent = editorContribution?.component;
  const PanelComponent = activePanel?.component;
  const AuxiliaryComponent = inspector?.component ?? activeAuxiliary?.component;
  return <div className="live-case-shell workbench-kernel" data-case-source={workspace.presentation.dataKind} data-host={platform.host.capabilities.kind} style={{ "--left-width": `${leftWidth}px`, "--right-width": `${rightWidth}px`, "--bottom-height": `${bottomHeight}px` } as React.CSSProperties}>
    <nav className="case-titlebar" aria-label="Application and workspace controls" data-tauri-drag-region><ApplicationMenuBar platform={platform} /><div className="titlebar-center" data-tauri-drag-region><IconButton aria-label="Back" disabled={!platform.navigation.canBack()} onClick={() => applyLocation(platform.navigation.back())}><Icon name="back" /></IconButton><IconButton aria-label="Forward" disabled={!platform.navigation.canForward()} onClick={() => applyLocation(platform.navigation.forward())}><Icon name="forward" /></IconButton><button className="titlebar-case" onClick={openCaseSwitcher}><Icon name="case" size={14} /><span>{workspace.case.case_ref}</span><small>Generation {workspace.case.generation}</small><Icon name="chevron" size={12} /></button></div><div className="titlebar-layout">{workspace.presentation.dataKind === "fixture" && <Badge tone="warning">Fixture data</Badge>}{stream !== "live" && stream !== "fixture" && <Badge tone={stream === "unavailable" ? "error" : "warning"}>{stream}</Badge>}<IconButton aria-label="Refresh Case" onClick={refresh}><Icon name="refresh" /></IconButton><IconButton aria-label="Toggle Case sidebar" onClick={() => setLeftOpen((value) => !value)}><Icon name="left" /></IconButton><IconButton aria-label="Toggle bottom panel" onClick={() => setBottomOpen((value) => !value)}><Icon name="bottom" /></IconButton><IconButton aria-label="Toggle context panel" onClick={() => setRightOpen((value) => !value)}><Icon name="right" /></IconButton><DesktopWindowControls /></div></nav>
    <div className="live-workbench">
      <aside className="live-rail" aria-label="Case perspectives">{containers.map((container) => <button key={container.id} aria-label={container.title} aria-pressed={activeContainer === container.id} onClick={() => openPerspective(container.id)}><Icon name={container.icon} size={20} /><span role="tooltip">{container.title}</span></button>)}</aside>
      {leftOpen && <><aside className="live-sidebar" id="case-sidebar"><PanelHeader title={containers.find((value) => value.id === activeContainer)?.title ?? activeContainer} />{sidebarViews.map((view) => { const ViewComponent = view.component; return <ViewComponent key={view.id} {...renderContext} containerId={activeContainer} />; })}</aside><Splitter label="Resize Case explorer" axis="x" value={leftWidth} min={170} max={320} set={setLeftWidth} /></>}
      <section className="live-center"><div className="live-tabs" role="tablist">{editorState.inputs.map((input) => <button key={input.id} role="tab" aria-selected={editorState.activeId === input.id} onClick={() => { editors.activate(input.id); if (input.perspective) setActiveContainer(input.perspective); }} onDoubleClick={() => editors.pin(input.id)}><Icon name={input.type === "material" ? "sources" : input.type === "settings" ? "overview" : containers.find((item) => item.id === input.perspective)?.icon ?? "overview"} size={14} />{input.title}{!input.pinned && <i>Preview</i>}<span role="button" aria-label={`Close ${input.title}`} onClick={(event) => { event.stopPropagation(); editors.close(input.id); }}>×</span></button>)}</div><main className="live-surface">{activeInput && EditorComponent ? <EditorComponent {...renderContext} input={activeInput} /> : <div className="empty-surface">No editor input.</div>}</main>
        {bottomOpen && <Splitter label="Resize bottom panel" axis="y" reverse value={bottomHeight} min={190} max={bottomLimit} set={(value) => { setBottomMaximized(false); restoredBottomHeight.current = value; setBottomHeight(value); }} />}
        {bottomOpen && <section className="live-bottom" id="case-tools" style={{ height: bottomHeight }} aria-label="Bottom tools"><header>{panelViews.map((view) => <button key={view.id} aria-pressed={panel === view.id} onClick={() => setPanel(view.id)}>{view.title}</button>)}<span /><IconButton aria-label={bottomMaximized ? "Restore bottom panel" : "Maximize bottom panel"} onClick={() => { if (bottomMaximized) { setBottomHeight(restoredBottomHeight.current); setBottomMaximized(false); } else { restoredBottomHeight.current = bottomHeight; setBottomHeight(bottomLimit); setBottomMaximized(true); } }}><Icon name={bottomMaximized ? "restore" : "maximize"} size={14} /></IconButton><IconButton aria-label="Close bottom panel" onClick={() => setBottomOpen(false)}><Icon name="close" size={14} /></IconButton></header><div className="tool-content">{PanelComponent && <div className="tool-pane"><PanelComponent {...renderContext} available={platform.host.capabilities.terminalAvailable} /></div>}</div></section>}
      </section>
      {rightOpen && <><Splitter label="Resize context panel" axis="x" reverse value={rightWidth} min={290} max={470} set={setRightWidth} /><aside className="live-context" id="case-conversation"><header><div className="segmented">{auxiliaryViews.map((view) => <button key={view.id} aria-pressed={auxiliary === view.id} onClick={() => setAuxiliary(view.id)}>{view.title}</button>)}</div><IconButton aria-label="Close context panel" onClick={() => setRightOpen(false)}><Icon name="right" /></IconButton></header>{AuxiliaryComponent && <AuxiliaryComponent {...renderContext} />}</aside></>}
    </div>
    <footer className="kernel-status"><span>{workspace.presentation.dataKind === "fixture" ? "Fixture Case data" : "Live Case data"}</span><span>{platform.host.capabilities.nativeDesktop ? "Native host" : "Web host"}</span><span>{platform.host.capabilities.terminalAvailable ? "Terminal available" : "Terminal unavailable"}</span><span>{workspace.presentation.backendPosture}</span></footer>
  </div>;
}
