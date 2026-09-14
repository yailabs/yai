import type {
  ContextMode,
  WorkspacePresentation,
} from "../clients/presentation";
import { Icon } from "../components/Icon";
import { Tabs } from "../components/Tabs";

const modes: readonly ContextMode[] = ["Conversation", "Inspector", "Activity"];

export function ContextPanel({
  data,
  mode,
  setMode,
  selected,
  step,
  open,
  collapse,
  draft,
  setDraft,
}: {
  data: WorkspacePresentation;
  mode: ContextMode;
  setMode: (mode: ContextMode) => void;
  selected: string;
  step: number;
  open: (id: string) => void;
  collapse: () => void;
  draft: string;
  setDraft: (text: string) => void;
}) {
  const material = data.materials.find((item) => item.id === selected);
  const inspector =
    data.information.inspector[selected] ??
    (material
      ? {
          eyebrow: material.category.toUpperCase(),
          title: material.name,
          description: material.provenance,
          facts: [
            { label: "Format", value: material.format },
            { label: "Path", value: material.path },
            { label: "Mode", value: "Fixture · read only" },
          ],
        }
      : {
          eyebrow: "CASE CONTEXT",
          title: data.case.label,
          description: data.case.purpose,
          facts: [
            {
              label: "Participant",
              value:
                data.participants.find(
                  (person) => person.id === data.case.participant,
                )?.name ?? "Unknown",
            },
            { label: "Current work", value: data.case.currentWork },
            { label: "Runtime", value: "Not attached" },
          ],
        });
  const events = data.information.memory.timeline
    .filter((event) => event.step <= step)
    .slice()
    .reverse();
  return (
    <aside
      className="context-panel"
      id="case-conversation"
      aria-label="Case context panel"
    >
      <header className="context-header">
        <Tabs
          id="context"
          label="Context panel modes"
          items={modes.map((id) => ({ id, label: id }))}
          active={mode}
          onSelect={(id) => setMode(id as ContextMode)}
        />
        <button
          className="icon-button"
          onClick={collapse}
          aria-label="Collapse context panel"
          title="Collapse context panel"
        >
          <Icon name="right" size={16} />
        </button>
      </header>
      {mode === "Conversation" && (
        <>
          <div className="thread-context">
            <span className="eyebrow">CONVERSATION IN THIS CASE</span>
            <h2>{data.case.currentWork}</h2>
            <div className="thread-people">
              <Icon name="people" size={14} />
              <span>{data.participants.length} participants</span>
              <span>Thread 01 · fixture</span>
            </div>
          </div>
          <div
            className="conversation-scroll"
            tabIndex={0}
            aria-label="Conversation history"
          >
            {data.activity.map((entry) => {
              if (entry.kind === "turn") {
                const person = data.participants.find(
                  (candidate) => candidate.id === entry.participant,
                )!;
                return (
                  <article
                    className={`turn ${person.kind === "AI participant" ? "ai-turn" : ""}`}
                    key={entry.id}
                  >
                    <header>
                      <span
                        className={`avatar ${person.kind === "AI participant" ? "ai" : ""}`}
                      >
                        {person.initials}
                      </span>
                      <strong>{person.name}</strong>
                      <time>{entry.time}</time>
                    </header>
                    {person.kind === "AI participant" && (
                      <span className="turn-kind">AI PARTICIPANT</span>
                    )}
                    <p>{entry.text}</p>
                  </article>
                );
              }
              return (
                <article key={entry.id} className={`case-notice ${entry.kind}`}>
                  <Icon
                    name={
                      entry.kind === "review"
                        ? "review"
                        : entry.kind === "execution"
                          ? "work"
                          : "case"
                    }
                    size={15}
                  />
                  <div>
                    <header>
                      <strong>{entry.title}</strong>
                      <time>{entry.time}</time>
                    </header>
                    <p>{entry.text}</p>
                    {entry.material && (
                      <button onClick={() => open(entry.material!)}>
                        Inspect{" "}
                        {entry.kind === "review" ? "review" : "material"}
                        <Icon name="arrow" size={13} />
                      </button>
                    )}
                  </div>
                </article>
              );
            })}
          </div>
          <div className="draft-composer">
            <label htmlFor="local-draft">
              Local draft <span>NOT SUBMITTED</span>
            </label>
            <textarea
              id="local-draft"
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="Keep a note for this conversation…"
              rows={2}
            />
            <div>
              <small>Draft stays here. Sending is unavailable.</small>
              {draft && <button onClick={() => setDraft("")}>Clear</button>}
            </div>
          </div>
        </>
      )}
      {mode === "Inspector" && (
        <div className="inspector-panel" role="tabpanel" tabIndex={0}>
          <span className="eyebrow">{inspector.eyebrow} · FIXTURE</span>
          <h2>{inspector.title}</h2>
          <p>{inspector.description}</p>
          <dl>
            {inspector.facts.map((fact) => (
              <div key={fact.label}>
                <dt>{fact.label}</dt>
                <dd>{fact.value}</dd>
              </div>
            ))}
          </dl>
          {inspector.note && <p className="plain-note">{inspector.note}</p>}
          <section>
            <span className="eyebrow">LOCAL CONTEXT</span>
            <p>
              Selection changes this inspector only. It does not mutate or
              select canonical Case state.
            </p>
          </section>
        </div>
      )}
      {mode === "Activity" && (
        <div className="context-activity" role="tabpanel" tabIndex={0}>
          <header>
            <span className="eyebrow">VISIBLE THROUGH GENERATION {step}</span>
            <h2>Recent Case activity</h2>
          </header>
          <ol>
            {events.map((event) => (
              <li key={event.id}>
                <time>{event.time}</time>
                <button onClick={() => event.material && open(event.material)}>
                  <strong>{event.title}</strong>
                  <small>{event.detail}</small>
                </button>
              </li>
            ))}
          </ol>
          <p className="plain-note">
            This sequence is a frozen authored projection. It never advances on
            a timer.
          </p>
        </div>
      )}
    </aside>
  );
}
