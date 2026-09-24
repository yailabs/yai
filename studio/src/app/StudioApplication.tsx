import { CaseLifecycleDialog, NewCaseDialog, ParticipantAccessDialog, useApplicationAvailability } from "../contrib/case/applicationActions";
import { DisposableStore } from "../platform/lifecycle";
import { when } from "../platform/context";
import { useModalFocus } from "../components/useModalFocus";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CaseAttachment, CaseUpdate, LiveCaseRow, OperationResult } from "../clients/live";
import type { CaseCatalog, CaseDataSource, CasePresentation } from "../clients/dataSource";
import type { PlatformServices } from "../platform/services";
import type { WorkbenchRegistry } from "../workbench/kernel/registry";
import { WorkbenchKernel } from "../workbench/kernel/WorkbenchKernel";
import { Badge, Button, EmptyState, IconButton, PanelHeader, SearchInput } from "../components/primitives";
import { Icon } from "../components/Icon";
import { DesktopWindowControls } from "./DesktopWindowControls";
import { beginDesktopWindowDrag, toggleDesktopWindowMaximize } from "../platform/desktopWindow";
import { CaseComposer } from "../start/CaseBootstrap";

export function StudioApplication({ dataSource, platform, registry }: { dataSource: CaseDataSource; platform: PlatformServices; registry: WorkbenchRegistry }) {
  const applicationAvailability = useApplicationAvailability(platform.application);
  const [productAction, setProductAction] = useState<"create" | "close" | "cancel">();
  const [accessCase, setAccessCase] = useState<string>();
  const [catalog, setCatalog] = useState<OperationResult<CaseCatalog>>();
  const [attachment, setAttachment] = useState<CaseAttachment>();
  const [workspace, setWorkspace] = useState<OperationResult<CasePresentation>>();
  const [switcher, setSwitcher] = useState(false);
  const [composer, setComposer] = useState(false);
  const [stream, setStream] = useState<"connecting" | "live" | "reconnecting" | "unavailable" | "fixture">(dataSource.kind === "fixture" ? "fixture" : "connecting");
  const readMaterial = useMemo(() => dataSource.readMaterial.bind(dataSource), [dataSource]);
  const searchCase = useMemo(() => dataSource.searchCase?.bind(dataSource), [dataSource]);
  const loadCases = useCallback(async () => setCatalog(await dataSource.listCases()), [dataSource]);
  const workspaceRef = useRef(workspace);
  workspaceRef.current = workspace;
  const opening = useRef(0);
  const refreshing = useRef(false);
  const [pendingCase, setPendingCase] = useState<string>();
  const [connectionError, setConnectionError] = useState<string>();
  const [openFailure, setOpenFailure] = useState<OperationResult<unknown>>();
  const requestedCase = useRef<string | undefined>(undefined);
  const loadCase = useCallback(async (caseRef: string) => {
    const request = ++opening.current;
    requestedCase.current = caseRef;
    setOpenFailure(undefined);
    setPendingCase(caseRef);
    setConnectionError(undefined);
    try {
      const opened = await dataSource.openCase(caseRef);
      if (request !== opening.current) return;
      if (opened.result_state !== "success" || !opened.data) {
        if (opened.error?.code === "authenticated_principal_participant_link_required") setAccessCase(caseRef);
        setConnectionError(opened.error?.safe_message ?? `Case open ${opened.result_state}.`);
        setOpenFailure(opened);
        return;
      }
      const summary = await dataSource.caseSummary(caseRef);
      if (request !== opening.current) return;
      if (summary.result_state !== "success" || !summary.data) {
        setConnectionError(summary.error?.safe_message ?? `Case read ${summary.result_state}.`);
        setOpenFailure(summary);
        return;
      }
      // Commit an attachment and its snapshot together. Pending/error responses
      // never tear down the window's drafts, terminal sessions or last snapshot.
      setAttachment(opened.data); setWorkspace(summary); setComposer(false);
      const query = dataSource.kind === "fixture" ? `?fixture=${caseRef.replace(/^fixture:/, "")}` : `?case=${encodeURIComponent(caseRef)}`;
      window.history.replaceState({ case_ref: caseRef }, "", `${window.location.pathname}${query}`);
    } catch (error) { if (request === opening.current) setConnectionError(String(error)); }
    finally { if (request === opening.current) setPendingCase(undefined); }
  }, [dataSource]);
  const refresh = useCallback(async (expectedGeneration?: number) => {
    const before = workspaceRef.current?.data;
    if (!before || refreshing.current) return;
    const epoch = opening.current;
    refreshing.current = true;
    try {
      let result = await dataSource.caseSummary(before.case.case_ref, expectedGeneration);
      if (result.result_state === "stale") result = await dataSource.caseSummary(before.case.case_ref);
      if (epoch !== opening.current) return;
      if (result.result_state === "success" && result.data) {
        setWorkspace((current) => current?.data && current.data.case.generation > result.data!.case.generation ? current : result);
        setConnectionError(undefined);
        setStream(dataSource.kind === "live" ? "live" : "fixture");
      } else {
        setConnectionError(result.error?.safe_message ?? `Refresh ${result.result_state}. Showing the last snapshot.`);
        setStream("unavailable");
      }
    } catch (error) { if (epoch === opening.current) { setConnectionError(String(error)); setStream("unavailable"); } }
    finally { refreshing.current = false; }
  }, [dataSource]);
  const reattach = useCallback(async () => {
    const caseRef = workspaceRef.current?.data?.case.case_ref;
    if (!caseRef) { await loadCases(); return; }
    setStream("reconnecting");
    const opened = await dataSource.openCase(caseRef);
    if (opened.result_state !== "success" || !opened.data) { setStream("unavailable"); return; }
    if (workspaceRef.current?.data?.case.case_ref !== caseRef) return;
    setAttachment(opened.data);
    await refresh();
  }, [dataSource, loadCases, refresh]);

  useEffect(() => {
    const registered = new DisposableStore();
    for (const action of ["create", "close", "cancel"] as const) {
      const id = `studio.case.${action}`;
      registered.add(platform.commands.registerCommand({ id, title: action === "create" ? "New Case…" : action === "close" ? "Close Case…" : "Cancel Case…", when: when.truthy(`application.case.${action}`), handler: () => setProductAction(action) }));
      registered.add(platform.menus.registerMenuItem({ id: `application:${id}`, location: action === "create" ? "File" : "Case", group: action === "create" ? "0" : "4", command: id, order: action === "cancel" ? 1 : 0 }));
    }
    return () => registered.dispose();
  }, [platform]);
  useEffect(() => {
    for (const action of ["create", "close", "cancel"]) platform.context.update(`application.case.${action}`, Boolean(platform.application?.supports(`case.${action}`) && (action === "create" || workspace?.data?.case.case_status === "open")));
  }, [platform, applicationAvailability, workspace?.data?.case.case_status]);

  useEffect(() => { void loadCases(); }, [loadCases]);
  const initialOpen = useRef(false);
  useEffect(() => {
    if (initialOpen.current) return;
    initialOpen.current = true;
    const parameters = new URLSearchParams(window.location.search);
    const requested = dataSource.kind === "fixture" ? parameters.get("fixture") : parameters.get("case");
    if (requested) void loadCase(dataSource.kind === "fixture" ? `fixture:${requested}` : requested);
  }, [dataSource.kind, loadCase]);
  useEffect(() => {
    if (!dataSource.subscribe || !dataSource.listen) return;
    let disposed = false;
    let stop: (() => void) | undefined;
    void dataSource.subscribe().then((result) => { if (!disposed) setStream(result.result_state === "success" ? "live" : "unavailable"); });
    void dataSource.listen((update: CaseUpdate) => {
      if (disposed) return;
      setStream("live");
      const current = workspaceRef.current?.data;
      if (update.case_ref === current?.case.case_ref && update.generation !== current.case.generation) void refresh(update.generation);
      void loadCases();
    }).then((unlisten) => { if (disposed) unlisten(); else stop = unlisten; }).catch(() => { if (!disposed) setStream("unavailable"); });
    return () => { disposed = true; stop?.(); };
  }, [dataSource, loadCases, refresh]);
  useEffect(() => {
    if (!attachment || !dataSource.heartbeat) return;
    let active = true;
    const heartbeat = async () => {
      const result = await dataSource.heartbeat!(attachment.case_ref);
      if (!active) return;
      if (result.result_state !== "success" || !result.data) { setStream("unavailable"); return; }
      setStream("live");
      if (result.data.generation !== workspace?.data?.case.generation) void refresh(result.data.generation);
    };
    const timer = window.setInterval(() => void heartbeat(), 1000); void heartbeat();
    return () => { active = false; window.clearInterval(timer); };
  }, [attachment, dataSource, refresh, workspace?.data?.case.generation]);
  useEffect(() => {
    if (dataSource.kind !== "live") return;
    return platform.host.subscribe((host) => {
      if (host.state === "starting") setStream("connecting");
      if (host.state === "reconnecting") setStream("reconnecting");
      if (host.state === "unavailable" || host.state === "stopped") setStream("unavailable");
      if (host.state === "live" && host.resync_required) void reattach();
      else if (host.state === "live") setStream("live");
    }).dispose;
  }, [dataSource.kind, platform.host, reattach]);
  useEffect(() => {
    const guard = () => { if (attachment) window.history.replaceState({ case_ref: attachment.case_ref }, "", window.location.href); };
    window.addEventListener("popstate", guard); return () => window.removeEventListener("popstate", guard);
  }, [attachment]);
  useEffect(() => { document.title = workspace?.data ? `${workspace.data.case.display_name} — YAI Studio` : "YAI Studio"; }, [workspace?.data]);

  const cases = catalog?.data?.cases ?? [];
  if (composer && dataSource.composition) return <ApplicationFrame platform={platform}><CaseComposer sections={dataSource.composition()} close={() => setComposer(false)} /></ApplicationFrame>;
  return <div className={`live-studio ${attachment ? "case-attached" : ""}`}>
    {!attachment && <StartChrome platform={platform} />}
    {!attachment && <StartCenter result={openFailure ?? catalog} cases={cases} dataKind={dataSource.kind} open={loadCase} retry={() => openFailure && requestedCase.current ? void loadCase(requestedCase.current) : void loadCases()} newCase={dataSource.composition ? () => setComposer(true) : platform.application?.supports("case.create") ? () => setProductAction("create") : undefined} />}
    {attachment && workspace?.data && <WorkbenchKernel workspace={workspace.data} stream={stream} platform={platform} registry={registry} readMaterial={readMaterial} searchCase={searchCase} refresh={() => refresh()} openCaseSwitcher={() => setSwitcher(true)} />}
    {attachment && !workspace?.data && workspace && workspace.result_state !== "success" && <HostFailure result={workspace} retry={() => void refresh()} />}
    {(pendingCase || connectionError) && <div className="connection-notice" role={connectionError ? "alert" : "status"}>{pendingCase ? "Opening Case…" : <><span>{connectionError}{attachment && " Your open work is retained."}</span><Button onClick={() => openFailure && requestedCase.current ? void loadCase(requestedCase.current) : attachment ? void refresh() : void loadCases()}>Retry</Button><IconButton aria-label="Dismiss connection notice" onClick={() => setConnectionError(undefined)}><Icon name="close" /></IconButton></>}</div>}
    {productAction === "create" && platform.application && <NewCaseDialog application={platform.application} close={() => { setProductAction(undefined); void loadCases(); }} created={async caseRef => { await loadCases(); await loadCase(caseRef); }} />}
    {accessCase && platform.application && !productAction && <ParticipantAccessDialog application={platform.application} caseRef={accessCase} close={() => setAccessCase(undefined)} attached={async () => { await loadCases(); await loadCase(accessCase); }} />}
    {(productAction === "close" || productAction === "cancel") && platform.application && workspace?.data && <CaseLifecycleDialog application={platform.application} workspace={workspace.data} action={productAction} close={() => setProductAction(undefined)} refresh={async () => { await refresh(); await loadCases(); }} />}
    {switcher && <CaseSwitcher cases={cases} dataKind={dataSource.kind} close={() => setSwitcher(false)} open={(id) => { setSwitcher(false); void loadCase(id); }} />}
  </div>;
}

