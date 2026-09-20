import { useCallback, useEffect, useState } from "react";
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
  const [catalog, setCatalog] = useState<OperationResult<CaseCatalog>>();
  const [attachment, setAttachment] = useState<CaseAttachment>();
  const [workspace, setWorkspace] = useState<OperationResult<CasePresentation>>();
  const [switcher, setSwitcher] = useState(false);
  const [composer, setComposer] = useState(false);
  const [stream, setStream] = useState<"connecting" | "live" | "reconnecting" | "unavailable" | "fixture">(dataSource.kind === "fixture" ? "fixture" : "connecting");
  const [cursor, setCursor] = useState<string>();
  const loadCases = useCallback(async () => setCatalog(await dataSource.listCases()), [dataSource]);
  const loadCase = useCallback(async (caseRef: string) => {
    const opened = await dataSource.openCase(caseRef);
    if (opened.result_state !== "success" || !opened.data) {
      setAttachment(undefined);
      setCatalog({ operation_ref: opened.operation_ref, result_state: opened.result_state, correlation_ref: opened.correlation_ref, error: opened.error });
      setWorkspace(undefined);
      return;
    }
    setAttachment(opened.data); setWorkspace(undefined); setComposer(false);
    setWorkspace(await dataSource.caseSummary(caseRef));
    const query = dataSource.kind === "fixture"
      ? `?fixture=${caseRef.replace(/^fixture:/, "")}`
      : `?case=${encodeURIComponent(caseRef)}`;
    window.history.replaceState({ case_ref: caseRef }, "", `${window.location.pathname}${query}`);
  }, [dataSource]);
  const refresh = useCallback(async (expectedGeneration?: number) => {
    if (!attachment) return;
    const result = await dataSource.caseSummary(attachment.case_ref, expectedGeneration);
    if (result.result_state === "stale") {
      setStream("reconnecting");
      const resynced = await dataSource.caseSummary(attachment.case_ref);
      setWorkspace(resynced); setStream(resynced.result_state === "success" ? (dataSource.kind === "live" ? "live" : "fixture") : "unavailable");
      return;
    }
    setWorkspace(result);
  }, [attachment, dataSource]);

  useEffect(() => { void loadCases(); }, [loadCases]);
  useEffect(() => {
    if (attachment) return;
    const parameters = new URLSearchParams(window.location.search);
    const requested = dataSource.kind === "fixture" ? parameters.get("fixture") : parameters.get("case");
    if (requested) void loadCase(dataSource.kind === "fixture" ? `fixture:${requested}` : requested);
  }, [attachment, dataSource.kind, loadCase]);
  useEffect(() => {
    if (!dataSource.subscribe || !dataSource.listen) return;
    let stop: () => void = () => undefined;
    void dataSource.subscribe(cursor).then((result) => setStream(result.result_state === "success" ? "live" : "unavailable"));
    void dataSource.listen((update: CaseUpdate) => {
      setCursor(update.cursor); setStream("live");
      if (update.case_ref === attachment?.case_ref && update.generation !== workspace?.data?.case.generation) void refresh(update.generation);
      void loadCases();
    }).then((unlisten) => { stop = unlisten; });
    return () => stop();
  }, [attachment?.case_ref, dataSource, loadCases, refresh, workspace?.data?.case.generation]);
  useEffect(() => {
    if (!attachment || !dataSource.heartbeat) return;
    let active = true;
    const heartbeat = async () => {
      const result = await dataSource.heartbeat!(attachment.case_ref);
      if (!active) return;
      if (result.result_state !== "success" || !result.data) { setStream("unavailable"); return; }
      setCursor(result.data.cursor); setStream("live");
      if (result.data.generation !== workspace?.data?.case.generation) void refresh(result.data.generation);
    };
    const timer = window.setInterval(() => void heartbeat(), 1000); void heartbeat();
    return () => { active = false; window.clearInterval(timer); };
  }, [attachment, dataSource, refresh, workspace?.data?.case.generation]);
  useEffect(() => {
    const guard = () => { if (attachment) window.history.replaceState({ case_ref: attachment.case_ref }, "", window.location.href); };
    window.addEventListener("popstate", guard); return () => window.removeEventListener("popstate", guard);
  }, [attachment]);
  useEffect(() => { document.title = attachment ? `${attachment.case_ref} — YAI Studio` : "YAI Studio"; }, [attachment]);

  const cases = catalog?.data?.cases ?? [];
  if (composer && dataSource.composition) return <ApplicationFrame platform={platform}><CaseComposer sections={dataSource.composition()} close={() => setComposer(false)} /></ApplicationFrame>;
  return <div className={`live-studio ${attachment ? "case-attached" : ""}`}>
    {!attachment && <StartChrome platform={platform} />}
    {!attachment && <StartCenter result={catalog} cases={cases} dataKind={dataSource.kind} open={loadCase} retry={loadCases} newCase={dataSource.composition ? () => setComposer(true) : undefined} />}
    {attachment && workspace?.data && <WorkbenchKernel key={attachment.case_ref} workspace={workspace.data} stream={stream} platform={platform} registry={registry} refresh={() => void refresh()} openCaseSwitcher={() => setSwitcher(true)} />}
    {attachment && workspace && workspace.result_state !== "success" && <HostFailure result={workspace} retry={() => void refresh()} />}
    {switcher && <CaseSwitcher cases={cases} dataKind={dataSource.kind} close={() => setSwitcher(false)} open={(id) => { setSwitcher(false); void loadCase(id); }} />}
  </div>;
}

