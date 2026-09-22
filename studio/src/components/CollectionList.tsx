import { useState } from "react";
import { Icon, type IconName } from "./Icon";
import { EmptyState, SearchInput, Button } from "./primitives";

export interface CollectionItem {
  id: string;
  title: string;
  detail?: string;
  icon?: IconName;
  meta?: string;
  searchText?: string;
}

/** Bounded rendering, full-set search. Counts always describe the supplied projection. */
export function CollectionList({ label, items, selected, onSelect, empty = "No items in this projection", pageSize = 40 }: {
  label: string; items: readonly CollectionItem[]; selected?: string;
  onSelect?: (id: string) => void; empty?: string; pageSize?: number;
}) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  const filtered = items.filter(item => terms.every(term => `${item.title} ${item.detail ?? ""} ${item.meta ?? ""} ${item.searchText ?? ""}`.toLocaleLowerCase().includes(term)));
  const pages = Math.max(1, Math.ceil(filtered.length / pageSize));
  const current = Math.min(page, pages - 1);
  const visible = filtered.slice(current * pageSize, (current + 1) * pageSize);
  const Row = onSelect ? "button" : "div";
  return <section className="collection-list" aria-label={label}>
    <div className="collection-tools"><SearchInput aria-label={`Search ${label}`} placeholder={`Search ${label.toLowerCase()}`} value={query} onChange={event => { setQuery(event.target.value); setPage(0); }} /><span role="status">{filtered.length} of {items.length}</span></div>
    <div className="collection-items">{visible.map(item => <Row className={`collection-row ${onSelect ? "interactive" : "readonly"} ${selected === item.id ? "selected" : ""}`} key={item.id} onClick={onSelect ? () => onSelect(item.id) : undefined} aria-pressed={onSelect ? selected === item.id : undefined} tabIndex={onSelect ? 0 : -1}>
      <Icon name={item.icon ?? "file"} /><span><strong>{item.title}</strong>{item.detail && <small>{item.detail}</small>}</span>{item.meta && <em>{item.meta}</em>}
    </Row>)}</div>
    {!visible.length && <EmptyState title={query ? "No matches" : empty} body={query ? "Try another word or clear the search." : "Only qualified Case facts are shown here."} />}
    {pages > 1 && <footer className="collection-pagination"><span>{current * pageSize + 1}–{Math.min((current + 1) * pageSize, filtered.length)} of {filtered.length}</span><Button disabled={current === 0} onClick={() => setPage(current - 1)}>Previous</Button><span>Page {current + 1} / {pages}</span><Button disabled={current + 1 >= pages} onClick={() => setPage(current + 1)}>Next</Button></footer>}
  </section>;
}
