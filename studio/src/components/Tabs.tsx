import type { KeyboardEvent } from "react";
import { Icon } from "./Icon";

export function Tabs({
  id,
  label,
  items,
  active,
  onSelect,
  onClose,
}: {
  id: string;
  label: string;
  items: readonly { id: string; label: string; changed?: boolean }[];
  active: string;
  onSelect: (id: string) => void;
  onClose?: (id: string) => void;
}) {
  function navigate(event: KeyboardEvent<HTMLButtonElement>, index: number) {
    let next: number;
    if (event.key === "ArrowRight") next = (index + 1) % items.length;
    else if (event.key === "ArrowLeft")
      next = (index + items.length - 1) % items.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = items.length - 1;
    else return;
    event.preventDefault();
    onSelect(items[next].id);
    document.getElementById(`${id}-${items[next].id}`)?.focus();
  }
  return (
    <div className="tabs" role="tablist" aria-label={label}>
      {items.map((item, index) => (
        <div
          className={`tab-wrap ${active === item.id ? "active" : ""}`}
          key={item.id}
          role="presentation"
        >
          <button
            id={`${id}-${item.id}`}
            role="tab"
            aria-selected={active === item.id}
            aria-controls={`${id}-panel`}
            tabIndex={active === item.id ? 0 : -1}
            onKeyDown={(event) => navigate(event, index)}
            onClick={() => onSelect(item.id)}
          >
            {item.label}
            {item.changed && (
              <span className="changed-mark" aria-label="Fixture change">
                ●
              </span>
            )}
          </button>
          {onClose && (
            <button
              className="tab-close"
              aria-label={`Close ${item.label}`}
              tabIndex={active === item.id ? 0 : -1}
              onClick={() => onClose(item.id)}
            >
              <Icon name="close" size={12} />
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
