import { useCallback, useEffect, useState, useSyncExternalStore } from "react";
import type { DesktopTerminalSnapshot } from "../../platform/host";
import type { SidebarViewProps, SurfaceRendererProps } from "../../workbench/kernel/types";
import { Badge, Button, EmptyState } from "../../components/primitives";
import { ExecutionHistory } from "./ExecutionActions";
import { resourceInput } from "../surfaces/inputs";
import "./TelemetrySurface.css";

const sections = ["Host", "Runtime", "Shells", "Clients", "Resources", "Endpoints", "Executions"] as const;
type Section = typeof sections[number];
function useSection(platform: SurfaceRendererProps["platform"]) {
  const subscribe = useCallback((listener: () => void) => platform.context.subscribe(listener).dispose, [platform]);
  const read = () => {
    const value = platform.context.get("studio.telemetry.section") as Section;
    return sections.includes(value) ? value : "Host";
  };
  return useSyncExternalStore(subscribe, read);
}

/** Existing Host and Case observations; never an OS process scanner. */
export function TelemetrySurface({ workspace, platform, actions }: SurfaceRendererProps) {
  const section = useSection(platform);
  const select = (name: Section) => platform.context.update("studio.telemetry.section", name);
  const panel = (name: Section) => ({ role: "tabpanel" as const, hidden: section !== name, tabIndex: 0, "aria-labelledby": `telemetry-tab-${name.toLowerCase()}` });
  const [connection, setConnection] = useState(platform.host.snapshot());
  const [shells, setShells] = useState<DesktopTerminalSnapshot>();
  const [shellFailure, setShellFailure] = useState<string>();
  useEffect(() => {
    let alive = true;
    let pending = false;
    const observe = async () => {
      if (pending || document.visibilityState === "hidden") return;
      pending = true;
      try {
        const snapshot = await platform.host.terminalSnapshot?.();
        if (alive) { setShells(snapshot); setShellFailure(undefined); }
      } catch {
        if (alive) { setShells(undefined); setShellFailure("Desktop shell observation unavailable."); }
      } finally { pending = false; }
    };
    void observe();
    const interval = window.setInterval(() => void observe(), 2000);
    return () => { alive = false; window.clearInterval(interval); };
  }, [platform.host]);
  const [sampled, setSampled] = useState<number>();
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string>();
  useEffect(() => platform.host.subscribe(setConnection).dispose, [platform.host]);
  const host = connection.state === "live" ? connection.telemetry : undefined;
  const runtime = host?.runtime_observation;
  const date = (value?: number) => value ? new Date(value).toLocaleString() : "Not observed";
  const refresh = async () => {
    setBusy(true); setFailure(undefined);
    try {
      if (await platform.host.status()) setSampled(Date.now());
      await platform.commands.executeCommand("studio.case.refresh");
    } catch (error) { setFailure(error instanceof Error ? error.message : "Observation unavailable"); }
    finally { setBusy(false); }
  };
  return <article className="telemetry-surface">
    <header className="operational-heading"><div><h1>Telemetry</h1><span>Host &amp; current Case</span></div><Button aria-label="Refresh observations" title="Refresh Host and Case observations" disabled={busy} onClick={() => void refresh()}>{busy ? "Refreshing…" : "Refresh"}</Button></header>
    <p className="surface-note">Host: {connection.state}{host ? ` · ${host.connected_clients} attached clients` : ""}{sampled ? ` · Refresh requested ${date(sampled)}` : ""}</p>
    {failure && <p role="alert">{failure}</p>}
    <nav className="telemetry-nav" role="tablist" aria-label="Telemetry sections" onKeyDown={event => {
      const index = sections.indexOf(section);
      const next = event.key === "ArrowRight" ? (index + 1) % sections.length : event.key === "ArrowLeft" ? (index + sections.length - 1) % sections.length : event.key === "Home" ? 0 : event.key === "End" ? sections.length - 1 : undefined;
      if (next === undefined) return;
      event.preventDefault(); select(sections[next]);
      const tab = event.currentTarget.querySelector<HTMLButtonElement>(`#telemetry-tab-${sections[next].toLowerCase()}`);
      tab?.focus({ preventScroll: true });
      tab?.scrollIntoView({ block: "nearest", inline: "nearest" });
    }}>{sections.map(name => <button key={name} id={`telemetry-tab-${name.toLowerCase()}`} role="tab" aria-selected={section === name} aria-controls={`telemetry-${name.toLowerCase()}`} tabIndex={section === name ? 0 : -1} onClick={() => select(name)}>{name}</button>)}</nav>
    <div className="telemetry-content">
    <section id="telemetry-host" {...panel("Host")}><header><h2>YAI Host</h2><Badge tone={host?.executable_posture === "replaced_on_disk" ? "warning" : host ? "success" : "warning"}>{host?.executable_posture === "replaced_on_disk" ? "Restart needed" : connection.state}</Badge></header>
      {host?.executable_posture === "replaced_on_disk" && <p role="status">The running Host executable was replaced on disk. Finish active work, then restart YAI in Settings to use the current binary.</p>}
      <dl>{[["PID", host?.pid], ["Uptime at observation", host ? `${Math.floor(host.uptime_ms / 1000)} s` : undefined], ["Application", host?.application_readiness], ["Runtime supervision", host?.runtime_supervision], ["Local endpoint", host?.endpoint], ["Last Host activity", date(host?.last_activity_unix_ms)]].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value ?? "Not observed"}</dd></div>)}</dl>
      {!host && <p className="surface-note">{connection.reason ?? "No current Host observation. Cached process facts are withheld."}</p>}
      <details><summary>Host identity</summary><dl>{[["Instance", host?.instance_id], ["Process identity", host?.process_identity], ["Build", host?.build], ["Executable link", host?.executable_posture], ["Protocol", host?.protocol]].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value ?? "Not observed"}</dd></div>)}</dl></details>
      <Button onClick={() => actions.openSettings("yai-host")}>Host controls</Button>
    </section>
    <section id="telemetry-runtime" {...panel("Runtime")}><header><h2>Runtime scheduler</h2><Badge>{runtime?.lifecycle ?? "Not observed"}</Badge></header>
      <p className="surface-note">{host?.runtime_supervision === "attached_existing_runtime" ? "Existing independently owned scheduler. Worker activity is not observed by this Host." : "Scheduler observations from the existing runtime owner. Worker counts are not OS process counts."}</p>
      {runtime ? <><dl>{[["PID", runtime.pid], ["Worker capacity", runtime.worker_capacity], ["Active workers at observation", runtime.active_workers ?? "Not observed"], ["Available workers at observation", runtime.available_workers ?? "Not observed"], ["Observed at", date(runtime.observed_at_unix_ms)], ["Heartbeat", date(runtime.heartbeat_at_unix_ms)]].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl><details><summary>Scheduler identity</summary><code>{runtime.instance_id}</code><p>{runtime.process_identity}</p></details></> : <p>No scheduler snapshot in the current Host observation. Older Hosts may expose supervision posture only.</p>}
    </section>
    <section id="telemetry-shells" {...panel("Shells")}><header><h2>Desktop shells</h2><span>{shells?.terminals.length ?? "Not observed"}</span></header>
      <p className="surface-note">Shell sessions owned by this Studio window.</p>
      {shells ? <><dl><div><dt>Studio PID</dt><dd>{shells.studio_pid}</dd></div><div><dt>Observed at</dt><dd>{date(shells.observed_at_unix_ms)}</dd></div></dl>
        {shells.terminals.map(terminal => <div className="telemetry-row telemetry-shell" key={terminal.terminal_id} data-terminal-id={terminal.terminal_id}><strong>{terminal.shell}</strong><span>PID {terminal.pid ?? "Not observed"}</span><span>Created {date(terminal.created_at_unix_ms)}</span></div>)}
        {!shells.terminals.length && <p>No open shell sessions in this Studio process.</p>}
      </> : <p>{shellFailure ?? "Desktop shell observations are available in native Studio."}</p>}
    </section>
    <section id="telemetry-clients" {...panel("Clients")}><header><h2>Connected clients</h2><span>{host?.connected_clients ?? "Unknown"}</span></header><p className="surface-note">Processes attached to this Host, not every process on the machine.</p>
      {host?.clients.map(client => <div className="telemetry-row" key={client.client_id}><strong>{client.client_kind}</strong><span>PID {client.pid}</span><span>Last seen {date(client.last_seen_unix_ms)}</span></div>)}
      {host && !host.clients.length && <p>No clients in the current observation.</p>}
    </section>
    <section id="telemetry-resources" {...panel("Resources")}><header><h2>Case Resources</h2><span>{workspace.environment.resources.length}</span></header><p className="surface-note">Database, HTTP and process bindings describe requestable infrastructure. Live reachability and process residency are not projected by this inventory.</p>
      {workspace.environment.resources.map(resource => <button className="telemetry-row" key={resource.id} onClick={() => actions.openSurface(resourceInput(workspace, resource.id, true))}><span><strong>{resource.label ?? resource.id}</strong><small>{resource.kind.replaceAll("_", " ")}</small></span><span>{resource.operations.join(" · ") || "No operations projected"}</span><Badge>Availability unknown</Badge></button>)}
      {!workspace.environment.resources.length && <EmptyState title="No Resources attached" body="Attach qualified infrastructure through Environment."/>}
    </section>
    <section id="telemetry-endpoints" {...panel("Endpoints")}><header><h2>Bound model endpoints</h2><span>{workspace.compute.targets.length}</span></header>
      <p className="surface-note">Retained provider observations include their observation time. They are not a new reachability probe.</p>
      {workspace.compute.targets.map(target => {
        const health = typeof target.posture === "object" ? target.posture?.health : undefined;
        return <div key={target.id} className="telemetry-target"><button className="telemetry-row" onClick={() => actions.inspect(target.id)}><span><strong>{target.model_id}</strong><small>{target.provider_key} · {target.locality}</small></span><span>{target.endpoint || "Endpoint not disclosed"}</span><Badge>{health?.effective_posture ? `YAI posture: ${health.effective_posture}` : "YAI posture not projected"}</Badge></button>
          {health && <dl aria-label={`Provider observation for ${target.model_id}`}><div><dt>Last report</dt><dd>{health.posture} · {date(health.observed_at_unix_ms)}</dd></div><div><dt>Evaluated by YAI</dt><dd>{date(health.evaluated_at_unix_ms ?? undefined)}</dd></div><div><dt>Circuit</dt><dd>{health.circuit}</dd></div><div><dt>Consecutive failures</dt><dd>{health.consecutive_failures}</dd></div>{health.failure_class && <div><dt>Last failure class</dt><dd>{health.failure_class}</dd></div>}</dl>}
        </div>;
      })}
      {!workspace.compute.targets.length && <EmptyState title="No model target bound" body="Configure inventory in Providers, then bind a target in Compute."/>}
      <Button onClick={() => actions.openPerspective("Compute")}>Open Compute</Button>
    </section>
    <section id="telemetry-executions" {...panel("Executions")}><h2>Executions</h2><p className="surface-note">Exact submitted operations and retained outcomes. A recorded effect is not proof that a process is still running.</p>{section === "Executions" && <ExecutionHistory workspace={workspace} platform={platform}/>}</section>
    </div>
  </article>;
}

export function TelemetryNavigation({ platform, actions }: SidebarViewProps) {
  const section = useSection(platform);
  return <div className="live-sidebar-content"><section className="sidebar-group"><h2>Observations</h2>{sections.map(name => <button className="telemetry-sidebar-link" data-section={name.toLowerCase()} aria-current={section === name ? "page" : undefined} key={name} onClick={() => { platform.context.update("studio.telemetry.section", name); actions.openPerspective("Telemetry"); }}>{name}</button>)}</section><p className="surface-note">Scope: this Host and current Case.</p></div>;
}
