import { useModalFocus } from "../../components/useModalFocus";
import { useEffect, useMemo, useRef, useState } from "react";
import { Icon, type IconName } from "../../components/Icon";

export interface WorkbenchSearchItem {
  id: string;
  label: string;
  detail?: string;
  category: string;
  icon: IconName;
  disabled?: boolean;
  run(): void;
}

export function fuzzyScore(query: string, text: string) {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return 1;
  const haystack = text.toLocaleLowerCase();
  const direct = haystack.indexOf(needle);
  if (direct >= 0) return 1000 - direct - (haystack.length - needle.length) * .01;
  let cursor = 0;
  let score = 0;
  for (const character of needle) {
    const found = haystack.indexOf(character, cursor);
    if (found < 0) return 0;
    score += found === cursor ? 12 : Math.max(1, 8 - (found - cursor));
    cursor = found + 1;
  }
  return score;
}

export function WorkbenchSearch({ title, placeholder, items, resolveItems, onClose }: { title: string; placeholder: string; items?: readonly WorkbenchSearchItem[]; resolveItems?: (query: string) => readonly WorkbenchSearchItem[] | Promise<readonly WorkbenchSearchItem[]>; onClose(): void }) {
  const root = useModalFocus(onClose);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string>();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const [resolvedItems, setResolvedItems] = useState<readonly WorkbenchSearchItem[]>([]);
  useEffect(() => {
    if (!resolveItems) return;
    let current = true;
    setPending(true); setError(undefined);
    const timer = window.setTimeout(() => {
      void Promise.resolve().then(() => resolveItems(query)).then((value) => { if (current) setResolvedItems(value); })
        .catch((reason) => { if (current) { setError(String(reason)); setResolvedItems([]); } })
        .finally(() => { if (current) setPending(false); });
    }, 120);
    return () => { current = false; window.clearTimeout(timer); };
  }, [query, resolveItems]);
  const sourceItems = resolveItems ? resolvedItems : (items ?? []);
  const filtered = useMemo(() => sourceItems
    .map((item) => ({ item, score: fuzzyScore(query, `${item.label} ${item.detail ?? ""} ${item.category}`) }))
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score || a.item.label.localeCompare(b.item.label))
    .slice(0, 80).map(({ item }) => item), [query, sourceItems]);
  useEffect(() => { input.current?.focus(); }, []);
  useEffect(() => setActive(0), [query]);
  useEffect(() => { root.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" }); }, [active, root]);
  const run = (item: WorkbenchSearchItem | undefined) => { if (!item || item.disabled) return; item.run(); onClose(); };
  return <div className="search-scrim" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
    <section ref={root} className="workbench-search" role="dialog" aria-modal="true" aria-label={title} onKeyDown={(event) => {
      if (event.key === "Escape") { event.preventDefault(); onClose(); }
      if (event.key === "ArrowDown") { event.preventDefault(); setActive((value) => Math.max(0, Math.min(filtered.length - 1, value + 1))); }
      if (event.key === "ArrowUp") { event.preventDefault(); setActive((value) => Math.max(0, value - 1)); }
      if (event.key === "Enter") { event.preventDefault(); run(filtered[active]); }
    }}>
      <header><span>{title}</span><kbd>Esc</kbd></header>
      <div className="workbench-search-input"><Icon name="search" size={16} /><input ref={input} value={query} onChange={(event) => setQuery(event.target.value)} placeholder={placeholder} aria-label={placeholder} /></div>
      <div className="workbench-search-results" role="listbox" aria-label={`${title} results`}>{filtered.map((item, index) => <button key={item.id} role="option" aria-selected={index === active} disabled={item.disabled} onMouseMove={() => setActive(index)} onClick={() => run(item)}><Icon name={item.icon} size={15} /><span><strong>{item.label}</strong><small>{item.category}{item.detail ? ` · ${item.detail}` : ""}</small></span>{item.disabled && <em>Unavailable</em>}</button>)}{error ? <p role="alert">{error}</p> : pending ? <p role="status">Searching…</p> : !filtered.length && <p>No matching results.</p>}</div>
    </section>
  </div>;
}
