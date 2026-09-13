import { useState } from "react";
import type { Activity, WorkspacePresentation } from "../clients/presentation";
import type { Layout } from "./layout";
import { Splitter } from "./layout";
import { ActivityBar, Sidebar } from "./Sidebar";
import { WorkSurface } from "./WorkSurface";
import { Conversation } from "./Conversation";
import { BottomPanel } from "./BottomPanel";

export function Workbench({
  data,
  layout,
}: {
  data: WorkspacePresentation;
  layout: Layout;
}) {
  const [activity, setActivity] = useState<Activity>("Case");
  const [tabs, setTabs] = useState([...data.initial.tabs]);
  const [active, setActive] = useState(data.initial.active);
  const [bottomTab, setBottomTab] = useState(data.initial.bottom);
  const [draft, setDraft] = useState("");
  function open(id: string) {
    setTabs((current) => (current.includes(id) ? current : [...current, id]));
    setActive(id);
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
          select={setActive}
          close={close}
          open={open}
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
            />
          </>
        )}
      </div>
      {layout.rightOpen && (
        <>
          <Splitter
            label="Resize conversation"
            controls="case-conversation"
            axis="x"
            reverse
            value={layout.right}
            min={260}
            max={layout.rightMax}
            onChange={layout.setRight}
          />
          <div className="conversation-slot" style={{ width: layout.right }}>
            <Conversation
              data={data}
              open={open}
              draft={draft}
              setDraft={setDraft}
              collapse={() => {
                layout.setRightOpen(false);
                document
                  .querySelector<HTMLButtonElement>(
                    '[aria-label="Toggle conversation"]',
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
