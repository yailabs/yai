import { useCallback, useEffect, useRef, useState } from "react";
import { LiveClient } from "../clients/live";
import type { CaseAttachment, CaseUpdate, LiveCaseRow, LiveEdge, LiveNode, LiveWorkspace, OperationResult } from "../clients/live";
import { Badge, Button, EmptyState, IconButton, PanelHeader, SearchInput } from "../components/primitives";
import { Icon } from "../components/Icon";
import { GraphViewport } from "./GraphViewport";

const client = new LiveClient();
const perspectives = ["Overview", "Environment", "Knowledge", "Memory", "Authority", "Work", "Compute"] as const;
type Perspective = typeof perspectives[number];
type SurfaceKind = Perspective | "Settings" | "Material";
type ContextMode = "Conversation" | "Inspector" | "Activity";
type BottomMode = "Terminal" | "Output" | "Executions" | "Evidence" | "Problems";
interface Tab { id: string; label: string; perspective: SurfaceKind; pinned: boolean; materialRef?: string }
interface NavigationEntry { surface: SurfaceKind; selected: string; context: ContextMode }

export function LiveStudio() {
  const [casesResult, setCasesResult] = useState<OperationResult<{ cases: LiveCaseRow[]; authority: string }>>();
  const [attachment, setAttachment] = useState<CaseAttachment>();
  const [workspaceResult, setWorkspaceResult] = useState<OperationResult<LiveWorkspace>>();
  const [switcher, setSwitcher] = useState(false);
  const [menu, setMenu] = useState<"File" | "View" | null>(null);
  const [stream, setStream] = useState<"connecting" | "live" | "reconnecting" | "unavailable">("connecting");
  const [cursor, setCursor] = useState<string>();
  const loadCases = useCallback(async () => setCasesResult(await client.listCases()), []);
  const loadCase = useCallback(async (case_ref: string) => {
    const opened = await client.openCase(case_ref);
    if (opened.result_state !== "success" || !opened.data) {
      setWorkspaceResult({ operation_ref: opened.operation_ref, result_state: opened.result_state, correlation_ref: opened.correlation_ref, error: opened.error });
      return;
    }
    setAttachment(opened.data);
    setWorkspaceResult(undefined);
    setWorkspaceResult(await client.caseSummary(case_ref));
    window.history.replaceState({ case_ref }, "", window.location.pathname);
  }, []);
  const refresh = useCallback(async (expected_generation?: number) => {
    if (!attachment) return;
    const next = await client.caseSummary(attachment.case_ref, expected_generation);
    if (next.result_state === "stale") {
      setStream("reconnecting");
      const resynced = await client.caseSummary(attachment.case_ref);
      setWorkspaceResult(resynced);
      setStream(resynced.result_state === "success" ? "live" : "unavailable");
      return;
    }
    setWorkspaceResult(next);
  }, [attachment]);

  useEffect(() => { void loadCases(); }, [loadCases]);
  useEffect(() => {
    let stop: () => void = () => undefined;
    void client.subscribe(cursor).then((result) => setStream(result.result_state === "success" ? "live" : "unavailable"));
    void client.listen((update: CaseUpdate) => {
      setCursor(update.cursor);
      setStream("live");
      if (update.case_ref === attachment?.case_ref && update.generation !== workspaceResult?.data?.case.generation) void refresh(update.generation);
      void loadCases();
    }).then((unlisten) => { stop = unlisten; });
    return () => stop();
  }, [attachment?.case_ref, workspaceResult?.data?.case.generation, refresh, loadCases]);
  useEffect(() => {
    if (!attachment) return;
    let active = true;
    const heartbeat = async () => {
      const result = await client.heartbeat(attachment.case_ref);
      if (!active) return;
      if (result.result_state !== "success" || !result.data) { setStream("unavailable"); return; }
      setCursor(result.data.cursor); setStream("live");
      if (result.data.generation !== workspaceResult?.data?.case.generation) void refresh(result.data.generation);
    };
    const timer = window.setInterval(() => void heartbeat(), 1000);
    void heartbeat();
    return () => { active = false; window.clearInterval(timer); };
  }, [attachment?.case_ref, workspaceResult?.data?.case.generation, refresh]);
  useEffect(() => {
    const guard = () => { if (attachment) window.history.replaceState({ case_ref: attachment.case_ref }, "", window.location.pathname); };
    window.addEventListener("popstate", guard);
    return () => window.removeEventListener("popstate", guard);
  }, [attachment]);
  useEffect(() => { document.title = attachment ? `${attachment.case_ref} — YAI Studio` : "YAI Studio"; }, [attachment]);

  const cases = casesResult?.data?.cases ?? [];
  const workspace = workspaceResult?.data;
  return <div className={`live-studio ${attachment ? "case-attached" : ""}`}>
    {!attachment && <nav className="native-menu" aria-label="Application menu">
      <strong>YAI</strong>
      <button onClick={() => setMenu(menu === "File" ? null : "File")}>File</button>
      <button onClick={() => setMenu(menu === "View" ? null : "View")}>View</button>
      {menu === "File" && <div className="native-menu-popover"><button onClick={() => { setSwitcher(true); setMenu(null); }}>Open Case…</button><button disabled>New Case <kbd>Unavailable</kbd></button><button onClick={() => window.close()}>Close Window</button></div>}
      {menu === "View" && <div className="native-menu-popover view"><button onClick={() => setMenu(null)}>Reset local view</button><button disabled>Command palette <kbd>Later</kbd></button></div>}
    </nav>}
    {!attachment && <LiveStartCenter result={casesResult} cases={cases} open={loadCase} retry={loadCases} />}
    {attachment && workspace && <LiveWorkbench workspace={workspace} stream={stream} refresh={() => void refresh()} openSwitcher={() => setSwitcher(true)} />}
    {attachment && workspaceResult && workspaceResult.result_state !== "success" && <HostFailure result={workspaceResult} retry={() => void refresh()} />}
    {switcher && <CaseSwitcher cases={cases} close={() => setSwitcher(false)} open={(id) => { setSwitcher(false); void loadCase(id); }} />}
  </div>;
}

