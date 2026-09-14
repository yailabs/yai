import type {
  Activity,
  ExplorerGroup,
  ExplorerItem,
  MemoryMode,
  WorkspacePresentation,
} from "../clients/presentation";
import { Icon } from "../components/Icon";
import { Status } from "./WorkSurface";

function ItemRow({
  item,
  open,
  inspect,
}: {
  item: ExplorerItem;
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  const activate = () => {
    inspect(item.id);
    if (item.material) open(item.material);
  };
  return (
    <button className="information-row" onClick={activate}>
      <span className={`information-icon kind-${item.kind}`}>
        <Icon
          name={
            item.kind === "policy" ||
            item.kind === "review" ||
            item.kind === "decision"
              ? "authority"
              : item.kind === "provider" ||
                  item.kind === "model" ||
                  item.kind === "machine"
                ? "compute"
                : item.kind === "knowledge"
                  ? "knowledge"
                  : item.kind === "memory"
                    ? "memory"
                    : item.kind === "workflow" || item.kind === "execution"
                      ? "work"
                      : item.kind === "repository"
                        ? "repository"
                        : "file"
          }
          size={16}
        />
      </span>
      <span>
        <strong>{item.label}</strong>
        <small>{item.detail}</small>
      </span>
      {item.posture && (
        <em className={`posture-${item.posture.replaceAll(" ", "-")}`}>
          {item.posture}
        </em>
      )}
      <Icon name="chevron" size={13} />
    </button>
  );
}

function Groups({
  groups,
  open,
  inspect,
}: {
  groups: readonly ExplorerGroup[];
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  return (
    <div className="information-groups">
      {groups.map((group) => (
        <section className="information-group" key={group.label}>
          <header>
            <h3>{group.label}</h3>
            <span>{group.items.length}</span>
          </header>
          {group.note && <p className="group-note">{group.note}</p>}
          <div>
            {group.items.map((item) => (
              <ItemRow
                key={item.id}
                item={item}
                open={open}
                inspect={inspect}
              />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function Overview({
  data,
  open,
  inspect,
}: {
  data: WorkspacePresentation;
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  const info = data.information;
  return (
    <div className="overview-surface section-layout">
      <header className="section-hero">
        <div>
          <span className="eyebrow">{data.case.reference} · OVERVIEW</span>
          <h2>{data.case.label}</h2>
          <p>{data.case.purpose}</p>
        </div>
        <div className="case-posture">
          <span className="offline-dot" />
          <strong>{info.overview.status}</strong>
          <small>Fixture snapshot</small>
        </div>
      </header>
      <div className="overview-grid">
        <section className="focus-block">
          <span className="eyebrow">CURRENT WORK</span>
          <h3>{data.case.currentWork}</h3>
          <p>{info.overview.attention}</p>
          <button className="text-link" onClick={() => open(`section:Work`)}>
            Open work <Icon name="arrow" size={14} />
          </button>
        </section>
        <section className="participant-block">
          <span className="eyebrow">PARTICIPANTS</span>
          <div className="overview-people">
            {data.participants.map((person) => (
              <button key={person.id} onClick={() => inspect(person.id)}>
                <span
                  className={`avatar ${person.kind === "AI participant" ? "ai" : ""}`}
                >
                  {person.initials}
                </span>
                <span>
                  <strong>{person.name}</strong>
                  <small>
                    {person.kind === "AI participant" ? "AI · " : ""}
                    {person.role}
                  </small>
                </span>
              </button>
            ))}
          </div>
        </section>
      </div>
      <section className="overview-highlights">
        <header className="section-heading">
          <div>
            <span className="eyebrow">CASE NOW</span>
            <h3>Relevant material and attention</h3>
          </div>
          <span>At this fixture snapshot</span>
        </header>
        <div>
          {info.overview.highlights.map((entry) => (
            <ItemRow
              key={entry.id}
              item={entry}
              open={open}
              inspect={inspect}
            />
          ))}
        </div>
      </section>
      <section className="overview-trace">
        <span className="eyebrow">RECENT ACTIVITY</span>
        {info.memory.timeline
          .slice(-3)
          .reverse()
          .map((event) => (
            <button
              key={event.id}
              onClick={() => {
                inspect(event.material ?? event.id);
                if (event.material) open(event.material);
              }}
            >
              <time>{event.time}</time>
              <span>
                <strong>{event.title}</strong>
                <small>{event.detail}</small>
              </span>
            </button>
          ))}
      </section>
    </div>
  );
}

function MemorySurface({
  data,
  mode,
  setMode,
  step,
  open,
  inspect,
}: {
  data: WorkspacePresentation;
  mode: MemoryMode;
  setMode: (mode: MemoryMode) => void;
  step: number;
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  const memory = data.information.memory;
  const events = memory.timeline.filter((event) => event.step <= step);
  const nodes = memory.nodes.filter((node) => node.step <= step);
  const nodeIds = new Set(nodes.map((node) => node.id));
  const edges = memory.edges.filter(
    (edge) =>
      edge.step <= step && nodeIds.has(edge.from) && nodeIds.has(edge.to),
  );
  return (
    <div className="memory-surface section-layout">
      <header className="section-heading memory-heading">
        <div>
          <span className="eyebrow">DERIVED CASE MEMORY · FIXTURE</span>
          <h2>History and relations</h2>
        </div>
        <div className="segmented" role="tablist" aria-label="Memory views">
          {(["Timeline", "Graph"] as const).map((value) => (
            <button
              role="tab"
              aria-selected={mode === value}
              key={value}
              onClick={() => setMode(value)}
            >
              <Icon name={value === "Graph" ? "graph" : "memory"} size={14} />
              {value}
            </button>
          ))}
        </div>
      </header>
      <p className="section-description">
        A temporal and relational presentation of disclosed fixture facts. The
        graph is derived navigation, never causal or operational authority.
      </p>
      {mode === "Timeline" ? (
        <ol className="case-timeline">
          {events.map((event) => (
            <li key={event.id}>
              <time>{event.time}</time>
              <span className={`timeline-dot kind-${event.kind}`} />
              <button
                onClick={() => {
                  inspect(event.material ?? event.id);
                  if (event.material) open(event.material);
                }}
              >
                <strong>{event.title}</strong>
                <span>{event.detail}</span>
                <small>
                  {event.kind} · generation {event.step}
                </small>
              </button>
            </li>
          ))}
        </ol>
      ) : (
        <div className="graph-wrap">
          <div className="graph-canvas" aria-label="Fixture Case graph">
            <svg
              viewBox="0 0 780 380"
              preserveAspectRatio="none"
              aria-hidden="true"
            >
              <defs>
                <marker
                  id="arrowhead"
                  markerWidth="7"
                  markerHeight="7"
                  refX="6"
                  refY="3.5"
                  orient="auto"
                >
                  <path d="M0,0 L7,3.5 L0,7" />
                </marker>
              </defs>
              {edges.map((edge) => {
                const from = nodes.find((node) => node.id === edge.from)!;
                const to = nodes.find((node) => node.id === edge.to)!;
                return (
                  <g key={`${edge.from}-${edge.to}`}>
                    <line
                      x1={from.x}
                      y1={from.y}
                      x2={to.x}
                      y2={to.y}
                      markerEnd="url(#arrowhead)"
                    />
                    <text x={(from.x + to.x) / 2} y={(from.y + to.y) / 2 - 7}>
                      {edge.label}
                    </text>
                  </g>
                );
              })}
            </svg>
            {nodes.map((node) => (
              <button
                key={node.id}
                className={`graph-node kind-${node.kind}`}
                style={{
                  left: `${(node.x / 780) * 100}%`,
                  top: `${(node.y / 380) * 100}%`,
                }}
                onClick={() => {
                  inspect(node.material ?? node.id);
                  if (node.material) open(node.material);
                }}
              >
                <span>
                  <Icon
                    name={
                      node.kind === "case"
                        ? "case"
                        : node.kind === "participant"
                          ? "people"
                          : node.kind === "source"
                            ? "sources"
                            : node.kind === "review"
                              ? "authority"
                              : node.kind === "execution"
                                ? "work"
                                : "file"
                    }
                    size={15}
                  />
                </span>
                <strong>{node.label}</strong>
                <small>{node.detail}</small>
              </button>
            ))}
          </div>
          <footer>
            <span>Visible through generation {step}</span>
            <span>
              Edges express fixture relations · not inferred causality
            </span>
          </footer>
        </div>
      )}
    </div>
  );
}

function WorkSurfaceView({
  data,
  step,
  open,
  inspect,
}: {
  data: WorkspacePresentation;
  step: number;
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  const visible = data.executions.slice(
    0,
    Math.max(
      1,
      Math.ceil(
        (data.executions.length * step) /
          data.information.progression.steps.length,
      ),
    ),
  );
  return (
    <div className="work-information section-layout">
      <header className="section-heading">
        <div>
          <span className="eyebrow">WORK · FIXTURE SNAPSHOT</span>
          <h2>Activity and transformations</h2>
        </div>
        <span>No execution is live</span>
      </header>
      <div className="work-lanes">
        {visible.map((execution, index) => (
          <button key={execution.id} onClick={() => inspect(execution.id)}>
            <span className="step-number">
              {String(index + 1).padStart(2, "0")}
            </span>
            <span>
              <strong>{execution.title}</strong>
              <small>
                {execution.detail} · {execution.time}
              </small>
            </span>
            <Status posture={execution.posture} />
          </button>
        ))}
      </div>
      <Groups groups={data.information.work} open={open} inspect={inspect} />
    </div>
  );
}

function ComputeSurface({
  data,
  open,
  inspect,
}: {
  data: WorkspacePresentation;
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  return (
    <div className="compute-surface section-layout">
      <header className="section-heading">
        <div>
          <span className="eyebrow">COMPUTE CONTEXT · FIXTURE</span>
          <h2>Inference and runtime targets</h2>
        </div>
        <span className="offline-capsule">NOT CONNECTED</span>
      </header>
      <div className="compute-summary">
        <div>
          <Icon name="compute" size={22} />
          <span>
            <strong>
              {data.provider.location} / {data.provider.name}
            </strong>
            <small>{data.provider.model}</small>
          </span>
        </div>
        <dl>
          <div>
            <dt>Posture</dt>
            <dd>{data.provider.posture}</dd>
          </div>
          <div>
            <dt>Telemetry</dt>
            <dd>Unavailable</dd>
          </div>
          <div>
            <dt>Management</dt>
            <dd>Not implemented</dd>
          </div>
        </dl>
        <p>{data.provider.note}</p>
      </div>
      <Groups groups={data.information.compute} open={open} inspect={inspect} />
      <p className="plane-note">
        <strong>Inference plane</strong> stays provider-generic. A future YVEX
        management plane requires its own qualified public capabilities.
      </p>
    </div>
  );
}

export function SectionSurface({
  activity,
  data,
  memoryMode,
  setMemoryMode,
  step,
  open,
  inspect,
}: {
  activity: Activity;
  data: WorkspacePresentation;
  memoryMode: MemoryMode;
  setMemoryMode: (mode: MemoryMode) => void;
  step: number;
  open: (id: string) => void;
  inspect: (id: string) => void;
}) {
  if (activity === "Overview")
    return <Overview data={data} open={open} inspect={inspect} />;
  if (activity === "Memory")
    return (
      <MemorySurface
        data={data}
        mode={memoryMode}
        setMode={setMemoryMode}
        step={step}
        open={open}
        inspect={inspect}
      />
    );
  if (activity === "Work")
    return (
      <WorkSurfaceView data={data} step={step} open={open} inspect={inspect} />
    );
  if (activity === "Compute")
    return <ComputeSurface data={data} open={open} inspect={inspect} />;
  const groups =
    activity === "Environment"
      ? data.information.environment
      : activity === "Knowledge"
        ? data.information.knowledge
        : data.information.authority;
  const copy =
    activity === "Environment"
      ? [
          "The world attached to this Case.",
          "Sources provide material and context. Resources provide governed operational capability; a single entity may have both roles without collapsing them.",
        ]
      : activity === "Knowledge"
        ? [
            "What the Case can present from qualified material.",
            "Claims and relations here are authored derived fixtures. Source material remains separately inspectable and no disagreement selects truth.",
          ]
        : [
            "Policy, scope, reviews and decisions.",
            "Authority is constitutive of the Case. These rows present posture only; no fixture control can decide, grant or execute.",
          ];
  return (
    <div className={`section-layout ${activity.toLowerCase()}-surface`}>
      <header className="section-heading">
        <div>
          <span className="eyebrow">{activity.toUpperCase()} · FIXTURE</span>
          <h2>{copy[0]}</h2>
        </div>
        <span>
          {groups.reduce((count, group) => count + group.items.length, 0)}{" "}
          presented items
        </span>
      </header>
      <p className="section-description">{copy[1]}</p>
      <Groups groups={groups} open={open} inspect={inspect} />
    </div>
  );
}
