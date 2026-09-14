import type { BottomTab, WorkspacePresentation } from "../clients/presentation";
import { Tabs } from "../components/Tabs";
import { Icon } from "../components/Icon";
import { Status } from "./WorkSurface";

const bottomTabs: readonly BottomTab[] = [
  "Terminal",
  "Output",
  "Executions",
  "Evidence",
  "Problems",
];
export function BottomPanel({
  data,
  active,
  select,
  collapse,
  open,
  height,
  step,
}: {
  data: WorkspacePresentation;
  active: BottomTab;
  select: (tab: BottomTab) => void;
  collapse: () => void;
  open: (id: string) => void;
  height: number;
  step: number;
}) {
  const stepCount = data.information.progression.steps.length;
  const visibleExecutions = data.executions.slice(
    0,
    Math.max(1, Math.ceil((data.executions.length * step) / stepCount)),
  );
  const visibleEvidence = data.evidence.slice(
    0,
    Math.max(1, Math.ceil((data.evidence.length * step) / stepCount)),
  );
  const visibleProblems = data.problems.slice(
    0,
    Math.ceil((data.problems.length * Math.max(0, step - 1)) / stepCount),
  );
  return (
    <section
      className="bottom-panel"
      id="case-tools"
      style={{ height }}
      aria-label="Case tools"
    >
      <header className="bottom-header">
        <Tabs
          id="tools"
          label="Case tools"
          items={bottomTabs.map((id) => ({ id, label: id }))}
          active={active}
          onSelect={(id) => select(id as BottomTab)}
        />
        <button
          className="icon-button"
          onClick={collapse}
          aria-label="Collapse bottom panel"
          title="Collapse bottom panel"
        >
          <Icon name="close" size={14} />
        </button>
      </header>
      <div className="tool-caption">
        <span>
          {active === "Terminal" ? "NATIVE HOST" : "FIXTURE SNAPSHOT"}
        </span>
        <span>
          No live {active === "Terminal" ? "PTY" : "runtime"} attached
        </span>
      </div>
      <div
        className="tool-content"
        role="tabpanel"
        id="tools-panel"
        aria-labelledby={`tools-${active}`}
        tabIndex={0}
      >
        {active === "Terminal" && (
          <div className="terminal-empty">
            <Icon name="terminal" size={23} />
            <div>
              <strong>Terminal host not attached in fixture mode</strong>
              <p>
                A real shell or YAI REPL will live here. This surface accepts no
                commands.
              </p>
            </div>
          </div>
        )}
        {active === "Output" && (
          <pre className="output-log">{data.output.join("\n")}</pre>
        )}
        {active === "Executions" && (
          <table className="data-table">
            <thead>
              <tr>
                <th>Execution</th>
                <th>State</th>
                <th>Snapshot</th>
              </tr>
            </thead>
            <tbody>
              {visibleExecutions.map((item) => (
                <tr key={item.id}>
                  <td>
                    <strong>{item.title}</strong>
                    <small>
                      {item.id} · {item.detail}
                    </small>
                  </td>
                  <td>
                    <Status posture={item.posture} />
                  </td>
                  <td className="mono">{item.time}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {active === "Evidence" && (
          <table className="data-table">
            <thead>
              <tr>
                <th>Evidence</th>
                <th>Source context</th>
              </tr>
            </thead>
            <tbody>
              {visibleEvidence.map((item) => (
                <tr key={item.id}>
                  <td>
                    {item.material ? (
                      <button
                        className="text-link"
                        onClick={() => open(item.material!)}
                      >
                        {item.title}
                        <Icon name="arrow" size={13} />
                      </button>
                    ) : (
                      item.title
                    )}
                    <small>
                      {item.id} · {item.detail}
                    </small>
                  </td>
                  <td>{item.origin}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {active === "Problems" &&
          (visibleProblems.length ? (
            <ul className="problem-list">
              {visibleProblems.map((item) => (
                <li key={item.id}>
                  <span className={`problem-level ${item.severity}`}>
                    <Icon name="review" size={14} />
                    {item.severity}
                  </span>
                  <button onClick={() => open(item.material)}>
                    <strong>{item.title}</strong>
                    <small>{item.detail}</small>
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <div className="terminal-empty">
              <Icon name="check" />
              <div>
                <strong>No problems in this fixture snapshot</strong>
                <p>No live diagnostics are attached.</p>
              </div>
            </div>
          ))}
      </div>
    </section>
  );
}