function LiveStartCenter({ result, cases, open, retry }: { result?: OperationResult<unknown>; cases: LiveCaseRow[]; open: (id: string) => void; retry: () => void }) {
  const [query, setQuery] = useState("");
  const visible = cases.filter((item) => item.case_ref.toLowerCase().includes(query.toLowerCase()));
  return <main className="live-start">
    <section className="live-start-intro"><span>Local Case Workbench</span><h1>Open a Case.</h1><p>Studio attaches to durable Case continuity owned by YAI on this machine.</p><SearchInput autoFocus aria-label="Search local Cases" placeholder="Search real local Cases" value={query} onChange={(event) => setQuery(event.target.value)} /></section>
    {result?.result_state && result.result_state !== "success" ? <HostFailure result={result} retry={retry} /> : <section className="live-case-list" aria-label="Real local Cases"><PanelHeader title="Local Cases" detail={`${visible.length} visible to the authenticated principal`} />{visible.map((item) => <button className="live-case-row" key={item.case_ref} onClick={() => open(item.case_ref)}><Icon name="case" size={18} /><span><strong>{item.display_name}</strong><small>{item.case_status} · generation {item.generation}</small></span><span className="case-facts">{item.source_count} sources · {item.participant_count} participants{item.pending_review_count ? ` · ${item.pending_review_count} reviews` : ""}</span><Icon name="chevron" size={14} /></button>)}{!visible.length && <EmptyState title="No local Cases" body="YAI returned an empty authorized Case list. Studio has not substituted sample data." />}</section>}
    <footer><Badge tone="success">Real local data</Badge><span>Fixtures are available only in explicit development mode.</span></footer>
  </main>;
}

function HostFailure({ result, retry }: { result: OperationResult<unknown>; retry: () => void }) {
  return <main className="host-failure"><Icon name="warning" size={28} /><h1>Local YAI unavailable</h1><p>{result.error?.safe_message ?? "YAI did not return this projection."}</p><code>{result.result_state} · {result.error?.code}</code><Button onClick={retry}>Retry</Button><small>No fixture data has been loaded.</small></main>;
}

function CaseSwitcher({ cases, close, open }: { cases: LiveCaseRow[]; close: () => void; open: (id: string) => void }) {
  const [query, setQuery] = useState("");
  return <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) close(); }}><section className="case-switcher" role="dialog" aria-modal="true" aria-label="Open Case"><PanelHeader title="Open Case" detail="Real local YAI" actions={<IconButton aria-label="Close" onClick={close}><Icon name="close" /></IconButton>} /><SearchInput autoFocus placeholder="Case ID" value={query} onChange={(event) => setQuery(event.target.value)} />{cases.filter((item) => item.case_ref.includes(query)).map((item) => <button className="ui-list-row" key={item.case_ref} onClick={() => open(item.case_ref)}><Icon name="case" /><span><strong>{item.case_ref}</strong><small>Generation {item.generation}</small></span></button>)}</section></div>;
}

