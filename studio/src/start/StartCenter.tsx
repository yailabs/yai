import { useMemo, useState } from "react";
import type {
  ScenarioId,
  StartCenterPresentation,
} from "../clients/presentation";
import { Icon } from "../components/Icon";

export function StartCenter({
  data,
  openCase,
  newCase,
}: {
  data: StartCenterPresentation;
  openCase: (id: ScenarioId) => void;
  newCase: (fromSource: boolean) => void;
}) {
  const [showOpen, setShowOpen] = useState(false);
  const [query, setQuery] = useState("");
  const matches = useMemo(
    () =>
      data.recentCases.filter((item) =>
        `${item.label} ${item.reference} ${item.currentWork}`
          .toLowerCase()
          .includes(query.toLowerCase()),
      ),
    [data.recentCases, query],
  );
  const last = data.recentCases.find((item) => item.id === data.lastCase)!;
  return (
    <main className="start-center" aria-label="YAI Studio Start Center">
      <section className="start-intro">
        <span className="eyebrow">CASE WORKBENCH · OFFLINE FIXTURE</span>
        <h1>Continue a Case.</h1>
        <p>
          Open a durable Case workspace or compose a new fixture draft from its
          sources, participants, authority, resources and compute posture.
        </p>
        <div className="start-actions">
          <button className="primary-action" onClick={() => openCase(last.id)}>
            <Icon name="arrow" size={15} />
            Continue {last.label}
          </button>
          <button onClick={() => setShowOpen((value) => !value)}>
            <Icon name="case" size={16} /> Open Case
          </button>
          <button onClick={() => newCase(false)}>
            <Icon name="plus" size={16} /> New Case
          </button>
          <button onClick={() => newCase(true)}>
            <Icon name="sources" size={16} /> New Case from Source
          </button>
        </div>
      </section>

      {showOpen && (
        <section className="open-case-panel" aria-label="Open Case fixture">
          <div>
            <span className="eyebrow">OPEN CASE</span>
            <h2>Select a fixture Case</h2>
          </div>
          <label className="case-search">
            <Icon name="search" size={16} />
            <span className="sr-only">Filter Cases</span>
            <input
              autoFocus
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search by Case or current work"
            />
          </label>
          <div className="open-case-results">
            {matches.map((item) => (
              <button key={item.id} onClick={() => openCase(item.id)}>
                <Icon name="case" size={17} />
                <span>
                  <strong>{item.label}</strong>
                  <small>
                    {item.reference} · {item.currentWork}
                  </small>
                </span>
                <Icon name="arrow" size={14} />
              </button>
            ))}
            {!matches.length && <p>No authored fixture Case matches.</p>}
          </div>
        </section>
      )}

      <section className="start-body">
        <div className="recent-cases">
          <header className="section-heading">
            <div>
              <span className="eyebrow">RECENT / ACTIVE</span>
              <h2>Cases</h2>
            </div>
            <span>Authored fixture index</span>
          </header>
          <div className="case-list">
            {data.recentCases.map((item) => (
              <button
                className="recent-case-row"
                key={item.id}
                onClick={() => openCase(item.id)}
              >
                <span className="case-mark">
                  <Icon name="case" size={18} />
                </span>
                <span className="recent-case-main">
                  <span className="recent-case-title">
                    <strong>{item.label}</strong>
                    <span>{item.reference}</span>
                  </span>
                  <span>{item.currentWork}</span>
                  <small>{item.purpose}</small>
                </span>
                <span className="recent-case-meta">
                  <span>{item.posture}</span>
                  <small>{item.environment}</small>
                  <time>{item.updated}</time>
                </span>
                <Icon name="chevron" size={15} />
              </button>
            ))}
          </div>
        </div>

        <aside className="start-secondary">
          <section>
            <span className="eyebrow">RECENT SOURCES</span>
            <div className="quiet-list">
              {data.recentSources.map((source) => (
                <div key={source.label}>
                  <Icon name="file" size={15} />
                  <span>
                    <strong>{source.label}</strong>
                    <small>
                      {source.kind} · {source.caseLabel}
                    </small>
                  </span>
                </div>
              ))}
            </div>
          </section>
          <section>
            <span className="eyebrow">ENVIRONMENTS</span>
            <div className="quiet-list">
              {data.environments.map((environment) => (
                <div key={environment.label}>
                  <Icon name="environment" size={15} />
                  <span>
                    <strong>{environment.label}</strong>
                    <small>{environment.detail}</small>
                  </span>
                  <em>{environment.posture}</em>
                </div>
              ))}
            </div>
          </section>
        </aside>
      </section>
      <p className="fixture-disclaimer">
        FIXTURE · Recent activity and availability are deterministic
        presentation data. No YAI runtime is queried.
      </p>
    </main>
  );
}
