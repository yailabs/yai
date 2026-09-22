import { lazy, Suspense, useEffect, useState } from "react";
const TerminalPanel = lazy(() => import("../../terminal/TerminalPanel").then((module) => ({default: module.TerminalPanel})));
import { EmptyState } from "../../components/primitives";
import type { PanelViewProps } from "../../workbench/kernel/types";

export function TerminalPanelView({ available, visible, platform, toolbarTarget, closePanel }: PanelViewProps) {
  const [opened, setOpened] = useState(visible);
  const [, preferencesChanged] = useState(0);
  useEffect(() => platform.configuration.subscribe(key => { if (key === "terminal.scrollback") preferencesChanged(value => value + 1); }).dispose, [platform.configuration]);
  useEffect(() => { if (visible) setOpened(true); }, [visible]);
  return available ? opened && <Suspense fallback={<p className="surface-loading">Opening terminal…</p>}><TerminalPanel editing={platform.editing} visible={visible} scrollback={platform.configuration.get<number>("terminal.scrollback") ?? 5000} toolbarTarget={toolbarTarget} onEmpty={closePanel} /></Suspense> : <EmptyState title="Terminal requires desktop host" body="The browser Workbench does not create or emulate a shell." />;
}
