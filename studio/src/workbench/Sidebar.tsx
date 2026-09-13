import { Icon } from "../components/Icon";
import type { IconName } from "../components/Icon";
import type {
  Activity,
  MaterialView,
  WorkspacePresentation,
} from "../clients/presentation";

export const activities: readonly { name: Activity; icon: IconName }[] = [
  { name: "Case", icon: "case" },
  { name: "Sources", icon: "sources" },
  { name: "Files", icon: "file" },
  { name: "Work", icon: "work" },
  { name: "Providers", icon: "provider" },
];
export function ActivityBar({
  active,
  select,
}: {
  active: Activity;
  select: (activity: Activity) => void;
}) {
  return (
    <nav className="activity-bar" aria-label="Case perspectives">
      {activities.map(({ name, icon }) => (
        <button
          key={name}
          aria-label={`${name} perspective`}
          title={name}
          aria-pressed={active === name}
          onClick={() => select(name)}
        >
          <Icon name={icon} size={21} />
          <span>{name}</span>
        </button>
      ))}
      <span className="rail-end" title="Offline fixture workspace">
        F
      </span>
    </nav>
  );
}
export function Sidebar({
  data,
  activity,
  selected,
  open,
}: {
  data: WorkspacePresentation;
  activity: Activity;
  selected: string;
  open: (id: string) => void;
}) {
  function materialRows(materials: readonly MaterialView[]) {
    return materials.map((item) => (
      <button
        key={item.id}
        className={`tree-row ${selected === item.id ? "selected" : ""}`}
        aria-pressed={selected === item.id}
        onClick={() => open(item.id)}
        title={item.path}
      >
        <Icon
          name={
            item.category === "work"
              ? "work"
              : item.category === "provider"
                ? "provider"
                : "file"
          }
          size={16}
        />
        <span className="truncate">{item.name}</span>
        {item.changed && (
          <span className="change-label" aria-label="Fixture changed">
            M
          </span>
        )}
      </button>
    ));
  }
  const sources = data.materials.filter((item) => item.category === "source");
  const artifacts = data.materials.filter(
    (item) => item.category === "artifact",
  );
  return (
    <aside
      className="sidebar"
      id="case-sidebar"
      aria-label={`${activity} explorer`}
    >
      <div className="region-heading">
        <span>
          {activity === "Case" ? "CASE EXPLORER" : activity.toUpperCase()}
        </span>
        <span className="micro">FIXTURE</span>
      </div>
      <div className="sidebar-scroll">
        <div className="case-identity">
          <span className="eyebrow">{data.case.reference}</span>
          <h1>{data.case.label}</h1>
          <p>{data.case.purpose}</p>
        </div>
        {activity === "Case" && (
          <>
            <details open>
              <summary>
                Participants <span>{data.participants.length}</span>
              </summary>
              <div className="participant-list">
                {data.participants.map((person) => (
                  <div className="person-row" key={person.id}>
                    <span
                      className={`avatar ${person.kind === "AI participant" ? "ai" : ""}`}
                    >
                      {person.initials}
                    </span>
                    <div>
                      <strong>{person.name}</strong>
                      <small>
                        {person.kind === "AI participant" ? "AI · " : ""}
                        {person.role}
                      </small>
                    </div>
                  </div>
                ))}
              </div>
            </details>
            <details open>
              <summary>Current work</summary>
              {materialRows(
                data.materials.filter((item) => item.category === "work"),
              )}
            </details>
          </>
        )}
        {(activity === "Case" || activity === "Sources") && (
          <details open>
            <summary>
              Sources <span>{sources.length}</span>
            </summary>
            {materialRows(sources)}
          </details>
        )}
        {(activity === "Case" || activity === "Files") && (
          <details open>
            <summary>
              {activity === "Files" ? "Files & artifacts" : "Artifacts"}{" "}
              <span>{artifacts.length}</span>
            </summary>
            {materialRows(artifacts)}
          </details>
        )}
        {activity === "Work" && (
          <>
            <details open>
              <summary>Work in this Case</summary>
              {materialRows(
                data.materials.filter(
                  (item) =>
                    item.category === "work" || item.format === "Review",
                ),
              )}
            </details>
            <div className="sidebar-note">
              <Icon name="clock" />
              <p>
                Execution states are a frozen fixture. Inspect them in the work
                surface or the Executions panel.
              </p>
            </div>
          </>
        )}
        {activity === "Providers" && (
          <>
            <details open>
              <summary>Inference context</summary>
              {materialRows(
                data.materials.filter((item) => item.category === "provider"),
              )}
            </details>
            <div className="sidebar-note">
              <Icon name="provider" />
              <p>Provider context is illustrative. No runtime is connected.</p>
            </div>
          </>
        )}
      </div>
      <div className="sidebar-foot">
        <span className="offline-dot" />
        Offline workspace<span>v1</span>
      </div>
    </aside>
  );
}
