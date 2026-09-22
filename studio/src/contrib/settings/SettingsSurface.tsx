import { ApplicationCapabilities } from "./ApplicationCapabilities";
import { preferenceNumberBounds } from "../../platform/configuration";
import { useApplicationAvailability } from "../case/applicationActions";
import { useEffect, useMemo, useState } from "react";
import { Badge, EmptyState } from "../../components/primitives";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { SettingDefinition, SettingSection } from "../../workbench/settings/registry";

const sections: readonly SettingSection[] = ["General", "Appearance", "Workbench", "Terminal", "YAI Host", "Providers", "YVEX", "Security", "Advanced"];

export function SettingsSurface({ workspace, actions, platform, settings, selection }: SurfaceRendererProps) {
  const application = useApplicationAvailability(platform.application);
  const [section, setSection] = useState<SettingSection>("General");
  const [query, setQuery] = useState("");
  const [, refresh] = useState(0);
  useEffect(() => platform.configuration.subscribe(() => refresh((value) => value + 1)).dispose, [platform.configuration]);
  useEffect(() => platform.host.subscribe(() => refresh((value) => value + 1)).dispose, [platform.host]);
  useEffect(() => {
    if (!selection.startsWith("settings:")) return;
    const requested = sections.find((candidate) => candidate.toLocaleLowerCase().replaceAll(" ", "-") === selection.slice("settings:".length));
    if (requested) { setQuery(""); setSection(requested); }
  }, [selection]);
  const chooseSection = (item: SettingSection) => { setQuery(""); setSection(item); actions.inspect(`settings:${item.toLocaleLowerCase().replaceAll(" ", "-")}`); };
  const definitions = settings.entries();
  const visible = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    return definitions.filter((setting) => (!needle ? setting.section === section : `${setting.title} ${setting.description} ${setting.section}`.toLocaleLowerCase().includes(needle)));
  }, [definitions, query, section]);
  return <div className="settings-surface" data-surface-type="studio.settings">
    <aside><h1>Settings</h1><div className="settings-search"><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search settings" aria-label="Search settings" /></div><nav className="settings-sections" aria-label="Settings sections">{sections.map((item) => <button key={item} aria-pressed={!query && section === item} onClick={() => chooseSection(item)}>{item}</button>)}</nav><select className="settings-compact-sections" aria-label="Settings section" value={section} onChange={event => chooseSection(event.target.value as SettingSection)}>{sections.map(item => <option key={item}>{item}</option>)}</select></aside>
    <div className="settings-content"><span>{query ? "Search results" : "Studio settings"}</span><h2>{query ? `Results for “${query}”` : section}</h2><p className="settings-scope-note">Local preferences stay in Studio. Case and host settings require their qualified owners.</p>{!query && section === "YAI Host" && <YaiHostSettings platform={platform} />}{!query && section === "Advanced" && <ApplicationCapabilities platform={platform} />}{!query && section === "Providers" && <section className="setting-row"><div><h3>Provider actions</h3><p>{application.catalog?.operations.some(item => item.operation_id.startsWith("provider.")) ? "The connected Host exposes provider governance operations. Studio configuration forms are not integrated yet." : "The connected Host does not currently advertise provider governance operations."}</p><button className="ui-button" onClick={() => actions.openPerspective("Compute")}>View current targets</button></div></section>}{(query || section !== "YAI Host") && visible.map((definition) => <SettingRow key={definition.id} definition={definition} value={platform.configuration.get(definition.id) ?? definition.defaultValue} update={(value) => platform.configuration.update(definition.id, value)} dataKind={workspace.presentation.dataKind} backend={workspace.presentation.backendPosture} />)}{(query || section !== "YAI Host") && !visible.length && <EmptyState title="No matching settings" body="Try a setting title, description or section." />}</div>
  </div>;
}

export function searchSettingsSurface({ settings }: Pick<SurfaceRendererProps, "settings">, _input: SurfaceRendererProps["input"], query: string) {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  return settings.entries().filter((setting) => `${setting.title} ${setting.description} ${setting.section}`.toLocaleLowerCase().includes(needle)).map((setting) => ({ id: setting.id, label: setting.title, detail: `${setting.section} · ${setting.available ? setting.scope : "unavailable"}`, objectRef: `settings:${setting.section.toLocaleLowerCase().replaceAll(" ", "-")}` }));
}

