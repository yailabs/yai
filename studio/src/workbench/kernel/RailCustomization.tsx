import { useEffect, useState } from "react";
import type { ConfigurationService } from "../../platform/configuration";
import { useModalFocus } from "../../components/useModalFocus";
import { Icon } from "../../components/Icon";
import type { WorkbenchRegistry } from "./registry";

export function useRailPreference(configuration: ConfigurationService) {
  const read = () => ({ hidden: configuration.get<string[]>("workbench.rail.hidden") ?? [], order: configuration.get<string[]>("workbench.rail.order") ?? [] });
  const [preference, setPreference] = useState(read);
  useEffect(() => configuration.subscribe(key => {
    if (key.startsWith("workbench.rail.")) setPreference(read());
  }).dispose, [configuration]);
  return preference;
}

export function RailCustomization({ registry, configuration, close }: {
  registry: WorkbenchRegistry; configuration: ConfigurationService; close(): void;
}) {
  const root = useModalFocus(close);
  const preference = useRailPreference(configuration);
  const all = registry.viewContainers();
  const visible = registry.railContainers(preference);
  const movable = visible.filter(item => item.rail?.fixed === false);
  const move = (id: string, delta: number) => {
    const ids = movable.map(item => item.id);
    const index = ids.indexOf(id), next = index + delta;
    if (index < 0 || next < 0 || next >= ids.length || movable[index].rail?.section !== movable[next].rail?.section) return;
    [ids[index], ids[next]] = [ids[next], ids[index]];
    configuration.update("workbench.rail.order", ids);
  };
  return <div className="modal-backdrop rail-customization-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) close(); }}>
    <section ref={root} role="dialog" aria-modal="true" aria-label="Customize Activity Rail" className="rail-customization">
      <header><h2>Activity Rail</h2><button aria-label="Close rail customization" onClick={close}><Icon name="close" /></button></header>
      <p>YAI perspectives and platform views stay fixed. Eligible additional views can be pinned and reordered on this device.</p>
      {(["core", "platform", "pinned"] as const).map(section => {
        const items = all.filter(item => (item.rail?.section ?? "core") === section);
        if (!items.length) return null;
        return <section key={section}><h3>{section === "core" ? "YAI" : section === "platform" ? "First-party platforms" : "Pinned views"}</h3>
          {items.map(item => {
            const fixed = item.rail?.fixed !== false, pinned = visible.some(entry => entry.id === item.id);
            const position = movable.findIndex(entry => entry.id === item.id);
            return <div className="rail-preference-row" key={item.id}>
              <Icon name={item.icon} /><span>{item.title}</span>
              {fixed ? <small>Fixed</small> : <>
                <button aria-label={`${pinned ? "Unpin" : "Pin"} ${item.title}`} aria-pressed={pinned} onClick={() => {
                  configuration.update("workbench.rail.hidden", pinned ? [...preference.hidden.filter(id => id !== item.id), item.id] : preference.hidden.filter(id => id !== item.id));
                  if (!pinned && !preference.order.includes(item.id)) configuration.update("workbench.rail.order", [...preference.order, item.id]);
                }}>{pinned ? "Unpin" : "Pin"}</button>
                <button aria-label={`Move ${item.title} up`} disabled={!pinned || position <= 0 || movable[position - 1]?.rail?.section !== section} onClick={() => move(item.id, -1)}>↑</button>
                <button aria-label={`Move ${item.title} down`} disabled={!pinned || movable[position + 1]?.rail?.section !== section} onClick={() => move(item.id, 1)}>↓</button>
              </>}
            </div>;
          })}</section>;
      })}
      <footer><button onClick={() => { configuration.update("workbench.rail.hidden", []); configuration.update("workbench.rail.order", []); }}>Restore rail defaults</button><button onClick={close}>Done</button></footer>
    </section>
  </div>;
}
