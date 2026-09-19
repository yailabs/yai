import type { LiveWorkspace } from "../clients/live";
import { Icon } from "../components/Icon";
import { EmptyState, IconButton } from "../components/primitives";
import { TerminalPanel } from "../terminal/TerminalPanel";

export type BottomMode = "Terminal" | "Output" | "Executions" | "Evidence" | "Problems";

export function LiveBottomPanel({ workspace, active, setActive, close, height, open, maximized, toggleMaximized }: {
  workspace: LiveWorkspace;
  active: BottomMode;
  setActive: (mode: BottomMode) => void;
  close: () => void;
  height: number;
  open: boolean;
  maximized: boolean;
  toggleMaximized: () => void;
}) {
  const tabs: BottomMode[] = ["Terminal", "Output", "Executions", "Evidence", "Problems"];
  return <section className="live-bottom" style={{ height }} aria-label="Bottom tools" hidden={!open}>
    <header>{tabs.map((tab) => <button key={tab} aria-pressed={active === tab} onClick={() => setActive(tab)}>{tab}</button>)}<span />
      <IconButton aria-label={maximized ? "Restore bottom panel" : "Maximize bottom panel"} title={maximized ? "Restore Panel" : "Maximize Panel"} onClick={toggleMaximized}><Icon name={maximized ? "restore" : "maximize"} size={14} /></IconButton>
      <IconButton aria-label="Close bottom panel" title="Close Panel" onClick={close}><Icon name="close" size={14} /></IconButton>
    </header>
    <div className="tool-content">
      <div className="tool-pane" hidden={active !== "Terminal"}><TerminalPanel /></div>
      {active === "Output" && <div className="tool-pane padded"><pre>application.protocol={"yai.studio.application.v1"}{"\n"}case={workspace.case.case_ref}{"\n"}generation={workspace.case.generation}{"\n"}projection=case.summary</pre></div>}
      {active === "Executions" && <div className="tool-pane padded"><EmptyState title="No execution projection" body="No dedicated execution view is exposed by this bounded host." /></div>}
      {active === "Evidence" && <div className="tool-pane padded"><EmptyState title="No evidence projection" body="Evidence remains linked through the real authority and workflow facts where present." /></div>}
      {active === "Problems" && <div className="tool-pane padded"><EmptyState title="No client problems" body="Unavailable YAI facts are rendered in their owning surface." /></div>}
    </div>
  </section>;
}