function SettingRow({ definition, value, update, dataKind, backend }: { definition: SettingDefinition; value: unknown; update(value: unknown): void; dataKind: string; backend: string }) {
  const informationalValue = definition.id === "host.currentTopology" ? `Resident local Host · ${dataKind} Case data · ${backend}` : definition.unavailableReason;
  return <section className={`setting-row ${definition.available ? "available" : "unavailable"}`}>
    <div><h3>{definition.title}</h3><p>{definition.description}</p><small>{definition.scope === "local" ? "Stored locally by Studio" : `${definition.scope} owner`}</small></div>
    <div className="setting-control">{definition.control === "boolean" && definition.available && <button role="switch" aria-label={definition.title} aria-checked={Boolean(value)} onClick={() => update(!value)}>{value ? "On" : "Off"}</button>}{definition.control === "number" && definition.available && <NumberSetting definition={definition} value={Number(value)} update={update} />}{definition.control === "select" && definition.available && <select aria-label={definition.title} value={String(value)} onChange={(event) => update(event.target.value)}>{definition.options?.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}</select>}{definition.control === "information" && <Badge tone={definition.available ? "info" : "warning"}>{informationalValue ?? "Available"}</Badge>}</div>
  </section>;
}

function NumberSetting({ definition, value, update }: { definition: SettingDefinition; value: number; update(value: number): void }) {
  const [draft, setDraft] = useState(String(value));
  const bounds = preferenceNumberBounds[definition.id];
  useEffect(() => setDraft(String(value)), [value]);
  const valid = draft.trim() !== "" && Number.isFinite(Number(draft)) && (!bounds || Number(draft) >= bounds.min && Number(draft) <= bounds.max && (bounds.step < 1 || Number.isInteger(Number(draft))));
  const commit = () => { if (valid) update(Number(draft)); };
  return <div className="number-setting"><input type="number" aria-label={definition.title} aria-invalid={!valid} {...bounds} value={draft} onChange={event => setDraft(event.target.value)} onBlur={commit} onKeyDown={event => { if (event.key === "Enter") commit(); if (event.key === "Escape") setDraft(String(value)); }} />{!valid && <small role="alert">Enter {bounds ? `${bounds.min}–${bounds.max}` : "a finite number"}.</small>}</div>;
}

function YaiHostSettings({ platform }: Pick<SurfaceRendererProps, "platform">) {
  const host = platform.host.snapshot();
  const telemetry = host.telemetry;
  const facts = [
    ["Status", host.state],
    ["PID", telemetry?.pid ?? "—"],
    ["Uptime", telemetry ? `${Math.floor(telemetry.uptime_ms / 1000)}s` : "—"],
    ["Version", telemetry?.version ?? "—"],
    ["Build", telemetry?.build ?? "—"],
    ["YAI Home", telemetry?.yai_home ?? "—"],
    ["Protocol", telemetry?.protocol ?? "—"],
    ["Transport", telemetry?.transport ?? "local IPC"],
    ["Application", telemetry?.application_readiness ?? "unavailable"],
    ["Studio clients", telemetry?.client_kinds.studio ?? 0],
    ["CLI clients", telemetry?.client_kinds.cli ?? 0],
    ["Runtime supervision", telemetry?.runtime_supervision === "not_integrated" ? "Not yet integrated" : telemetry?.runtime_supervision ?? "Not yet integrated"],
  ] as const;
  const act = async (action: "start" | "stop" | "restart") => {
    if (action === "stop" && !window.confirm("Stop the resident YAI Host? Open Studio clients will become unavailable; durable Cases are unchanged.")) return;
    if (action === "restart" && !window.confirm("Restart the resident YAI Host? Clients will reconnect and resynchronize their Cases.")) return;
    try { await platform.host[action](); } catch { await platform.host.status(); }
  };
  return <section className="yai-host-settings" aria-label="YAI Host telemetry">
    <div className="host-setting-header"><div><h3>Current host</h3><p>One resident application service for this YAI_HOME. Closing Studio does not stop it.</p></div><Badge tone={host.state === "live" ? "success" : host.state === "unavailable" ? "error" : "warning"}>{host.state}</Badge></div>
    <dl>{facts.map(([name, value]) => <div key={name}><dt>{name}</dt><dd>{value}</dd></div>)}</dl>
    {host.reason && <p className="host-setting-error">{host.reason}</p>}
    <div className="host-setting-actions">{host.state === "stopped" || host.state === "unavailable" ? <button onClick={() => void act("start")}>Start YAI</button> : <><button onClick={() => void act("restart")}>Restart YAI</button><button className="danger" onClick={() => void act("stop")}>Stop YAI</button></>}</div>
    <small>RuntimeInstance remains a separate bounded scheduler in this milestone.</small>
  </section>;
}