function LiveWorkbench({ workspace, stream, refresh, openSwitcher }: { workspace: LiveWorkspace; stream: "connecting" | "live" | "reconnecting" | "unavailable"; refresh: () => void; openSwitcher: () => void }) {
  const [perspective, setPerspective] = useState<Perspective>("Overview");
  const [surface, setSurface] = useState<SurfaceKind>("Overview");
  const [context, setContext] = useState<ContextMode>("Inspector");
  const [selected, setSelected] = useState(workspace.case.case_ref);
  const [bottom, setBottom] = useState<BottomMode>("Terminal");
  const [leftOpen, setLeftOpen] = useState(true); const [rightOpen, setRightOpen] = useState(true); const [bottomOpen, setBottomOpen] = useState(true);
  const [leftWidth, setLeftWidth] = useState(228); const [rightWidth, setRightWidth] = useState(330); const [bottomHeight, setBottomHeight] = useState(300);
  const [bottomMax, setBottomMax] = useState(() => Math.max(240, Math.min(520, Math.floor(window.innerHeight * .55))));
  const [tabs, setTabs] = useState<Tab[]>([{ id: "perspective:Overview", label: "Overview", perspective: "Overview", pinned: true }]);
  const [activeTab, setActiveTab] = useState("perspective:Overview");
  const [titleMenu, setTitleMenu] = useState<"File" | "View" | null>(null);
  const history = useRef<NavigationEntry[]>([{ surface: "Overview", selected: workspace.case.case_ref, context: "Inspector" }]); const historyIndex = useRef(0); const [, renderHistory] = useState(0);
  const recordNavigation = (entry: NavigationEntry) => {
    const current = history.current[historyIndex.current];
    if (current?.surface === entry.surface && current.selected === entry.selected && current.context === entry.context) return;
    history.current = [...history.current.slice(0, historyIndex.current + 1), entry];
    historyIndex.current++;
    renderHistory((value) => value + 1);
  };
  const navigate = (next: Perspective, record = true) => {
    setPerspective(next);
    setSurface(next);
    const id = `perspective:${next}`;
    setTabs((current) => current.some((tab) => tab.id === id) ? current : [...current, { id, label: next, perspective: next, pinned: true }]);
    setActiveTab(id);
    if (record) recordNavigation({ surface: next, selected, context });
  };
  const openSettings = (record = true) => {
    setSurface("Settings");
    setTabs((current) => current.some((tab) => tab.id === "settings") ? current : [...current, { id: "settings", label: "Settings", perspective: "Settings", pinned: true }]);
    setActiveTab("settings");
    if (record) recordNavigation({ surface: "Settings", selected: "settings:general", context });
  };
  const openMaterial = (materialRef: string, label: string, pinned = false, record = true) => {
    const id = pinned ? `material:${materialRef}` : "preview:material";
    setSurface("Material"); setSelected(materialRef); setContext("Inspector"); setActiveTab(id);
    setTabs((current) => {
      const retained = current.filter((tab) => tab.pinned || tab.id === id);
      const next = { id, label, perspective: "Material" as const, pinned, materialRef };
      return retained.some((tab) => tab.id === id) ? retained.map((tab) => tab.id === id ? next : tab) : [...retained, next];
    });
    if (record) recordNavigation({ surface: "Material", selected: materialRef, context: "Inspector" });
  };
  const pinMaterialTab = (tab: Tab) => {
    if (tab.perspective !== "Material" || !tab.materialRef || tab.pinned) return;
    const pinnedId = `material:${tab.materialRef}`;
    setTabs((items) => items.map((item) => item.id === tab.id ? { ...item, id: pinnedId, pinned: true } : item));
    if (activeTab === tab.id) setActiveTab(pinnedId);
  };
  const moveHistory = (delta: number) => {
    const next = historyIndex.current + delta;
    if (next < 0 || next >= history.current.length) return;
    historyIndex.current = next;
    const entry = history.current[next];
    setSelected(entry.selected); setContext(entry.context);
    if (entry.surface === "Settings") openSettings(false); else if (entry.surface === "Material") openMaterial(entry.selected, materialLabel(workspace, entry.selected), false, false); else navigate(entry.surface, false);
    renderHistory((value) => value + 1);
  };
  const inspect = (id: string) => { setSelected(id); setContext("Inspector"); recordNavigation({ surface, selected: id, context: "Inspector" }); };
  const closeTab = (id: string) => {
    setTabs((current) => current.filter((tab) => tab.id !== id));
    if (activeTab === id) {
      setActiveTab("perspective:Overview");
      navigate("Overview", false);
      setTabs((current) => current.some((tab) => tab.id === "perspective:Overview") ? current : [{ id: "perspective:Overview", label: "Overview", perspective: "Overview", pinned: true }, ...current]);
    }
  };
  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      if (event.key.toLowerCase() === "j") { event.preventDefault(); setBottomOpen((value) => !value); }
      if (event.key.toLowerCase() === "b" && event.shiftKey) { event.preventDefault(); setRightOpen((value) => !value); }
      else if (event.key.toLowerCase() === "b") { event.preventDefault(); setLeftOpen((value) => !value); }
      const index = Number(event.key) - 1;
      if (index >= 0 && index < perspectives.length) { event.preventDefault(); navigate(perspectives[index]); }
    };
    window.addEventListener("keydown", shortcut); return () => window.removeEventListener("keydown", shortcut);
  }, []);
  useEffect(() => {
    const clampBottom = () => {
      const nextMax = Math.max(240, Math.min(520, Math.floor(window.innerHeight * .55)));
      setBottomMax(nextMax);
      setBottomHeight((value) => Math.max(180, Math.min(nextMax, value)));
    };
    window.addEventListener("resize", clampBottom);
    return () => window.removeEventListener("resize", clampBottom);
  }, []);
  return <div className="live-case-shell" style={{ "--left-width": `${leftWidth}px`, "--right-width": `${rightWidth}px`, "--bottom-height": `${bottomHeight}px` } as React.CSSProperties}>
    <nav className="case-titlebar" aria-label="Application and workspace controls">
      <div className="titlebar-menu"><strong>YAI</strong><button onClick={() => setTitleMenu(titleMenu === "File" ? null : "File")}>File</button><button onClick={() => setTitleMenu(titleMenu === "View" ? null : "View")}>View</button>{titleMenu === "File" && <div className="native-menu-popover"><button onClick={() => { openSwitcher(); setTitleMenu(null); }}>Open Case…</button><button disabled>New Case <kbd>Unavailable</kbd></button><button onClick={() => { openSettings(); setTitleMenu(null); }}>Settings…</button><button onClick={() => window.close()}>Close Window</button></div>}{titleMenu === "View" && <div className="native-menu-popover view"><button onClick={() => { setLeftOpen((value) => !value); setTitleMenu(null); }}>Toggle Case sidebar <kbd>Ctrl B</kbd></button><button onClick={() => { setBottomOpen((value) => !value); setTitleMenu(null); }}>Toggle bottom tools <kbd>Ctrl J</kbd></button><button onClick={() => { setRightOpen((value) => !value); setTitleMenu(null); }}>Toggle Context Panel</button></div>}</div>
      <div className="titlebar-center"><IconButton aria-label="Back" disabled={historyIndex.current === 0} onClick={() => moveHistory(-1)}><Icon name="back" /></IconButton><IconButton aria-label="Forward" disabled={historyIndex.current === history.current.length - 1} onClick={() => moveHistory(1)}><Icon name="forward" /></IconButton><button className="titlebar-case" onClick={openSwitcher}><Icon name="case" size={14} /><span>{workspace.case.case_ref}</span><small>Generation {workspace.case.generation}</small><Icon name="chevron" size={12} /></button></div>
      <div className="titlebar-layout">{stream !== "live" && <Badge tone={stream === "unavailable" ? "error" : "warning"}>{stream}</Badge>}<IconButton aria-label="Refresh real Case" onClick={refresh}><Icon name="refresh" /></IconButton><IconButton aria-label="Toggle Case sidebar" onClick={() => setLeftOpen((value) => !value)}><Icon name="left" /></IconButton><IconButton aria-label="Toggle bottom panel" onClick={() => setBottomOpen((value) => !value)}><Icon name="bottom" /></IconButton><IconButton aria-label="Toggle context panel" onClick={() => setRightOpen((value) => !value)}><Icon name="right" /></IconButton></div>
    </nav>
    <div className="live-workbench">
    <aside className="live-rail" aria-label="Case perspectives">{perspectives.map((item) => <button key={item} title={item} aria-label={item} aria-pressed={perspective === item} onClick={() => navigate(item)}><Icon name={perspectiveIcon(item)} size={20} /><span>{item}</span></button>)}</aside>
    {leftOpen && <><aside className="live-sidebar"><PanelHeader title={perspective} detail={`Generation ${workspace.case.generation}`} /><CaseSidebar workspace={workspace} perspective={perspective} selected={selected} inspect={inspect} navigate={navigate} openMaterial={openMaterial} /></aside><Splitter label="Resize Case explorer" axis="x" value={leftWidth} min={200} max={360} set={setLeftWidth} /></>}
    <section className="live-center">
      <div className="live-tabs" role="tablist">{tabs.map((tab) => { const tabIcon = tab.perspective === "Material" ? "sources" : tab.perspective === "Settings" ? "overview" : perspectiveIcon(tab.perspective); return <button key={tab.id} role="tab" aria-selected={activeTab === tab.id} onClick={() => { setActiveTab(tab.id); if (tab.perspective === "Settings") openSettings(); else if (tab.perspective === "Material") { if (tab.materialRef) openMaterial(tab.materialRef, tab.label, tab.pinned); } else navigate(tab.perspective); }} onDoubleClick={() => pinMaterialTab(tab)}><Icon name={tabIcon} size={14} />{tab.label}{!tab.pinned && <i>Preview</i>}<span role="button" aria-label={`Close ${tab.label}`} onClick={(event) => { event.stopPropagation(); closeTab(tab.id); }}>×</span></button>; })}</div>
      <main className="live-surface"><Surface workspace={workspace} surface={surface} inspect={inspect} selected={selected} navigate={navigate} openMaterial={openMaterial} recordSettings={(id) => recordNavigation({ surface: "Settings", selected: id, context })} /></main>
      {bottomOpen && <><Splitter label="Resize bottom panel" axis="y" reverse value={bottomHeight} min={180} max={bottomMax} set={setBottomHeight} /><BottomPanel workspace={workspace} active={bottom} setActive={setBottom} close={() => setBottomOpen(false)} height={bottomHeight} /></>}
    </section>
    {rightOpen && <><Splitter label="Resize context panel" axis="x" reverse value={rightWidth} min={290} max={470} set={setRightWidth} /><ContextPanel workspace={workspace} mode={context} setMode={setContext} selected={selected} inspect={inspect} close={() => setRightOpen(false)} openMaterial={openMaterial} /></>}
    </div>
  </div>;
}

