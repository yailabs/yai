import { lazy, Suspense, useEffect, useState } from "react";
const TerminalPanel = lazy(() => import("../../terminal/TerminalPanel").then((module) => ({default: module.TerminalPanel})));
import { EmptyState } from "../../components/primitives";
import type { PanelViewProps } from "../../workbench/kernel/types";

export function TerminalPanelView({ available, visible, platform, toolbarTarget, closePanel }: PanelViewProps) {
  const [opened, setOpened] = useState(visible);
  useEffect(() => { if (visible) setOpened(true); }, [visible]);
  return available ? opened && <Suspense fallback={<p className="surface-loading">Opening terminal…</p>}><TerminalPanel visible={visible} scrollback={platform.configuration.get<number>("terminal.scrollback") ?? 5000} toolbarTarget={toolbarTarget} onEmpty={closePanel} /></Suspense> : <EmptyState title="Terminal requires desktop host" body="The browser Workbench does not create or emulate a shell." />;
}

export function OutputPanelView({ workspace }: PanelViewProps) {
  return <div className="tool-pane padded"><pre>case.data={workspace.presentation.dataKind}{"\n"}backend.posture={workspace.presentation.backendPosture}{"\n"}case={workspace.case.case_ref}{"\n"}generation={workspace.case.generation}{"\n"}projection=case.summary</pre></div>;
}

export function EmptyToolView({ workspace }: PanelViewProps) {
  return <EmptyState title="No projection available" body={`No dedicated tool projection is exposed for ${workspace.case.case_ref}.`} />;
}
