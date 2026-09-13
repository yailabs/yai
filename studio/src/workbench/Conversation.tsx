import type { WorkspacePresentation } from "../clients/presentation";
import { Icon } from "../components/Icon";

export function Conversation({
  data,
  open,
  collapse,
  draft,
  setDraft,
}: {
  data: WorkspacePresentation;
  open: (id: string) => void;
  collapse: () => void;
  draft: string;
  setDraft: (text: string) => void;
}) {
  return (
    <aside
      className="conversation"
      id="case-conversation"
      aria-label="Case conversation"
    >
      <div className="region-heading">
        <span>CONVERSATION</span>
        <button
          className="icon-button"
          onClick={collapse}
          aria-label="Collapse conversation"
          title="Collapse conversation"
        >
          <Icon name="right" size={16} />
        </button>
      </div>
      <div className="thread-context">
        <span className="eyebrow">IN THIS CASE</span>
        <h2>{data.case.currentWork}</h2>
        <div className="thread-people">
          <Icon name="people" size={14} />
          <span>{data.participants.length} participants</span>
          <span>Thread 01 · fixture</span>
        </div>
        <div className="thread-provider">
          <Icon name="provider" size={12} />
          {data.provider.model}
        </div>
      </div>
      <div
        className="conversation-scroll"
        tabIndex={0}
        aria-label="Conversation history"
      >
        <div className="snapshot-divider">
          <span>Snapshot</span>
          <span>13 September · fixture</span>
        </div>
        {data.activity.map((entry) => {
          if (entry.kind === "turn") {
            const person = data.participants.find(
              (person) => person.id === entry.participant,
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
                    Inspect {entry.kind === "review" ? "review" : "material"}
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
    </aside>
  );
}