function ApplicationFrame({ platform, children }: { platform: PlatformServices; children: React.ReactNode }) { return <div className="live-studio"><StartChrome platform={platform} />{children}</div>; }
function StartChrome({ platform }: { platform: PlatformServices }) { return <nav className="native-menu" aria-label="Application menu" data-tauri-drag-region onPointerDown={beginDesktopWindowDrag} onDoubleClick={toggleDesktopWindowMaximize}><strong>YAI</strong><span className="native-menu-drag" data-tauri-drag-region>YAI Studio</span><DesktopWindowControls />{platform.host.capabilities.nativeDesktop && <span className="sr-only">Native desktop host</span>}</nav>; }
function StartCenter({ result, cases, dataKind, open, retry, newCase }: { result?: OperationResult<unknown>; cases: LiveCaseRow[]; dataKind: "live" | "fixture"; open: (id: string) => void; retry: () => void; newCase?: () => void }) {
  const [query, setQuery] = useState(""); const visible = cases.filter((item) => `${item.case_ref} ${item.display_name}`.toLowerCase().includes(query.toLowerCase()));
  return <main className="live-start"><section className="live-start-intro"><span>{dataKind === "live" ? "Local Case Workbench" : "Fixture development Workbench"}</span><h1>Open a Case.</h1><p>{dataKind === "live" ? "Studio attaches to durable Case continuity owned by YAI on this machine." : "Explicit authored Case data runs through the same Workbench as live data."}</p><SearchInput autoFocus aria-label="Search Cases" placeholder={dataKind === "live" ? "Search real local Cases" : "Search fixture Cases"} value={query} onChange={(event) => setQuery(event.target.value)} />{newCase && <Button onClick={newCase}>{dataKind === "live" ? "New Case" : "Compose fixture Case"}</Button>}</section>{result?.result_state && result.result_state !== "success" ? <HostFailure result={result} retry={retry} /> : <section className="live-case-list" aria-label={dataKind === "live" ? "Real local Cases" : "Fixture Cases"}><PanelHeader title={dataKind === "live" ? "Local Cases" : "Fixture Cases"} detail={`${visible.length} available`} />{visible.map((item) => <button className="live-case-row" data-case-ref={item.case_ref} key={item.case_ref} onClick={() => open(item.case_ref)}><Icon name="case" size={18} /><span><strong>{item.display_name}</strong><small>{item.case_status}</small></span><span className="case-facts">{item.source_count} sources · {item.participant_count} participants</span><Icon name="chevron" size={14} /></button>)}{!result ? <p role="status" className="surface-loading">Loading Cases…</p> : !visible.length && <EmptyState title={cases.length ? "No matching Cases" : "No Cases"} body={cases.length ? "Try another name or clear the search to see your Cases." : dataKind === "live" ? "YAI returned an empty authorized Case list. Studio has not substituted sample data." : "No authored Cases are available."} />}</section>}<footer><Badge tone={dataKind === "live" ? "success" : "warning"}>{dataKind === "live" ? "Real local data" : "Fixture Case data"}</Badge><span>{dataKind === "live" ? "No fixture fallback." : "Explicit development mode."}</span></footer></main>;
}
function HostFailure({ result, retry }: { result: OperationResult<unknown>; retry: () => void }) { return <main className="host-failure"><Icon name="warning" size={28} /><h1>Local YAI unavailable</h1><p>{result.error?.safe_message ?? "YAI did not return this projection."}</p><code>{result.result_state} · {result.error?.code}</code><Button onClick={retry}>Retry</Button><small>No fixture data has been loaded.</small></main>; }
function CaseSwitcher({ cases, dataKind, close, open }: { cases: LiveCaseRow[]; dataKind: "live" | "fixture"; close: () => void; open: (id: string) => void }) { const root = useModalFocus(close); const [query, setQuery] = useState(""); return <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) close(); }}><section ref={root} className="case-switcher" role="dialog" aria-modal="true" aria-label="Open Case"><PanelHeader title="Open Case" detail={dataKind === "live" ? "Real local YAI" : "Explicit fixture data"} actions={<IconButton aria-label="Close" onClick={close}><Icon name="close" /></IconButton>} /><SearchInput autoFocus placeholder="Search Cases" value={query} onChange={(event) => setQuery(event.target.value)} />{cases.filter((item) => `${item.display_name} ${item.case_ref}`.toLocaleLowerCase().includes(query.toLocaleLowerCase())).map((item) => <button className="ui-list-row" key={item.case_ref} data-case-ref={item.case_ref} title={item.case_ref} onClick={() => open(item.case_ref)}><Icon name="case" /><span><strong>{item.display_name}</strong><small>{item.case_status}</small></span></button>)}</section></div>; }