function ApplicationFrame({ platform, children }: { platform: PlatformServices; children: React.ReactNode }) { return <div className="live-studio"><StartChrome platform={platform} />{children}</div>; }
function StartChrome({ platform }: { platform: PlatformServices }) { return <nav className="native-menu" aria-label="Application menu" data-tauri-drag-region onPointerDown={beginDesktopWindowDrag} onDoubleClick={toggleDesktopWindowMaximize}><strong>YAI</strong><span className="native-menu-drag" data-tauri-drag-region>YAI Studio</span><DesktopWindowControls />{platform.host.capabilities.nativeDesktop && <span className="sr-only">Native desktop host</span>}</nav>; }
function StartCenter({ result, cases, dataKind, open, retry, newCase }: { result?: OperationResult<unknown>; cases: LiveCaseRow[]; dataKind: "live" | "fixture"; open: (id: string) => void; retry: () => void; newCase?: () => void }) {
  const [query, setQuery] = useState(""); const visible = cases.filter((item) => `${item.case_ref} ${item.display_name}`.toLowerCase().includes(query.toLowerCase()));
  return <main className="live-start"><section className="live-start-intro"><span>{dataKind === "live" ? "Local Case Workbench" : "Fixture development Workbench"}</span><h1>Open a Case.</h1><p>{dataKind === "live" ? "Studio attaches to durable Case continuity owned by YAI on this machine." : "Explicit authored Case data runs through the same Workbench as live data."}</p><SearchInput autoFocus aria-label="Search Cases" placeholder={dataKind === "live" ? "Search real local Cases" : "Search fixture Cases"} value={query} onChange={(event) => setQuery(event.target.value)} />{newCase && <Button onClick={newCase}>Compose fixture Case</Button>}</section>{result?.result_state && result.result_state !== "success" ? <HostFailure result={result} retry={retry} /> : <section className="live-case-list" aria-label={dataKind === "live" ? "Real local Cases" : "Fixture Cases"}><PanelHeader title={dataKind === "live" ? "Local Cases" : "Fixture Cases"} detail={`${visible.length} available`} />{visible.map((item) => <button className="live-case-row" key={item.case_ref} onClick={() => open(item.case_ref)}><Icon name="case" size={18} /><span><strong>{item.display_name}</strong><small>{item.case_status} · generation {item.generation}</small></span><span className="case-facts">{item.source_count} sources · {item.participant_count} participants</span><Icon name="chevron" size={14} /></button>)}{!visible.length && <EmptyState title="No Cases" body={dataKind === "live" ? "YAI returned an empty authorized Case list. Studio has not substituted sample data." : "No authored fixture matches the query."} />}</section>}<footer><Badge tone={dataKind === "live" ? "success" : "warning"}>{dataKind === "live" ? "Real local data" : "Fixture Case data"}</Badge><span>{dataKind === "live" ? "No fixture fallback." : "Explicit development mode."}</span></footer></main>;
}
function HostFailure({ result, retry }: { result: OperationResult<unknown>; retry: () => void }) { return <main className="host-failure"><Icon name="warning" size={28} /><h1>Local YAI unavailable</h1><p>{result.error?.safe_message ?? "YAI did not return this projection."}</p><code>{result.result_state} · {result.error?.code}</code><Button onClick={retry}>Retry</Button><small>No fixture data has been loaded.</small></main>; }
function CaseSwitcher({ cases, dataKind, close, open }: { cases: LiveCaseRow[]; dataKind: "live" | "fixture"; close: () => void; open: (id: string) => void }) { const [query, setQuery] = useState(""); return <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) close(); }}><section className="case-switcher" role="dialog" aria-modal="true" aria-label="Open Case"><PanelHeader title="Open Case" detail={dataKind === "live" ? "Real local YAI" : "Explicit fixture data"} actions={<IconButton aria-label="Close" onClick={close}><Icon name="close" /></IconButton>} /><SearchInput autoFocus placeholder="Case ID" value={query} onChange={(event) => setQuery(event.target.value)} />{cases.filter((item) => item.case_ref.includes(query)).map((item) => <button className="ui-list-row" key={item.case_ref} onClick={() => open(item.case_ref)}><Icon name="case" /><span><strong>{item.display_name}</strong><small>{item.case_ref} · generation {item.generation}</small></span></button>)}</section></div>; }
