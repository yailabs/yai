import { useEffect, useMemo, useState } from "react";
import { Badge, EmptyState } from "../../components/primitives";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { SettingDefinition, SettingSection } from "../../workbench/settings/registry";

const sections: readonly SettingSection[] = ["General", "Appearance", "Workbench", "Terminal", "YAI Host", "Providers", "YVEX", "Security", "Advanced"];

export function SettingsSurface({ workspace, actions, platform, settings, selection }: SurfaceRendererProps) {
  const [section, setSection] = useState<SettingSection>("General");
  const [query, setQuery] = useState("");
  const [, refresh] = useState(0);
  useEffect(() => platform.configuration.subscribe(() => refresh((value) => value + 1)).dispose, [platform.configuration]);
  useEffect(() => {
    if (!selection.startsWith("settings:")) return;
    const requested = sections.find((candidate) => candidate.toLocaleLowerCase().replaceAll(" ", "-") === selection.slice("settings:".length));
    if (requested) { setQuery(""); setSection(requested); }
  }, [selection]);
  const definitions = settings.entries();
  const visible = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    return definitions.filter((setting) => (!needle ? setting.section === section : `${setting.title} ${setting.description} ${setting.section}`.toLocaleLowerCase().includes(needle)));
  }, [definitions, query, section]);
  return <div className="settings-surface" data-surface-type="studio.settings">
    <aside><h1>Settings</h1><div className="settings-search"><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search settings" aria-label="Search settings" /></div>{sections.map((item) => <button key={item} aria-pressed={!query && section === item} onClick={() => { setQuery(""); setSection(item); actions.inspect(`settings:${item.toLocaleLowerCase().replaceAll(" ", "-")}`); }}>{item}</button>)}</aside>
    <div className="settings-content"><span>{query ? "Search results" : "Studio settings"}</span><h2>{query ? `Results for “${query}”` : section}</h2><p className="settings-scope-note">Local preferences stay in Studio. Case and host settings require their qualified owners.</p>{visible.map((definition) => <SettingRow key={definition.id} definition={definition} value={platform.configuration.get(definition.id) ?? definition.defaultValue} update={(value) => platform.configuration.update(definition.id, value)} dataKind={workspace.presentation.dataKind} backend={workspace.presentation.backendPosture} />)}{!visible.length && <EmptyState title="No matching settings" body="Try a setting title, description or section." />}</div>
  </div>;
}

export function searchSettingsSurface({ settings }: Pick<SurfaceRendererProps, "settings">, _input: SurfaceRendererProps["input"], query: string) {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  return settings.entries().filter((setting) => `${setting.title} ${setting.description} ${setting.section}`.toLocaleLowerCase().includes(needle)).map((setting) => ({ id: setting.id, label: setting.title, detail: `${setting.section} · ${setting.available ? setting.scope : "unavailable"}`, objectRef: `settings:${setting.section.toLocaleLowerCase().replaceAll(" ", "-")}` }));
}

function SettingRow({ definition, value, update, dataKind, backend }: { definition: SettingDefinition; value: unknown; update(value: unknown): void; dataKind: string; backend: string }) {
  const informationalValue = definition.id === "host.currentTopology" ? `Embedded local application boundary · ${dataKind} Case data · ${backend}` : definition.unavailableReason;
  return <section className={`setting-row ${definition.available ? "available" : "unavailable"}`}>
    <div><h3>{definition.title}</h3><p>{definition.description}</p><small>{definition.scope === "local" ? "Stored locally by Studio" : `${definition.scope} owner`}</small></div>
    <div className="setting-control">{definition.control === "boolean" && definition.available && <button role="switch" aria-checked={Boolean(value)} onClick={() => update(!value)}>{value ? "On" : "Off"}</button>}{definition.control === "number" && definition.available && <input type="number" value={Number(value)} onChange={(event) => update(Number(event.target.value))} />}{definition.control === "select" && definition.available && <select value={String(value)} onChange={(event) => update(event.target.value)}>{definition.options?.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}</select>}{definition.control === "information" && <Badge tone={definition.available ? "info" : "warning"}>{informationalValue ?? "Available"}</Badge>}</div>
  </section>;
}
