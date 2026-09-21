import { Icon } from "../../components/Icon";
import { useModalFocus } from "../../components/useModalFocus";

export function OpenWith({ title, choices, selected, choose, close }: {
  title: string;
  choices: readonly { type: string; title: string }[];
  selected: string;
  choose(type: string): void;
  close(): void;
}) {
  const root = useModalFocus(close);
  return <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) close(); }}>
    <section ref={root} className="open-with" role="dialog" aria-modal="true" aria-label={`Open ${title} with`}>
      <h2>Open With…</h2>
      {choices.map((choice) => <button key={choice.type} aria-pressed={selected === choice.type} onClick={() => { choose(choice.type); close(); }}>
        <Icon name="file" /><span><strong>{choice.title}</strong>{selected === choice.type && <small>Current view</small>}</span>
      </button>)}
    </section>
  </div>;
}
