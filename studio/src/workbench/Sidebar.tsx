import { Icon } from "../components/Icon";
import type { IconName } from "../components/Icon";
import type {
  Activity,
  ExplorerGroup,
  ExplorerItem,
  MemoryMode,
  WorkspacePresentation,
} from "../clients/presentation";

export const activities: readonly { name: Activity; icon: IconName }[] = [
  { name: "Overview", icon: "overview" },
  { name: "Environment", icon: "environment" },
  { name: "Knowledge", icon: "knowledge" },
  { name: "Memory", icon: "memory" },
  { name: "Authority", icon: "authority" },
  { name: "Work", icon: "work" },
  { name: "Compute", icon: "compute" },
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
          aria-pressed={active === name}
          onClick={() => select(name)}
        >
          <Icon name={icon} size={20} />
          <span>{name}</span>
        </button>
      ))}
    </nav>
  );
}

function rowIcon(item: ExplorerItem): IconName {
  if (["policy", "review", "decision"].includes(item.kind)) return "authority";
  if (["provider", "model", "machine"].includes(item.kind)) return "compute";
  if (item.kind === "knowledge") return "knowledge";
  if (item.kind === "memory") return "memory";
  if (["workflow", "execution"].includes(item.kind)) return "work";
  if (item.kind === "repository") return "repository";
  if (item.kind === "resource") return "environment";
  return "file";
}

function GroupRows({
  groups,
  selected,
  activate,
}: {
  groups: readonly ExplorerGroup[];
  selected: string;
  activate: (item: ExplorerItem) => void;
}) {
  return (
    <>
      {groups.map((group) => (
        <details open key={group.label}>
          <summary>
            {group.label}
            <span>{group.items.length}</span>
          </summary>
          {group.note && <p className="explorer-group-note">{group.note}</p>}
          {group.items.map((item) => (
            <button
              key={item.id}
              className={`tree-row ${selected === item.id || selected === item.material ? "selected" : ""}`}
              aria-pressed={selected === item.id || selected === item.material}
              onClick={() => activate(item)}
              title={item.detail}
            >
              <Icon name={rowIcon(item)} size={15} />
              <span className="truncate">
                <strong>{item.label}</strong>
                <small>{item.detail}</small>
              </span>
              {item.posture && (
                <span
                  className={`item-posture posture-${item.posture.replaceAll(" ", "-")}`}
                >
                  {item.posture}
                </span>
              )}
            </button>
          ))}
        </details>
      ))}
    </>
  );
}

export function Sidebar({
  data,
  activity,
  selected,
  open,
  inspect,
  setMemoryMode,
}: {
  data: WorkspacePresentation;
  activity: Activity;
  selected: string;
  open: (id: string) => void;
  inspect: (id: string) => void;
  setMemoryMode: (mode: MemoryMode) => void;
}) {
  const info = data.information;
  const groups =
    activity === "Environment"
      ? info.environment
      : activity === "Knowledge"
        ? info.knowledge
        : activity === "Authority"
          ? info.authority
          : activity === "Work"
            ? info.work
            : activity === "Compute"
              ? info.compute
              : [];
  const activate = (item: ExplorerItem) => {
    inspect(item.id);
    if (item.material) open(item.material);
  };
  return (
    <aside
      className="sidebar"
      id="case-sidebar"
      aria-label={`${activity} explorer`}
    >
      <div className="region-heading">
        <span>{activity.toUpperCase()}</span>
      </div>
      <div className="sidebar-scroll">
        {activity === "Overview" && (
          <>
            <details open>
              <summary>
                Participants<span>{data.participants.length}</span>
              </summary>
              <div className="participant-list">
                {data.participants.map((person) => (
                  <button
                    className="person-row"
                    key={person.id}
                    onClick={() => inspect(person.id)}
                  >
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
            </details>
            <details open>
              <summary>Current work</summary>
              <button
                className="tree-row selected"
                onClick={() => open("section:Work")}
              >
                <Icon name="work" size={15} />
                <span className="truncate">
                  <strong>{data.case.currentWork}</strong>
                  <small>{info.overview.status}</small>
                </span>
              </button>
            </details>
            <details open>
              <summary>
                Attention<span>{info.overview.highlights.length}</span>
              </summary>
              {info.overview.highlights.map((item) => (
                <button
                  key={item.id}
                  className="tree-row"
                  onClick={() => activate(item)}
                >
                  <Icon name={rowIcon(item)} size={15} />
                  <span className="truncate">
                    <strong>{item.label}</strong>
                    <small>{item.detail}</small>
                  </span>
                </button>
              ))}
            </details>
          </>
        )}
        {activity === "Memory" && (
          <>
            <details open>
              <summary>Experience</summary>
              <button
                className="tree-row"
                onClick={() => {
                  setMemoryMode("Timeline");
                  open("section:Memory");
                }}
              >
                <Icon name="memory" size={15} />
                <span className="truncate">
                  <strong>Timeline</strong>
                  <small>{info.memory.timeline.length} authored events</small>
                </span>
              </button>
              <button
                className="tree-row"
                onClick={() => {
                  setMemoryMode("Graph");
                  open("section:Memory");
                }}
              >
                <Icon name="graph" size={15} />
                <span className="truncate">
                  <strong>Graph</strong>
                  <small>{info.memory.nodes.length} visible relations</small>
                </span>
              </button>
            </details>
            <details open>
              <summary>Recall posture</summary>
              <div className="sidebar-note">
                <Icon name="memory" size={16} />
                <p>
                  History is an authored projection. Recall reasons and
                  missingness remain a future live query.
                </p>
              </div>
            </details>
          </>
        )}
        <GroupRows groups={groups} selected={selected} activate={activate} />
      </div>
      <div className="sidebar-foot">
        <span className="offline-dot" />
        Offline Case view<span>v2</span>
      </div>
    </aside>
  );
}
