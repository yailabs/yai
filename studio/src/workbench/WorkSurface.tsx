import type {
  MaterialView,
  Posture,
  WorkspacePresentation,
} from "../clients/presentation";
import { Icon } from "../components/Icon";
import { Tabs } from "../components/Tabs";

export function Status({ posture }: { posture: Posture }) {
  const icon =
    posture === "completed"
      ? "check"
      : posture === "failed"
        ? "close"
        : posture === "waiting for review"
          ? "review"
          : "clock";
  return (
    <span className={`status status-${posture.replaceAll(" ", "-")}`}>
      <Icon name={icon} size={13} />
      {posture}
    </span>
  );
}
function Material({
  material,
  data,
  open,
}: {
  material: MaterialView;
  data: WorkspacePresentation;
  open: (id: string) => void;
}) {
  const body = material.body;
  if (body.kind === "diff")
    return (
      <div className="diff-surface">
        <div className="surface-intro">
          <span className="eyebrow">PROPOSED CHANGE / FIXTURE</span>
          <h2>{body.title}</h2>
          <p>{body.description}</p>
        </div>
        <div className="diff-legend">
          <span>
            {body.before} <span aria-hidden="true">→</span> {body.after}
          </span>
          <span className="diff-count">
            +{body.lines.filter((line) => line.change === "add").length} / −
            {body.lines.filter((line) => line.change === "remove").length}
          </span>
        </div>
        <div
          className="code-scroll"
          tabIndex={0}
          aria-label="Read-only illustrative diff"
        >
          <pre>
            {body.lines.map((line, index) => (
              <div key={index} className={`code-line ${line.change ?? ""}`}>
                <span className="line-number" aria-hidden="true">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span className="line-marker">
                  {line.change === "add"
                    ? "+"
                    : line.change === "remove"
                      ? "−"
                      : " "}
                </span>
                <code>{line.text || " "}</code>
              </div>
            ))}
          </pre>
        </div>
      </div>
    );
  if (body.kind === "work")
    return (
      <article className="document work-document">
        <span className="eyebrow">WORK / FROZEN FIXTURE</span>
        <h2>{body.title}</h2>
        <p className="document-intro">{body.description}</p>
        <div className="work-caption">
          <span>ACTIVITY</span>
          <span>STATE AT SNAPSHOT</span>
        </div>
        <ol className="work-steps">
          {data.executions.map((execution, index) => (
            <li key={execution.id}>
              <span className="step-number">
                {String(index + 1).padStart(2, "0")}
              </span>
              <div>
                <strong>{execution.title}</strong>
                <p>{execution.detail}</p>
                <small>
                  {execution.id} · {execution.time}
                </small>
              </div>
              <Status posture={execution.posture} />
            </li>
          ))}
        </ol>
        <div className="work-next">
          <Icon name="evidence" />
          <div>
            <strong>Keep the consequence reviewable</strong>
            <p>
              This view is a static example. No execution is started, resumed or
              approved by Studio.
            </p>
            <div className="reference-links">
              {data.materials
                .filter((item) => item.category === "artifact")
                .map((item) => (
                  <button key={item.id} onClick={() => open(item.id)}>
                    {item.name}
                    <Icon name="arrow" size={14} />
                  </button>
                ))}
            </div>
          </div>
        </div>
      </article>
    );
  if (body.kind === "provider")
    return (
      <article className="document">
        <span className="eyebrow">PROVIDER CONTEXT / FIXTURE</span>
        <h2>{body.title}</h2>
        <p className="document-intro">Inference is context for the Case.</p>
        <dl className="provider-facts">
          <dt>Provider</dt>
          <dd>{data.provider.name}</dd>
          <dt>Location label</dt>
          <dd>{data.provider.location}</dd>
          <dt>Model label</dt>
          <dd>{data.provider.model}</dd>
          <dt>Connection</dt>
          <dd>Not attached · fixture mode</dd>
          <dt>Runtime telemetry</dt>
          <dd>Unavailable</dd>
          <dt>Qualification</dt>
          <dd>Not assessed by this fixture</dd>
        </dl>
        <p>{data.provider.note}</p>
        <div className="plain-note">
          No model management or provider configuration is available in this
          shell.
        </div>
      </article>
    );
  return (
    <article className="document">
      <span className="eyebrow">{body.eyebrow}</span>
      <h2>{body.title}</h2>
      <p className="document-intro">{body.intro}</p>
      <div className="document-byline">
        <span>{material.format}</span>
        <span>Read only</span>
        <span>Synthetic content</span>
      </div>
      {body.sections.map((section) => (
        <section key={section.title}>
          <h3>{section.title}</h3>
          <p>{section.body}</p>
          {section.points && (
            <ul>
              {section.points.map((point) => (
                <li key={point}>{point}</li>
              ))}
            </ul>
          )}
        </section>
      ))}
      {body.references && (
        <section className="document-references">
          <h3>Source context</h3>
          <div className="reference-links">
            {body.references.map((id) => {
              const item = data.materials.find((item) => item.id === id)!;
              return (
                <button key={id} onClick={() => open(id)}>
                  <Icon name="file" size={14} />
                  {item.name}
                  <Icon name="arrow" size={14} />
                </button>
              );
            })}
          </div>
        </section>
      )}
    </article>
  );
}
export function WorkSurface({
  data,
  tabs,
  active,
  select,
  close,
  open,
}: {
  data: WorkspacePresentation;
  tabs: readonly string[];
  active: string;
  select: (id: string) => void;
  close: (id: string) => void;
  open: (id: string) => void;
}) {
  const material = data.materials.find((item) => item.id === active);
  return (
    <main className="work-surface" aria-label="Case work surface">
      <Tabs
        id="surface"
        label="Open materials"
        items={tabs.map((id) => {
          const item = data.materials.find((item) => item.id === id)!;
          return { id, label: item.name, changed: item.changed };
        })}
        active={active}
        onSelect={select}
        onClose={close}
      />
      {material ? (
        <div
          role="tabpanel"
          id="surface-panel"
          aria-labelledby={`surface-${active}`}
          tabIndex={0}
          className="surface-panel"
        >
          <div className="breadcrumbs">
            <Icon name="case" size={14} />
            <span>{data.case.label}</span>
            <Icon name="chevron" size={12} />
            <span className="truncate">{material.path}</span>
            <span className="material-format">{material.format}</span>
          </div>
          <div
            className="material-scroll"
            key={active}
            tabIndex={0}
            aria-label="Material content"
          >
            <Material material={material} data={data} open={open} />
          </div>
          <div className="provenance">
            <Icon name="evidence" size={13} />
            <span>{material.provenance}</span>
          </div>
        </div>
      ) : (
        <div className="empty-surface">
          <Icon name="case" size={32} />
          <h2>{data.case.label}</h2>
          <p>Choose a material from the Case Explorer.</p>
          <button onClick={() => open(data.initial.active)}>
            Reopen{" "}
            {
              data.materials.find((item) => item.id === data.initial.active)
                ?.name
            }
            <Icon name="arrow" size={15} />
          </button>
        </div>
      )}
    </main>
  );
}
