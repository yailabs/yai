import { useEffect, useState } from "react";
import { FixtureClient } from "../clients/fixture";
import type { ScenarioId } from "../clients/presentation";
import { Icon } from "../components/Icon";
import { useLayout } from "../workbench/layout";
import { Workbench } from "../workbench/Workbench";
import "../styles/workbench.css";

const client = new FixtureClient();
const readScenario = () =>
  new URLSearchParams(window.location.search).get("fixture") ?? "ordinary";
export function App() {
  const [scenario, setScenario] = useState(readScenario);
  const layout = useLayout();
  const choices = client.scenarios();
  const valid = choices.some((choice) => choice.id === scenario);
  const data = valid ? client.workspace(scenario as ScenarioId) : null;
  const participant = data?.participants.find(
    (person) => person.id === data.case.participant,
  );
  useEffect(() => {
    const pop = () => setScenario(readScenario());
    window.addEventListener("popstate", pop);
    return () => window.removeEventListener("popstate", pop);
  }, []);
  useEffect(() => {
    document.title = data
      ? `${data.case.label} — YAI Studio · Fixture`
      : "Unknown fixture — YAI Studio";
  }, [data]);
  function selectScenario(id: string) {
    const url = new URL(window.location.href);
    url.searchParams.set("fixture", id);
    window.history.pushState(null, "", url);
    setScenario(id);
  }
  return (
    <div className="studio">
      <header className="case-chrome">
        <a
          className="brand"
          href="?fixture=ordinary"
          aria-label="YAI Studio ordinary fixture"
        >
          YAI<span>STUDIO</span>
        </a>
        <div className="chrome-case">
          <Icon name="case" size={17} />
          <strong>{data?.case.label ?? "Unknown fixture"}</strong>
          <span className="chrome-path">{data?.case.context}</span>
        </div>
        {participant && (
          <div
            className="chrome-participant"
            title={`Current participant · ${participant.name}`}
          >
            <span className="avatar">{participant.initials}</span>
            <span>{participant.name}</span>
          </div>
        )}
        <div className="chrome-provider" title={data?.provider.note}>
          <Icon name="provider" size={14} />
          <span>
            {data?.provider.location} / {data?.provider.name}
          </span>
        </div>
        <label className="fixture-picker">
          <span>FIXTURE</span>
          <select
            aria-label="Fixture scenario"
            title="Switch fixture; resets material tabs and the unsubmitted draft"
            value={valid ? scenario : ""}
            onChange={(event) => selectScenario(event.target.value)}
          >
            {!valid && (
              <option value="" disabled>
                Unknown scenario
              </option>
            )}
            {choices.map((choice) => (
              <option key={choice.id} value={choice.id}>
                {choice.label}
              </option>
            ))}
          </select>
        </label>
        <div className="layout-controls">
          <button
            className="icon-button"
            aria-label="Toggle Case explorer"
            aria-expanded={layout.leftOpen}
            aria-controls="case-sidebar"
            title="Case explorer (Ctrl/⌘ B)"
            onClick={() => layout.setLeftOpen((value) => !value)}
          >
            <Icon name="left" size={17} />
          </button>
          <button
            className="icon-button"
            aria-label="Toggle bottom panel"
            aria-expanded={layout.bottomOpen}
            aria-controls="case-tools"
            title="Bottom panel (Ctrl/⌘ J)"
            onClick={() => layout.setBottomOpen((value) => !value)}
          >
            <Icon name="bottom" size={17} />
          </button>
          <button
            className="icon-button"
            aria-label="Toggle conversation"
            aria-expanded={layout.rightOpen}
            aria-controls="case-conversation"
            title="Conversation (Ctrl/⌘ Shift B)"
            onClick={() => layout.setRightOpen((value) => !value)}
          >
            <Icon name="right" size={17} />
          </button>
        </div>
      </header>
      {data ? (
        <Workbench key={scenario} data={data} layout={layout} />
      ) : (
        <main className="unknown-fixture">
          <h1>Unknown fixture: {scenario}</h1>
          <p>
            Select one of the authored scenarios above. No replacement data has
            been loaded.
          </p>
        </main>
      )}
      <footer className="status-bar">
        <span className="fixture-status">FIXTURE DATA</span>
        <span>No runtime attached</span>
        <span className="status-participant">
          {participant?.name} · local view
        </span>
        <span className="status-end">
          Layout stays in Studio <span aria-hidden="true">/</span> No changes
          sent
        </span>
      </footer>
    </div>
  );
}