function Splitter({ label, axis, reverse = false, value, min, max, set }: { label: string; axis: "x" | "y"; reverse?: boolean; value: number; min: number; max: number; set: (value: number) => void }) {
  const clamp = (next: number) => set(Math.max(min, Math.min(max, next)));
  return <div className={`live-splitter ${axis}`} role="separator" aria-label={label} tabIndex={0} aria-orientation={axis === "x" ? "vertical" : "horizontal"} aria-valuemin={min} aria-valuemax={max} aria-valuenow={value} onKeyDown={(event) => { const decrease = axis === "x" ? "ArrowLeft" : "ArrowDown"; const increase = axis === "x" ? "ArrowRight" : "ArrowUp"; if (event.key === decrease || event.key === increase) { event.preventDefault(); clamp(value + (event.key === increase ? 16 : -16) * (axis === "x" && reverse ? -1 : 1)); } else if (event.key === "Home") { event.preventDefault(); clamp(min); } else if (event.key === "End") { event.preventDefault(); clamp(max); } }} onPointerDown={(event) => { event.currentTarget.setPointerCapture?.(event.pointerId); const start = axis === "x" ? event.clientX : event.clientY; const initial = value; const move = (next: PointerEvent) => { const point = axis === "x" ? next.clientX : next.clientY; const delta = (point - start) * (reverse ? -1 : 1); clamp(initial + delta); }; const up = () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); }; window.addEventListener("pointermove", move); window.addEventListener("pointerup", up); }} />;
}

function CaseSidebar({ workspace, perspective, selected, inspect, navigate, openMaterial }: { workspace: LiveWorkspace; perspective: Perspective; selected: string; inspect: (id: string) => void; navigate: (p: Perspective) => void; openMaterial: (id: string, label: string, pinned?: boolean) => void }) {
  const rows = perspectiveRows(workspace, perspective);
  return <div className="live-sidebar-content"><button className={`case-identity ${selected === workspace.case.case_ref ? "selected" : ""}`} onClick={() => inspect(workspace.case.case_ref)}><small>{workspace.case.case_status} · generation {workspace.case.generation}</small><h1>{workspace.case.display_name}</h1><p>{workspace.case.tenant_ref ?? "Tenant unavailable"}</p></button>{perspective === "Overview" && <><section className="sidebar-group case-outline"><h2>Case structure <span>Real</span></h2>{overviewDimensions(workspace).map((item) => <button key={item.perspective} onClick={() => navigate(item.perspective)}><Icon name={perspectiveIcon(item.perspective)} /><span><strong>{item.perspective}</strong><small>{item.detail}</small></span></button>)}</section><section className="sidebar-group"><h2>Participants <span>{workspace.overview.participants.length}</span></h2>{workspace.overview.participants.map((person) => <button key={person.id} className={selected === person.id ? "selected" : ""} onClick={() => inspect(person.id)}><Icon name="people" /><span><strong>{person.id}</strong><small>{person.roles.join(", ") || "No role exposed"}</small></span>{person.is_current && <Badge tone="info">Current</Badge>}</button>)}</section></>}{rows.map((group) => <section className="sidebar-group" key={group.label}><h2>{group.label}<span>{group.items.length}</span></h2>{group.items.map((item) => <button key={item.id} className={selected === item.id ? "selected" : ""} onClick={() => ("material" in item && item.material) ? openMaterial(item.id, item.label) : inspect(item.id)} onDoubleClick={() => { if ("material" in item && item.material) openMaterial(item.id, item.label, true); }}><Icon name={item.icon} /><span><strong>{item.label}</strong><small>{item.detail}</small></span></button>)}{!group.items.length && <p className="sidebar-empty">No qualified items.</p>}</section>)}</div>;
}

function Surface({ workspace, surface, inspect, selected, navigate, openMaterial, recordSettings }: { workspace: LiveWorkspace; surface: SurfaceKind; inspect: (id: string) => void; selected: string; navigate: (perspective: Perspective) => void; openMaterial: (id: string, label: string, pinned?: boolean) => void; recordSettings: (id: string) => void }) {
  if (surface === "Settings") return <Settings workspace={workspace} recordNavigation={recordSettings} />;
  if (surface === "Material") return <MaterialSurface workspace={workspace} materialRef={selected} />;
  if (surface === "Overview") return <Overview workspace={workspace} inspect={inspect} navigate={navigate} />;
  if (surface === "Environment") return <Environment workspace={workspace} inspect={inspect} openMaterial={openMaterial} />;
  if (surface === "Knowledge") return <Knowledge workspace={workspace} inspect={inspect} />;
  if (surface === "Memory") return <Memory workspace={workspace} inspect={inspect} />;
  if (surface === "Authority") return <Authority workspace={workspace} inspect={inspect} />;
  if (surface === "Work") return <Work workspace={workspace} inspect={inspect} />;
  return <Compute workspace={workspace} inspect={inspect} selected={selected} />;
}

function MaterialSurface({ workspace, materialRef }: { workspace: LiveWorkspace; materialRef: string }) {
  const file = workspace.environment.files.find((item) => item.id === materialRef);
  const source = workspace.environment.sources.find((item) => item.id === (file?.source_ref ?? materialRef));
  const qualified = workspace.knowledge.sources.find((item) => item.id === materialRef || item.source_ref === (file?.source_ref ?? source?.id));
  const sourceKeys = new Set([qualified?.id, qualified?.source_ref, file?.source_ref, source?.id].filter(Boolean));
  const units = workspace.knowledge.units.filter((unit) => sourceKeys.has(unit.source_ref) && (!file || !qualified?.path || qualified.path === file.path));
  const title = file?.path ?? qualified?.path ?? source?.label ?? compact(materialRef);
  return <article className="material-surface">
    <header><span>{source?.label ?? qualified?.label ?? "Case material"}</span><h1>{title}</h1><p>{file ? `${file.bytes.toLocaleString()} bytes · ${file.revision_ref}` : source?.perimeter ?? qualified?.detail ?? "Qualified material"}</p></header>
    <dl className="material-metadata"><div><dt>Case</dt><dd>{workspace.case.case_ref}</dd></div><div><dt>Source</dt><dd>{file?.source_ref ?? source?.id ?? qualified?.source_ref ?? "Unavailable"}</dd></div><div><dt>Revision</dt><dd>{file?.revision_ref ?? source?.revision_ref ?? qualified?.revision_ref ?? "Unavailable"}</dd></div><div><dt>Digest</dt><dd>{file?.digest ?? qualified?.digest ?? "Unavailable"}</dd></div></dl>
    {units.length ? <section className="material-document">{units.map((unit) => <div key={unit.id} className={unit.kind === "heading" ? "heading" : "body"}>{unit.text}</div>)}</section> : <EmptyState title="Content unavailable" body="YAI exposes this material identity and revision, but no qualified readable content projection for the current participant. Studio has not read the filesystem directly." />}
  </article>;
}

