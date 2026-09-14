import { useState } from "react";
import type { CompositionSection } from "../clients/presentation";
import { Icon } from "../components/Icon";

type SectionId = CompositionSection["id"];

function CompositionContent({ id }: { id: SectionId }) {
  if (id === "identity")
    return (
      <div className="composer-fields">
        <label className="setup-field">
          <span>Case name</span>
          <input defaultValue="New inquiry" />
        </label>
        <label className="setup-field">
          <span>Purpose</span>
          <textarea
            defaultValue="Establish the question, assemble qualified context and keep consequential work reviewable."
            rows={2}
          />
        </label>
        <p className="composer-principle">
          <Icon name="memory" size={15} />
          Durable continuity across client attachments
        </p>
      </div>
    );

  if (id === "sources")
    return (
      <div className="source-kind-grid compact">
        {["File", "Folder", "Repository", "Database", "API", "MCP"].map(
          (kind, index) => (
            <label key={kind}>
              <input type="checkbox" defaultChecked={index < 2} />
              <Icon
                name={
                  kind === "Repository"
                    ? "repository"
                    : kind === "Database"
                      ? "database"
                      : kind === "API" || kind === "MCP"
                        ? "compute"
                        : "file"
                }
                size={15}
              />
              <span>{kind}</span>
            </label>
          ),
        )}
        <p className="composer-principle">
          <Icon name="knowledge" size={15} /> Source material does not grant an
          operational capability.
        </p>
      </div>
    );

  if (id === "participants")
    return (
      <div className="setup-rows compact">
        <div>
          <span className="avatar">FM</span>
          <p>
            <strong>Francesco</strong>
            <small>Human · Case owner fixture</small>
          </p>
          <span className="setup-tag">current</span>
        </div>
        <div>
          <span className="avatar ai">CP</span>
          <p>
            <strong>Case participant</strong>
            <small>AI · analysis and preparation</small>
          </p>
          <span className="setup-tag muted">draft</span>
        </div>
      </div>
    );

  if (id === "authority")
    return (
      <div className="setup-options compact">
        <label>
          <input type="radio" name="authority" defaultChecked />
          <span>
            <strong>Review external effects</strong>
            <small>Consequences require explicit review.</small>
          </span>
        </label>
        <label>
          <input type="radio" name="authority" />
          <span>
            <strong>Observation only</strong>
            <small>No operational resource use.</small>
          </span>
        </label>
        <label>
          <input type="radio" name="authority" />
          <span>
            <strong>Configure later</strong>
            <small>Leave operations unavailable.</small>
          </span>
        </label>
      </div>
    );

  if (id === "resources")
    return (
      <div className="setup-rows compact">
        <label>
          <input type="checkbox" />
          <Icon name="environment" size={16} />
          <p>
            <strong>Working directory</strong>
            <small>Filesystem resource · not attached</small>
          </p>
        </label>
        <label>
          <input type="checkbox" />
          <Icon name="compute" size={16} />
          <p>
            <strong>Tool environment</strong>
            <small>Process capability · no native host</small>
          </p>
        </label>
        <p className="composer-principle">
          <Icon name="authority" size={15} /> Sources and governed resources
          remain distinct roles.
        </p>
      </div>
    );

  return (
    <div className="setup-options compact">
      <label>
        <input type="radio" name="compute" defaultChecked />
        <span>
          <strong>Local · OpenAI-compatible</strong>
          <small>Presentation label; no endpoint discovery.</small>
        </span>
      </label>
      <label>
        <input type="radio" name="compute" />
        <span>
          <strong>YVEX · local</strong>
          <small>Management remains unavailable.</small>
        </span>
      </label>
      <label>
        <input type="radio" name="compute" />
        <span>
          <strong>No compute</strong>
          <small>Compose without an inference target.</small>
        </span>
      </label>
    </div>
  );
}

export function CaseComposer({
  sections,
  focusSection,
  close,
}: {
  sections: readonly CompositionSection[];
  focusSection?: SectionId;
  close: () => void;
}) {
  const [acknowledged, setAcknowledged] = useState(false);

  return (
    <main className="case-bootstrap" aria-label="New Case fixture composition">
      <div
        className="composer-tabbar"
        role="tablist"
        aria-label="Case draft tabs"
      >
        <button role="tab" aria-selected="true">
          <Icon name="case" size={14} /> New Case
        </button>
      </div>
      <div className="case-composer">
        <header className="composer-heading">
          <div>
            <span className="eyebrow">NEW CASE · OFFLINE FIXTURE</span>
            <h1>Compose the Case</h1>
            <p>
              Define its continuity, material, participants and operating
              posture together. Every part remains visible in this draft.
            </p>
          </div>
          <button className="quiet-action" onClick={close}>
            <Icon name="close" size={14} /> Close draft
          </button>
        </header>

        <div className="composer-grid">
          {sections.map((section, index) => (
            <section
              className={`composer-section ${section.id === focusSection ? "focused" : ""}`}
              data-section={section.id}
              data-focus={section.id === focusSection || undefined}
              key={section.id}
              aria-labelledby={`composition-${section.id}`}
            >
              <header>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <div>
                  <span className="eyebrow">{section.eyebrow}</span>
                  <h2 id={`composition-${section.id}`}>{section.label}</h2>
                </div>
              </header>
              <p className="composer-description">{section.description}</p>
              <CompositionContent id={section.id} />
            </section>
          ))}
        </div>

        <footer className="composer-footer">
          <p>
            <Icon name="review" size={15} /> No Case, authority, provider or
            resource attachment will be created from this fixture.
          </p>
          <label>
            <input
              type="checkbox"
              checked={acknowledged}
              onChange={(event) => setAcknowledged(event.target.checked)}
            />
            Draft reviewed locally
          </label>
          <button
            className="primary-action"
            onClick={close}
            disabled={!acknowledged}
          >
            Done with fixture draft
          </button>
        </footer>
      </div>
    </main>
  );
}
