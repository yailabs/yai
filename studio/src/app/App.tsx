import { useEffect, useMemo, useRef, useState } from "react";
import { FixtureClient } from "../clients/fixture";
import type { ScenarioId } from "../clients/presentation";
import { ApplicationMenu } from "./ApplicationMenu";
import { Icon } from "../components/Icon";
import { CaseComposer } from "../start/CaseBootstrap";
import { StartCenter } from "../start/StartCenter";
import { useLayout } from "../workbench/layout";
import { Workbench } from "../workbench/Workbench";
import { ComponentGallery } from "../live/ComponentGallery";
import { LiveStudio } from "../live/LiveStudio";
import "../styles/workbench.css";
import "../styles/foundation.css";

const client = new FixtureClient();

export function App() {
  if (import.meta.env.DEV && new URLSearchParams(window.location.search).get("gallery") === "1") {
    return <ComponentGallery />;
  }
  if (import.meta.env.VITE_STUDIO_MODE !== "fixture") {
    return <LiveStudio />;
  }
  return <FixtureApp />;
}

function FixtureApp() {
  const [location, setLocation] = useState(window.location.search);
  const layout = useLayout();
  const params = useMemo(() => new URLSearchParams(location), [location]);
  const choices = client.scenarios();
  const scenario = params.get("fixture");
  const validScenario = choices.some((choice) => choice.id === scenario);
  const data = validScenario ? client.workspace(scenario as ScenarioId) : null;
  const view =
    params.get("view") === "new"
      ? "new"
      : scenario && !validScenario
        ? "unknown"
        : data
          ? "case"
          : "start";
  const composition = client.composition();
  const requestedFocus = params.get("focus");
  const focusSection = composition.find(
    (section) => section.id === requestedFocus,
  )?.id;
  const returnCase = choices.some(
    (choice) => choice.id === params.get("return"),
  )
    ? (params.get("return") as ScenarioId)
    : null;
  const attachedCase = useRef<ScenarioId | null>(
    validScenario ? (scenario as ScenarioId) : returnCase,
  );
  const requestedSnapshot = params.get("snapshot");
  const snapshot = data?.information.progression.steps.some(
    (step) => step.id === requestedSnapshot,
  )
    ? requestedSnapshot!
    : data?.information.progression.initial;
  const participant = data?.participants.find(
    (person) => person.id === data.case.participant,
  );

  useEffect(() => {
    const pop = () => {
      const next = new URLSearchParams(window.location.search);
      const returningToLocalSurface =
        next.get("view") === "new" ||
        choices.some((choice) => choice.id === next.get("fixture"));
      if (attachedCase.current && !returningToLocalSurface) {
        const retained = `?fixture=${attachedCase.current}`;
        window.history.replaceState(null, "", retained);
        setLocation(retained);
        return;
      }
      setLocation(window.location.search);
    };
    window.addEventListener("popstate", pop);
    return () => window.removeEventListener("popstate", pop);
  }, []);
  useEffect(() => {
    document.title =
      view === "start"
        ? "YAI Studio · Start Center · Fixture"
        : view === "new"
          ? "New Case — YAI Studio · Fixture"
          : view === "unknown"
            ? "Unknown fixture — YAI Studio"
            : data
              ? `${data.case.label} — YAI Studio · Fixture`
              : "YAI Studio · Fixture";
  }, [data, view]);

  function navigate(next: URLSearchParams, replace = false) {
    const search = next.toString();
    window.history[replace ? "replaceState" : "pushState"](
      null,
      "",
      search ? `?${search}` : window.location.pathname,
    );
    setLocation(search ? `?${search}` : "");
  }
  function goStart() {
    navigate(new URLSearchParams());
  }
  function openCase(id: ScenarioId) {
    attachedCase.current = id;
    navigate(new URLSearchParams({ fixture: id }), true);
  }
  function newCase(fromSource: boolean) {
    const next = new URLSearchParams({ view: "new" });
    if (fromSource) next.set("focus", "sources");
    const origin = validScenario ? (scenario as ScenarioId) : returnCase;
    if (origin) next.set("return", origin);
    navigate(next);
  }
  function closeComposition() {
    if (returnCase) openCase(returnCase);
    else goStart();
  }
  function changeSnapshot(id: string) {
    const next = new URLSearchParams(params);
    next.set("snapshot", id);
    navigate(next);
  }

  return (
    <div className={`studio studio-${view}`}>
      <ApplicationMenu
        view={view}
        cases={choices}
        currentCase={validScenario ? (scenario as ScenarioId) : returnCase}
        layout={layout}
        openCase={openCase}
        newCase={newCase}
      />
      <header className="case-chrome">
        <span className="brand">
          YAI<span>STUDIO</span>
        </span>
        <div className="chrome-case">
          <Icon name={view === "new" ? "plus" : "case"} size={17} />
          <strong>
            {data?.case.label ?? (view === "new" ? "New Case" : "Cases")}
          </strong>
          <span className="chrome-path">
            {data?.case.context ??
              (view === "new" ? "Fixture composition" : "Start Center")}
          </span>
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
        {data && (
          <div className="chrome-provider" title={data.provider.note}>
            <Icon name="compute" size={14} />
            <span>
              {data.provider.location} / {data.provider.name}
            </span>
          </div>
        )}
        <span className="fixture-badge">FIXTURE</span>
        {data && (
          <>
            <label className="snapshot-picker">
              <span className="sr-only">Fixture generation</span>
              <select
                aria-label="Fixture generation"
                value={snapshot}
                onChange={(event) => changeSnapshot(event.target.value)}
              >
                {data.information.progression.steps.map((step) => (
                  <option key={step.id} value={step.id}>
                    Gen {step.id} · {step.label}
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
                aria-label="Toggle context panel"
                aria-expanded={layout.rightOpen}
                aria-controls="case-conversation"
                title="Context panel (Ctrl/⌘ Shift B)"
                onClick={() => layout.setRightOpen((value) => !value)}
              >
                <Icon name="right" size={17} />
              </button>
            </div>
          </>
        )}
      </header>

      {view === "start" && (
        <StartCenter
          data={client.catalog()}
          openCase={openCase}
          newCase={newCase}
        />
      )}
      {view === "new" && (
        <CaseComposer
          sections={composition}
          focusSection={focusSection}
          close={closeComposition}
        />
      )}
      {view === "case" && data && (
        <Workbench
          key={scenario}
          data={data}
          layout={layout}
          stepId={snapshot!}
        />
      )}
      {view === "unknown" && (
        <main className="unknown-fixture">
          <h1>Unknown fixture: {scenario}</h1>
          <p>
            Select an authored Case from the Start Center. No replacement data
            has been loaded.
          </p>
          <button className="primary-action" onClick={goStart}>
            Open Start Center
          </button>
        </main>
      )}

      <footer className="status-bar">
        <span className="fixture-status">FIXTURE DATA</span>
        <span>No runtime attached</span>
        {participant && (
          <span className="status-participant">
            {participant.name} · local view
          </span>
        )}
        <span className="status-end">
          {view === "case"
            ? "Layout stays in Studio / No changes sent"
            : "Offline product presentation / No Case mutation"}
        </span>
      </footer>
    </div>
  );
}