function Settings({ workspace, recordNavigation }: { workspace: LiveWorkspace; recordNavigation: (id: string) => void }) {
  const sections = ["General", "Appearance", "Runtime"] as const;
  const [section, setSection] = useState<typeof sections[number]>("General");
  const choose = (next: typeof sections[number]) => { setSection(next); recordNavigation(`settings:${next.toLowerCase()}`); };
  return <div className="settings-surface">
    <aside><h1>Settings</h1>{sections.map((item) => <button key={item} aria-pressed={section === item} onClick={() => choose(item)}>{item}</button>)}</aside>
    <div className="settings-content"><span>Studio local preferences</span><h2>{section}</h2>
      {section === "General" && <section><h3>Case navigation</h3><p>Back and Forward traverse Studio selection history. Opening another Case creates a new ephemeral attachment while durable continuity remains in YAI.</p></section>}
      {section === "Appearance" && <section><h3>Interface foundation</h3><p>Studio currently follows its permanent dark, system-font foundation. Theme mutation is not exposed in this vertical.</p></section>}
      {section === "Runtime" && <section><h3>Local application host</h3><dl><div><dt>Protocol</dt><dd>yai.studio.application.v1</dd></div><div><dt>Case</dt><dd>{workspace.case.case_ref}</dd></div><div><dt>Generation</dt><dd>{workspace.case.generation}</dd></div><div><dt>Transport</dt><dd>In-process Tauri bridge</dd></div></dl></section>}
    </div>
  </div>;
}

function SurfaceHeader({ workspace, title, body }: { workspace: LiveWorkspace; title: string; body: string }) { return <header className="surface-title"><small>{workspace.case.case_ref} · Generation {workspace.case.generation}</small><h1>{title}</h1><p>{body}</p></header>; }
function Overview({ workspace, inspect, navigate }: { workspace: LiveWorkspace; inspect: (id: string) => void; navigate: (perspective: Perspective) => void }) { const dimensions = overviewDimensions(workspace); return <div className="live-page case-overview"><SurfaceHeader workspace={workspace} title={workspace.case.display_name} body="The durable Case is the workspace: its world, knowledge, memory, authority, work and compute remain one continuity." /><section className="case-model" aria-label="Case composition"><div className="case-core" role="button" tabIndex={0} onClick={() => inspect(workspace.case.case_ref)} onKeyDown={(event) => { if (event.key === "Enter" || event.key === " ") inspect(workspace.case.case_ref); }}><Icon name="case" size={24} /><span>Case continuity</span><strong>{workspace.case.case_ref}</strong><dl><div><dt>Generation</dt><dd>{workspace.case.generation}</dd></div><div><dt>Lifecycle</dt><dd>{workspace.case.case_status}</dd></div><div><dt>Attached as</dt><dd>{workspace.case.participant_ref}</dd></div></dl></div><div className="case-dimensions">{dimensions.map((item) => <button key={item.perspective} onClick={() => navigate(item.perspective)}><Icon name={perspectiveIcon(item.perspective)} size={18} /><span><strong>{item.perspective}</strong><small>{item.meaning}</small></span><em>{item.detail}</em><Icon name="chevron" size={13} /></button>)}</div></section><section className="live-section"><PanelHeader title="Attention" detail="Deterministic consequences of current YAI facts" />{workspace.overview.attention.map((item) => <button className="fact-row" key={item.ref ?? item.title} onClick={() => inspect(item.ref ?? item.kind)}><Icon name={item.kind === "review" ? "review" : item.kind === "provider" ? "compute" : "work"} /><span><strong>{item.title}</strong><small>{item.detail}</small></span></button>)}{!workspace.overview.attention.length && <EmptyState title="No current attention" body="YAI exposes no pending review, unavailable provider or waiting workflow for this Case." />}</section><section className="live-section"><PanelHeader title="Recent committed activity" detail="Transition history" />{workspace.memory.timeline.slice(-6).reverse().map((entry) => <button className="timeline-row" key={entry.id} onClick={() => inspect(entry.id)}><time>{formatTime(entry.committed_at_unix_ms)}</time><span><strong>{humanize(entry.kind)}</strong><small>{entry.component} · sequence {entry.sequence}</small></span></button>)}</section></div>; }
function Environment({ workspace, inspect, openMaterial }: { workspace: LiveWorkspace; inspect: (id: string) => void; openMaterial: (id: string, label: string, pinned?: boolean) => void }) { return <div className="live-page"><SurfaceHeader workspace={workspace} title="Environment" body="Sources, files and operational resources attached to this Case by YAI." /><section className="live-section"><PanelHeader title="Files" detail="Qualified source revision inventory" />{workspace.environment.files.map((file) => <button className="fact-row" key={file.id} onClick={() => openMaterial(file.id, fileName(file.path))} onDoubleClick={() => openMaterial(file.id, fileName(file.path), true)}><Icon name="sources" /><span><strong>{file.path}</strong><small>{file.source_label} · {file.bytes.toLocaleString()} bytes</small></span></button>)}{!workspace.environment.files.length && <EmptyState title="No file inventory available" body="No acquired source revision exposes files for this Case. Declared sources remain visible below." />}</section><section className="live-section"><PanelHeader title="Sources" detail="Admitted source relationships" />{workspace.environment.sources.map((source) => <button className="fact-row" key={source.id} onClick={() => openMaterial(source.id, source.label)} onDoubleClick={() => openMaterial(source.id, source.label, true)}><Icon name="sources" /><span><strong>{source.label}</strong><small>{source.perimeter} · {source.media_type}</small></span><Badge tone={source.posture === "acquired" ? "success" : "warning"}>{source.posture ?? "declared"}</Badge></button>)}{!workspace.environment.sources.length && <EmptyState title="No sources attached" body="YAI exposes no admitted source relation for this Case." />}</section><section className="live-section"><PanelHeader title="Resources" detail="Operational attachments" />{workspace.environment.resources.map((resource) => <button className="fact-row" key={resource.id} onClick={() => inspect(resource.id)}><Icon name="environment" /><span><strong>{resource.id}</strong><small>{resource.kind} · policy {resource.policy_ref}</small></span><Badge>{resource.review_requirement}</Badge></button>)}{!workspace.environment.resources.length && <EmptyState title="No resources attached" body="No operational resource is currently attached to this Case." />}</section></div>; }
function Knowledge({ workspace, inspect }: { workspace: LiveWorkspace; inspect: (id: string) => void }) { const nodes = dedupeNodes([...workspace.knowledge.sources.map((source) => ({ id: source.id, label: source.label, kind: "source", detail: source.status })), ...workspace.knowledge.units.map((unit) => ({ id: unit.id, label: unit.entity_ref ?? compact(unit.text), kind: unit.predicate ? "claim" : unit.kind, detail: unit.posture })), ...workspace.knowledge.entities.map((entity) => ({ id: entity.id, label: compact(entity.id), kind: "entity" }))]); return <div className="live-page"><SurfaceHeader workspace={workspace} title="Knowledge" body="Qualified, source-grounded derivations available to this Case." />{workspace.knowledge.status !== "available" ? <EmptyState title="No qualified Knowledge derivation" body={workspace.knowledge.message} /> : <><GraphViewport nodes={nodes} edges={workspace.knowledge.relations} layout="relational" onInspect={inspect} /><section className="live-section"><PanelHeader title="Source-grounded units" detail={`${workspace.knowledge.units.length} qualified units`} />{workspace.knowledge.units.slice(0, 80).map((unit) => <button className="fact-row" key={unit.id} onClick={() => inspect(unit.id)}><Icon name="knowledge" /><span><strong>{unit.entity_ref ?? humanize(unit.kind)}</strong><small>{unit.text}</small></span><Badge>{unit.posture}</Badge></button>)}</section></>}</div>; }
function Memory({ workspace, inspect }: { workspace: LiveWorkspace; inspect: (id: string) => void }) { const [mode, setMode] = useState<"Timeline" | "Graph">("Timeline"); const graph = relationGraph(workspace); return <div className="live-page memory-page"><SurfaceHeader workspace={workspace} title="Memory" body="Committed history and derived relations. Neither view is canonical Case truth." /><div className="segmented"><button aria-pressed={mode === "Timeline"} onClick={() => setMode("Timeline")}>Timeline</button><button aria-pressed={mode === "Graph"} onClick={() => setMode("Graph")}>Graph</button></div>{mode === "Timeline" ? <ol className="real-timeline">{workspace.memory.timeline.slice().reverse().map((entry) => <li key={entry.id}><time>{formatTime(entry.committed_at_unix_ms)}</time><button onClick={() => inspect(entry.id)}><i /><span><strong>{humanize(entry.kind)}</strong><small>{entry.component} · generation {entry.sequence}</small></span></button></li>)}</ol> : <GraphViewport nodes={graph.nodes} edges={graph.edges} layout="relational" onInspect={inspect} />}</div>; }
function Authority({ workspace, inspect }: { workspace: LiveWorkspace; inspect: (id: string) => void }) { const graph = authorityGraph(workspace); return <div className="live-page"><SurfaceHeader workspace={workspace} title="Authority" body="Who may act, under which policy, review and decision facts." />{workspace.authority.empty ? <EmptyState title="No policy is bound to this Case" body="YAI exposes no policy, review or grant relationship for the current generation." /> : <GraphViewport nodes={graph.nodes} edges={graph.edges} layout="directed" onInspect={inspect} />}<section className="live-section"><PanelHeader title="Reviews" detail="Current authority posture" />{workspace.authority.reviews.map((review) => <button className="fact-row" key={review.id} onClick={() => inspect(review.id)}><Icon name="review" /><span><strong>{review.id}</strong><small>{review.operation_ref || "Operation unavailable"}</small></span><Badge tone={review.status === "pending" ? "warning" : "neutral"}>{review.status}</Badge></button>)}</section></div>; }
function Work({ workspace, inspect }: { workspace: LiveWorkspace; inspect: (id: string) => void }) { const nodes = workspace.work.nodes.map((node) => ({ id: node.node_id, label: node.node_id, kind: node.node_kind, detail: node.posture })); return <div className="live-page"><SurfaceHeader workspace={workspace} title="Work" body="Workflow definition and current progression remain distinct." />{workspace.work.status === "empty" ? <EmptyState title="No workflow is configured" body={workspace.work.message ?? "YAI exposes no workflow binding for this Case."} /> : <><GraphViewport nodes={nodes} edges={workspace.work.edges} layout="directed" onInspect={inspect} /><section className="live-section">{workspace.work.nodes.map((node) => <button className="fact-row" key={node.node_id} onClick={() => inspect(node.node_id)}><Icon name="work" /><span><strong>{node.node_id}</strong><small>{node.node_kind} · {node.reason}</small></span><Badge>{node.posture}</Badge></button>)}</section></>}</div>; }
function Compute({ workspace, inspect }: { workspace: LiveWorkspace; inspect: (id: string) => void; selected: string }) { return <div className="live-page"><SurfaceHeader workspace={workspace} title="Compute" body="Generic provider targets and qualified runtime posture exposed by YAI." /><section className="live-section">{workspace.compute.targets.map((target) => <button className="provider-row" key={target.id} onClick={() => inspect(target.id)}><Icon name="compute" /><span><strong>{target.provider_key}</strong><small>{target.adapter} · {target.model_id}</small></span><dl><dt>Locality</dt><dd>{target.locality}</dd><dt>Endpoint</dt><dd>{target.endpoint}</dd><dt>Management</dt><dd>{target.management}</dd></dl></button>)}{!workspace.compute.targets.length && <EmptyState title="No provider is connected" body={workspace.compute.message} />}</section></div>; }

