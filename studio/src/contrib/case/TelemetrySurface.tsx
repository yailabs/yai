import { useEffect, useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { Badge, Button, EmptyState } from "../../components/primitives";
import { ExecutionHistory } from "./ExecutionActions";
import { resourceInput } from "../surfaces/inputs";
import "./TelemetrySurface.css";

/** Existing Host and Case observations; never an OS process scanner. */
export function TelemetrySurface({ workspace, platform, actions }: SurfaceRendererProps) {
  const [connection, setConnection] = useState(platform.host.snapshot());
  const [sampled, setSampled] = useState<number>();
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string>();
  useEffect(() => platform.host.subscribe(setConnection).dispose, [platform.host]);
  const host = connection.state === "live" ? connection.telemetry : undefined;
  const date = (value?: number) => value ? new Date(value).toLocaleString() : "Not observed";
  const refresh = async () => {
    setBusy(true); setFailure(undefined);
    try {
      if (await platform.host.status()) setSampled(Date.now());
      await platform.commands.executeCommand("studio.case.refresh");
    } catch (error) { setFailure(error instanceof Error ? error.message : "Observation unavailable"); }
    finally { setBusy(false); }
  };
  return <article className="live-page telemetry-surface">
    <header className="operational-heading"><div><small>Operational observations</small><h1>Telemetry</h1><p>Host processes, attached clients and this Case’s infrastructure.</p></div><Button disabled={busy} onClick={() => void refresh()}>{busy ? "Refreshing…" : "Refresh observations"}</Button></header>
    <p className="surface-note">{sampled ? `Last requested observation: ${date(sampled)}.` : "Host connection observations update independently of Case snapshots."} Opening this view starts no processes or endpoint probes.</p>
    {failure && <p role="alert">{failure}</p>}
    <nav className="telemetry-nav" aria-label="Telemetry sections">{["Host", "Clients", "Resources", "Endpoints", "Executions"].map(name => <a key={name} href={`#telemetry-${name.toLowerCase()}`}>{name}</a>)}</nav>
    <section id="telemetry-host"><header><h2>YAI Host</h2><Badge tone={host ? "success" : "warning"}>{connection.state}</Badge></header>
      <dl>{[["PID", host?.pid], ["Uptime at observation", host ? `${Math.floor(host.uptime_ms / 1000)} s` : undefined], ["Application", host?.application_readiness], ["Runtime supervision", host?.runtime_supervision], ["Local endpoint", host?.endpoint], ["Last Host activity", date(host?.last_activity_unix_ms)]].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value ?? "Not observed"}</dd></div>)}</dl>
      {!host && <p className="surface-note">{connection.reason ?? "No current Host observation. Cached process facts are withheld."}</p>}
      <details><summary>Host identity</summary><dl>{[["Instance", host?.instance_id], ["Process identity", host?.process_identity], ["Build", host?.build], ["Protocol", host?.protocol]].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value ?? "Not observed"}</dd></div>)}</dl></details>
      <Button onClick={() => actions.openSettings("yai-host")}>Host controls</Button>
    </section>
    <section id="telemetry-clients"><header><h2>Connected clients</h2><span>{host?.connected_clients ?? "Unknown"}</span></header><p className="surface-note">Processes attached to this Host, not every process on the machine.</p>
      {host?.clients.map(client => <div className="telemetry-row" key={client.client_id}><strong>{client.client_kind}</strong><span>PID {client.pid}</span><span>Last seen {date(client.last_seen_unix_ms)}</span></div>)}
      {host && !host.clients.length && <p>No clients in the current observation.</p>}
    </section>
    <section id="telemetry-resources"><header><h2>Case Resources</h2><span>{workspace.environment.resources.length}</span></header><p className="surface-note">Database, HTTP and process bindings describe requestable infrastructure. Live reachability and process residency are not projected by this inventory.</p>
      {workspace.environment.resources.map(resource => <button className="telemetry-row" key={resource.id} onClick={() => actions.openSurface(resourceInput(workspace, resource.id, true))}><span><strong>{resource.label ?? resource.id}</strong><small>{resource.kind.replaceAll("_", " ")}</small></span><span>{resource.operations.join(" · ") || "No operations projected"}</span><Badge>Availability unknown</Badge></button>)}
      {!workspace.environment.resources.length && <EmptyState title="No Resources attached" body="Attach qualified infrastructure through Environment."/>}
    </section>
    <section id="telemetry-endpoints"><header><h2>Bound model endpoints</h2><span>{workspace.compute.targets.length}</span></header>
      <p className="surface-note">Retained provider observations include their observation time. They are not a new reachability probe.</p>
      {workspace.compute.targets.map(target => {
        const health = typeof target.posture === "object" ? target.posture?.health : undefined;
        return <div key={target.id} className="telemetry-target"><button className="telemetry-row" onClick={() => actions.inspect(target.id)}><span><strong>{target.model_id}</strong><small>{target.provider_key} · {target.locality}</small></span><span>{target.endpoint || "Endpoint not disclosed"}</span><Badge>{health?.observed_at_unix_ms ? `Observed: ${health.posture}` : "Live health unknown"}</Badge></button>
          {health && <dl aria-label={`Provider observation for ${target.model_id}`}><div><dt>Observed at</dt><dd>{date(health.observed_at_unix_ms)}</dd></div><div><dt>Circuit</dt><dd>{health.circuit}</dd></div><div><dt>Consecutive failures</dt><dd>{health.consecutive_failures}</dd></div>{health.failure_class && <div><dt>Last failure class</dt><dd>{health.failure_class}</dd></div>}</dl>}
        </div>;
      })}
      {!workspace.compute.targets.length && <EmptyState title="No model target bound" body="Configure inventory in Providers, then bind a target in Compute."/>}
      <Button onClick={() => actions.openPerspective("Compute")}>Open Compute</Button>
    </section>
    <section id="telemetry-executions"><h2>Executions</h2><p className="surface-note">Exact submitted operations and retained outcomes. A recorded effect is not proof that a process is still running.</p><ExecutionHistory workspace={workspace} platform={platform}/></section>
  </article>;
}

export function TelemetryNavigation() {
  return <div className="live-sidebar-content"><section className="sidebar-group"><h2>Observations</h2>{["Host", "Clients", "Resources", "Endpoints", "Executions"].map(name => <a className="telemetry-sidebar-link" key={name} href={`#telemetry-${name.toLowerCase()}`}>{name}</a>)}</section><p className="surface-note">Host-wide processes and Case-scoped bindings are separate observations. Refresh explicitly to request current facts.</p></div>;
}
