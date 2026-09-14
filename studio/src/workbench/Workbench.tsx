import { useState } from "react";
import type {
  Activity,
  ContextMode,
  MemoryMode,
  WorkspacePresentation,
} from "../clients/presentation";
import type { Layout } from "./layout";
import { Splitter } from "./layout";
import { ActivityBar, Sidebar } from "./Sidebar";
import { WorkSurface } from "./WorkSurface";
import { ContextPanel } from "./ContextPanel";
import { BottomPanel } from "./BottomPanel";

export function Workbench({
  data,
  layout,
  stepId,
}: {
  data: WorkspacePresentation;
  layout: Layout;
  stepId: string;
}) {
  const [activity, setActivity] = useState<Activity>("Overview");
  const [tabs, setTabs] = useState([
    "section:Overview",
    ...data.initial.tabs.slice(0, 2),
  ]);
  const [active, setActive] = useState("section:Overview");
  const [bottomTab, setBottomTab] = useState(data.initial.bottom);
  const [draft, setDraft] = useState("");
  const [contextMode, setContextMode] = useState<ContextMode>("Inspector");
  const [selectedContext, setSelectedContext] = useState("case");
  const [memoryMode, setMemoryMode] = useState<MemoryMode>("Timeline");
  const step =
    data.information.progression.steps.find((item) => item.id === stepId)
      ?.index ?? 1;
  function open(id: string) {
    setTabs((current) => (current.includes(id) ? current : [...current, id]));
    setActive(id);
    if (id.startsWith("section:"))
      setActivity(id.slice("section:".length) as Activity);
    else setSelectedContext(id);
  }
  function inspect(id: string) {
    setSelectedContext(id);
    setContextMode("Inspector");
  }
  function close(id: string) {
    const next = tabs.filter((tab) => tab !== id);
    setTabs(next);
    {
      const selection =
        active === id
          ? (next[Math.min(tabs.indexOf(id), next.length - 1)] ?? "")
          : active;
      setActive(selection);
      requestAnimationFrame(() => {
        if (selection) document.getElementById(`surface-${selection}`)?.focus();
        else
          document
            .querySelector<HTMLButtonElement>(".empty-surface button")
            ?.focus();
      });
    }
  }
  return (
    <div className="workbench">
      <ActivityBar
        active={activity}
        select={(value) => {
          setActivity(value);
          open(`section:${value}`);
          layout.setLeftOpen(true);
        }}
      />
      {layout.leftOpen && (
        <>
          <div className="sidebar-slot" style={{ width: layout.left }}>
            <Sidebar
              data={data}
              activity={activity}
              selected={active}
              open={open}
              inspect={inspect}
              setMemoryMode={setMemoryMode}
            />
          </div>
          <Splitter
            label="Resize Case explorer"
            controls="case-sidebar"
            axis="x"
            value={layout.left}
            min={190}
            max={layout.leftMax}
            onChange={layout.setLeft}
          />
        </>
      )}
      <div className="central-column">
        <WorkSurface
          data={data}
          tabs={tabs}
          active={active}
          memoryMode={memoryMode}
          setMemoryMode={setMemoryMode}
          step={step}
          select={(id) => {
            setActive(id);
            if (id.startsWith("section:"))
              setActivity(id.slice("section:".length) as Activity);
            else setSelectedContext(id);
          }}
          close={close}
          open={open}
          inspect={inspect}
        />
        {layout.bottomOpen && (
          <>
            <Splitter
              label="Resize bottom panel"
              controls="case-tools"
              axis="y"
              reverse
              value={layout.bottom}
              min={140}
              max={layout.bottomMax}
              onChange={layout.setBottom}
            />
            <BottomPanel
              data={data}
              active={bottomTab}
              select={setBottomTab}
              collapse={() => {
                layout.setBottomOpen(false);
                document
                  .querySelector<HTMLButtonElement>(
                    '[aria-label="Toggle bottom panel"]',
                  )
                  ?.focus();
              }}
              open={open}
              height={layout.bottom}
              step={step}
            />
          </>
        )}
      </div>
      {layout.rightOpen && (
        <>
          <Splitter
            label="Resize context panel"
            controls="case-conversation"
            axis="x"
            reverse
            value={layout.right}
            min={260}
            max={layout.rightMax}
            onChange={layout.setRight}
          />
          <div className="conversation-slot" style={{ width: layout.right }}>
            <ContextPanel
              data={data}
              mode={contextMode}
              setMode={setContextMode}
              selected={selectedContext}
              step={step}
              open={open}
              draft={draft}
              setDraft={setDraft}
              collapse={() => {
                layout.setRightOpen(false);
                document
                  .querySelector<HTMLButtonElement>(
                    '[aria-label="Toggle context panel"]',
                  )
                  ?.focus();
              }}
            />
          </div>
        </>
      )}
    </div>
  );
}