function ContextPanel({ workspace, mode, setMode, selected, inspect, close, openMaterial }: { workspace: LiveWorkspace; mode: ContextMode; setMode: (mode: ContextMode) => void; selected: string; inspect: (id: string) => void; close: () => void; openMaterial: (id: string, label: string, pinned?: boolean) => void }) {
  const fact = findFact(workspace, selected);
  const kind = factKind(workspace, selected);
  const material = isMaterial(workspace, selected);
  const relations = workspace.memory.relations.filter((edge) => edge.from === selected || edge.to === selected).slice(0, 8);
  return <aside className="live-context"><header><div className="segmented">{(["Conversation", "Inspector", "Activity"] as ContextMode[]).map((item) => <button key={item} aria-pressed={mode === item} onClick={() => setMode(item)}>{item}</button>)}</div><IconButton aria-label="Close context panel" onClick={close}><Icon name="right" /></IconButton></header>
    {mode === "Conversation" && <div className="context-scroll"><PanelHeader title="Conversation" detail={workspace.conversation.read_only ? "Read only" : "Available"} />{workspace.conversation.turns.map((turn) => <article className="real-turn" key={turn.id}><header><strong>{turn.participant_ref}</strong><Badge>Generation {turn.generation}</Badge></header>{turn.parts.map((part, index) => <p key={index}>{part.text ?? `[${part.modality} · ${part.media_type}]`}</p>)}</article>)}{!workspace.conversation.turns.length && <EmptyState title="No committed turns" body="This Case has no conversation Turns visible to the current participant." />}<div className="read-only-draft"><textarea disabled placeholder="Send is not qualified in this vertical" /><small>Conversation write is unavailable. Studio will not fake-send.</small></div></div>}
    {mode === "Inspector" && <div className="context-scroll inspector"><PanelHeader title="Inspector" detail="Context in this Case" /><section className="inspector-hero"><div><Icon name={factIcon(kind)} size={18} /></div><span>{humanize(kind)}</span><h2>{fact.title}</h2><p>{fact.detail}</p>{material && <Button onClick={() => openMaterial(selected, materialLabel(workspace, selected), true)}>Open in work surface</Button>}</section><section className="inspector-group"><h3>Properties</h3><dl>{fact.values.map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl></section><section className="inspector-group"><h3>Case relationships <span>{relations.length}</span></h3>{relations.map((edge) => { const related = edge.from === selected ? edge.to : edge.from; return <button key={edge.id} onClick={() => inspect(related)}><span>{edge.from === selected ? "To" : "From"}</span><strong>{compact(related)}</strong><small>{humanize(edge.kind)}</small></button>; })}{!relations.length && <p>No qualified graph relation is exposed for this fact.</p>}</section><footer>Inspector selection and navigation are local UI state. Case facts remain owned by YAI.</footer></div>}
    {mode === "Activity" && <div className="context-scroll"><PanelHeader title="Activity" detail={`Through generation ${workspace.case.generation}`} />{workspace.memory.timeline.slice(-20).reverse().map((entry) => <div className="context-event" key={entry.id}><time>{formatTime(entry.committed_at_unix_ms)}</time><span><strong>{humanize(entry.kind)}</strong><small>{entry.component}</small></span></div>)}</div>}
  </aside>;
}

function BottomPanel({ workspace, active, setActive, close, height }: { workspace: LiveWorkspace; active: BottomMode; setActive: (mode: BottomMode) => void; close: () => void; height: number }) { const tabs: BottomMode[] = ["Terminal", "Output", "Executions", "Evidence", "Problems"]; return <section className="live-bottom" style={{ height }}><header>{tabs.map((tab) => <button key={tab} aria-pressed={active === tab} onClick={() => setActive(tab)}>{tab}</button>)}<span /><IconButton aria-label="Close bottom panel" onClick={close}><Icon name="close" /></IconButton></header><div className="tool-content">{active === "Terminal" && <div className="terminal-unavailable"><code>YAI Studio</code><strong>Terminal host not attached</strong><p>This live Case vertical does not provide PTY capability.</p></div>}{active === "Output" && <pre>application.protocol={"yai.studio.application.v1"}{"\n"}case={workspace.case.case_ref}{"\n"}generation={workspace.case.generation}{"\n"}projection=case.summary</pre>}{active === "Executions" && <EmptyState title="No execution projection" body="No dedicated execution view is exposed by this bounded host." />}{active === "Evidence" && <EmptyState title="No evidence projection" body="Evidence remains linked through the real authority and workflow facts where present." />}{active === "Problems" && <EmptyState title="No client problems" body="Unavailable YAI facts are rendered in their owning surface." />}</div></section>; }

function perspectiveRows(workspace: LiveWorkspace, perspective: Perspective) { const icon = (name: string) => name as Parameters<typeof Icon>[0]["name"]; if (perspective === "Environment") return [{ label: "Files", items: workspace.environment.files.map((item) => ({ id: item.id, label: fileName(item.path), detail: item.path, icon: icon("sources"), material: true })) }, { label: "Sources", items: workspace.environment.sources.map((item) => ({ id: item.id, label: item.label, detail: item.posture ?? "declared", icon: icon("sources"), material: true })) }, { label: "Resources", items: workspace.environment.resources.map((item) => ({ id: item.id, label: item.id, detail: item.kind, icon: icon("environment") })) }]; if (perspective === "Knowledge") return [{ label: "Documents", items: workspace.knowledge.sources.map((item) => ({ id: item.id, label: item.label, detail: item.path, icon: icon("sources"), material: true })) }, { label: "Entities", items: workspace.knowledge.entities.map((item) => ({ id: item.id, label: compact(item.id), detail: `${item.definitions.length} definitions`, icon: icon("knowledge") })) }]; if (perspective === "Authority") return [{ label: "Reviews", items: workspace.authority.reviews.map((item) => ({ id: item.id, label: item.id, detail: item.status, icon: icon("review") })) }]; if (perspective === "Work") return [{ label: "Workflow", items: workspace.work.nodes.map((item) => ({ id: item.node_id, label: item.node_id, detail: item.posture, icon: icon("work") })) }]; if (perspective === "Compute") return [{ label: "Targets", items: workspace.compute.targets.map((item) => ({ id: item.id, label: item.provider_key, detail: item.model_id, icon: icon("compute") })) }]; if (perspective === "Memory") return [{ label: "History", items: workspace.memory.timeline.slice(-12).reverse().map((item) => ({ id: item.id, label: humanize(item.kind), detail: `Generation ${item.sequence}`, icon: icon("memory") })) }]; return []; }
function relationGraph(workspace: LiveWorkspace) { const kinds = new Map<string, string>(); workspace.memory.relations.forEach((edge) => { kinds.set(edge.from, edge.from_kind ?? "fact"); kinds.set(edge.to, edge.to_kind ?? "fact"); }); return { nodes: [...kinds].map(([id, kind]) => ({ id, label: compact(id), kind })), edges: workspace.memory.relations }; }
function overviewDimensions(workspace: LiveWorkspace): Array<{ perspective: Exclude<Perspective, "Overview">; meaning: string; detail: string }> { return [
  { perspective: "Environment", meaning: "World attached to the Case", detail: `${workspace.environment.sources.length} sources · ${workspace.environment.resources.length} resources` },
  { perspective: "Knowledge", meaning: "Qualified source-grounded derivation", detail: workspace.knowledge.status === "available" ? `${workspace.knowledge.units.length} units · ${workspace.knowledge.entities.length} entities` : workspace.knowledge.status },
  { perspective: "Memory", meaning: "Committed history and derived relations", detail: `${workspace.memory.timeline.length} events · ${workspace.memory.relations.length} relations` },
  { perspective: "Authority", meaning: "Policy, scope, review and decision", detail: workspace.authority.empty ? "No bound authority facts" : `${workspace.authority.policies.length} policies · ${workspace.authority.reviews.length} reviews` },
  { perspective: "Work", meaning: "Workflow definition and progression", detail: workspace.work.status },
  { perspective: "Compute", meaning: "Provider and runtime posture", detail: workspace.compute.status === "available" ? `${workspace.compute.targets.length} targets` : workspace.compute.status },
]; }
function authorityGraph(workspace: LiveWorkspace) { const nodes: LiveNode[] = [{ id: workspace.case.participant_ref, label: compact(workspace.case.participant_ref), kind: "participant" }]; const edges: LiveEdge[] = []; workspace.authority.reviews.forEach((review, index) => { nodes.push({ id: review.id, label: compact(review.id), kind: "review" }); edges.push({ id: `authority:${index}:participant-review`, from: workspace.case.participant_ref, to: review.id, kind: "scope" }); if (review.policy_ref) { nodes.push({ id: review.policy_ref, label: compact(review.policy_ref), kind: "policy" }); edges.push({ id: `authority:${index}:policy-review`, from: review.policy_ref, to: review.id, kind: "evaluates" }); } if (review.decision_ref) { nodes.push({ id: review.decision_ref, label: compact(review.decision_ref), kind: "decision" }); edges.push({ id: `authority:${index}:review-decision`, from: review.id, to: review.decision_ref, kind: "resolved by" }); } }); return { nodes: dedupeNodes(nodes), edges }; }
function dedupeNodes(nodes: LiveNode[]) { return [...new Map(nodes.map((node) => [node.id, node])).values()]; }
function findFact(workspace: LiveWorkspace, id: string) {
  const values = (...rows: Array<[string, string]>): Array<[string, string]> => rows;
  if (id === workspace.case.case_ref) return { title: workspace.case.display_name, detail: "Durable continuity and application truth owned by YAI.", values: values(["Case", workspace.case.case_ref], ["Tenant", workspace.case.tenant_ref ?? "Unavailable"], ["Lifecycle", workspace.case.case_status], ["Generation", String(workspace.case.generation)], ["Attached participant", workspace.case.participant_ref], ["Participants", String(workspace.overview.participants.length)], ["Sources", String(workspace.environment.sources.length)], ["Files", String(workspace.environment.files.length)], ["Resources", String(workspace.environment.resources.length)], ["Committed events", String(workspace.memory.timeline.length)], ["Derived relations", String(workspace.memory.relations.length)], ["Knowledge", workspace.knowledge.status], ["Authority", workspace.authority.empty ? "No bound facts" : "Available"], ["Workflow", workspace.work.status], ["Compute", workspace.compute.status]) };
  const file = workspace.environment.files.find((item) => item.id === id);
  if (file) return { title: fileName(file.path), detail: file.path, values: values(["Source", file.source_label], ["Revision", file.revision_ref], ["Bytes", file.bytes.toLocaleString()], ["Digest", file.digest]) };
  const source = workspace.environment.sources.find((item) => item.id === id);
  if (source) return { title: source.label, detail: source.perimeter, values: values(["Source", source.id], ["Media type", source.media_type], ["Roles", source.roles.join(", ")], ["Posture", source.posture ?? "declared"], ["Revision", source.revision_ref ?? "Unavailable"], ["Files", String(source.items ?? 0)]) };
  const document = workspace.knowledge.sources.find((item) => item.id === id);
  if (document) return { title: document.label, detail: document.path, values: values(["Source", document.source_ref], ["Revision", document.revision_ref], ["Media type", document.media_type], ["Extractor", document.extractor], ["Status", document.status], ["Digest", document.digest]) };
  const participant = workspace.overview.participants.find((item) => item.id === id);
  if (participant) return { title: participant.id, detail: participant.is_current ? "Current attached participant" : "Participant in this Case", values: values(["Participant", participant.id], ["Roles", participant.roles.join(", ") || "No role exposed"], ["Current attachment", participant.is_current ? "Yes" : "No"]) };
  const timeline = workspace.memory.timeline.find((item) => item.id === id);
  if (timeline) return { title: humanize(timeline.kind), detail: timeline.summary ?? "Committed Case transition", values: values(["Reference", timeline.id], ["Generation", String(timeline.sequence)], ["Owner", timeline.component], ["Participant", timeline.participant_ref ?? "Unavailable"], ["Committed", formatTime(timeline.committed_at_unix_ms)]) };
  const review = workspace.authority.reviews.find((item) => item.id === id);
  if (review) return { title: compact(review.id), detail: "Authority review bound to this Case.", values: values(["Status", review.status], ["Operation", review.operation_ref], ["Policy", review.policy_ref], ["Decision", review.decision_ref ?? "Pending"], ["Evidence", review.evidence_ref ?? "Unavailable"]) };
  const work = workspace.work.nodes.find((item) => item.node_id === id);
  if (work) return { title: work.node_id, detail: work.reason, values: values(["Kind", work.node_kind], ["Posture", work.posture], ["Case", workspace.case.case_ref]) };
  const knowledge = workspace.knowledge.units.find((item) => item.id === id);
  if (knowledge) return { title: knowledge.entity_ref ?? humanize(knowledge.kind), detail: knowledge.text, values: values(["Reference", knowledge.id], ["Posture", knowledge.posture], ["Source", knowledge.source_ref]) };
  const provider = workspace.compute.targets.find((item) => item.id === id);
  if (provider) return { title: provider.provider_key, detail: provider.model_id, values: values(["Target", provider.id], ["Adapter", provider.adapter], ["Locality", provider.locality], ["Endpoint", provider.endpoint], ["Management", provider.management]) };
  return { title: compact(id), detail: "Selected Case reference", values: values(["Reference", id], ["Case", workspace.case.case_ref], ["Generation", String(workspace.case.generation)]) };
}
function perspectiveIcon(value: Perspective) { return ({ Overview: "overview", Environment: "environment", Knowledge: "knowledge", Memory: "memory", Authority: "authority", Work: "work", Compute: "compute" } as const)[value]; }
function compact(value: string) { return value.length > 38 ? `${value.slice(0, 20)}…${value.slice(-12)}` : value; }
function fileName(value: string) { return value.split(/[\\/]/).filter(Boolean).at(-1) ?? value; }
function materialLabel(workspace: LiveWorkspace, id: string) { const file = workspace.environment.files.find((item) => item.id === id); if (file) return fileName(file.path); return workspace.environment.sources.find((item) => item.id === id)?.label ?? workspace.knowledge.sources.find((item) => item.id === id)?.label ?? compact(id); }
function isMaterial(workspace: LiveWorkspace, id: string) { return workspace.environment.files.some((item) => item.id === id) || workspace.environment.sources.some((item) => item.id === id) || workspace.knowledge.sources.some((item) => item.id === id); }
function factKind(workspace: LiveWorkspace, id: string) {
  if (id === workspace.case.case_ref) return "case";
  if (workspace.environment.files.some((item) => item.id === id)) return "file";
  if (workspace.environment.sources.some((item) => item.id === id) || workspace.knowledge.sources.some((item) => item.id === id)) return "source";
  if (workspace.overview.participants.some((item) => item.id === id)) return "participant";
  if (workspace.authority.reviews.some((item) => item.id === id)) return "review";
  if (workspace.compute.targets.some((item) => item.id === id)) return "provider";
  if (workspace.work.nodes.some((item) => item.node_id === id)) return "workflow";
  if (workspace.memory.timeline.some((item) => item.id === id)) return "event";
  return "case fact";
}
function factIcon(kind: string): Parameters<typeof Icon>[0]["name"] { return ({ case: "case", file: "sources", source: "sources", participant: "people", review: "review", provider: "compute", workflow: "work", event: "memory" } as Record<string, Parameters<typeof Icon>[0]["name"]>)[kind] ?? "overview"; }
function humanize(value: string) { return value.replaceAll("_", " ").replace(/^./, (char) => char.toUpperCase()); }
function formatTime(value: number) { return new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit", second: "2-digit" }).format(value); }
