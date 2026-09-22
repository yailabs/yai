import { useCallback, useEffect, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { createPortal } from "react-dom";
import "@xterm/xterm/css/xterm.css";
import { Icon } from "../components/Icon";
import { IconButton } from "../components/primitives";
import { Splitter } from "../workbench/kernel/Splitter";

import type { EditingService } from "../platform/editing";
import { terminalEventName, type TerminalCommand } from "./commands";
interface TerminalCreated { terminal_id: string; shell: string; cwd: string }
interface TerminalOutput { terminal_id: string; data: number[] }
interface TerminalExit { terminal_id: string; exit_code: number; signal?: string }
interface TerminalInstance extends TerminalCreated { status: "running" | "exited"; exitCode?: number }

const encoder = new TextEncoder();

export function TerminalPanel({ editing, visible = true, scrollback = 5000, toolbarTarget, onEmpty }: { editing: EditingService; visible?: boolean; scrollback?: number; toolbarTarget?: HTMLElement | null; onEmpty?(): void }) {
  const desktop = Boolean(window.__TAURI__);
  const [terminals, setTerminals] = useState<TerminalInstance[]>([]);
  const [activeId, setActiveId] = useState<string>();
  const [bridgeReady, setBridgeReady] = useState(false);
  const [error, setError] = useState<string>();
  const [instancePaneWidth, setInstancePaneWidth] = useState(176);
  const renderers = useRef(new Map<string, Terminal>());
  const pendingOutput = useRef(new Map<string, Uint8Array[]>());
  const creating = useRef(false);
  const lifecycle = useRef(0);
  const ownedTerminals = useRef(new Set<string>());
  const activeTerminal = terminals.find((terminal) => terminal.terminal_id === activeId);
  const activeTerminalIndex = activeTerminal ? terminals.indexOf(activeTerminal) + 1 : 0;
  const compactInstances = instancePaneWidth < 96;
  const showInstancePane = terminals.length > 1;
  const showActiveToolbar = !showInstancePane || compactInstances;

  const createTerminal = useCallback(async () => {
    if (!window.__TAURI__ || creating.current) return;
    creating.current = true;
    const epoch = lifecycle.current;
    setError(undefined);
    try {
      const created = await window.__TAURI__.core.invoke<TerminalCreated>("terminal_create", { rows: 24, cols: 80 });
      if (epoch !== lifecycle.current) {
        await window.__TAURI__.core.invoke("terminal_kill", { terminalId: created.terminal_id });
        return;
      }
      ownedTerminals.current.add(created.terminal_id);
      setTerminals((current) => [...current, { ...created, status: "running" }]);
      setActiveId(created.terminal_id);
    } catch (reason) {
      setError(String(reason));
    } finally {
      creating.current = false;
    }
  }, []);

  const closeTerminal = useCallback(async (terminalId: string) => {
    const closingLast = terminals.length === 1 && terminals[0]?.terminal_id === terminalId;
    if (window.__TAURI__) {
      try { await window.__TAURI__.core.invoke("terminal_kill", { terminalId }); } catch { /* already exited */ }
    }
    ownedTerminals.current.delete(terminalId);
    renderers.current.get(terminalId)?.dispose();
    renderers.current.delete(terminalId);
    pendingOutput.current.delete(terminalId);
    setTerminals((current) => {
      const next = current.filter((terminal) => terminal.terminal_id !== terminalId);
      setActiveId((active) => active === terminalId ? next.at(-1)?.terminal_id : active);
      return next;
    });
    if (closingLast) onEmpty?.();
  }, [onEmpty, terminals]);

  const runCommand = useCallback((command: TerminalCommand) => {
    if (command === "new") { void createTerminal(); return; }
    if (!activeId) return;
    if (command === "kill") { void closeTerminal(activeId); return; }
    const terminal = renderers.current.get(activeId);
    if (command === "clear") terminal?.clear();
    if (command === "focus") terminal?.focus();
  }, [activeId, closeTerminal, createTerminal]);
  const registerRenderer = useCallback((terminalId: string, renderer: Terminal) => {
    renderers.current.set(terminalId, renderer);
    for (const bytes of pendingOutput.current.get(terminalId) ?? []) renderer.write(bytes);
    pendingOutput.current.delete(terminalId);
  }, []);
  const unregisterRenderer = useCallback((terminalId: string) => renderers.current.delete(terminalId), []);

  useEffect(() => {
    if (!desktop) return;
    let stopOutput: () => void = () => {};
    let stopExit: () => void = () => {};
    let active = true;
    void (async () => {
      const registeredOutput = await window.__TAURI__!.event.listen<TerminalOutput>("yai://terminal-output", ({ payload }) => {
        const bytes = new Uint8Array(payload.data);
        const terminal = renderers.current.get(payload.terminal_id);
        if (terminal) terminal.write(bytes);
        else pendingOutput.current.set(payload.terminal_id, [...(pendingOutput.current.get(payload.terminal_id) ?? []), bytes]);
      });
      if (!active) { registeredOutput(); return; }
      stopOutput = registeredOutput;
      const registeredExit = await window.__TAURI__!.event.listen<TerminalExit>("yai://terminal-exit", ({ payload }) => {
        setTerminals((current) => current.map((terminal) => terminal.terminal_id === payload.terminal_id ? { ...terminal, status: "exited", exitCode: payload.exit_code } : terminal));
        renderers.current.get(payload.terminal_id)?.write(`\r\n\x1b[90m[process exited ${payload.exit_code}${payload.signal ? ` · ${payload.signal}` : ""}]\x1b[0m\r\n`);
      });
      if (!active) { registeredExit(); return; }
      stopExit = registeredExit;
      if (active) setBridgeReady(true);
    })().catch((reason) => {
      if (active) setError(`Terminal event bridge unavailable: ${String(reason)}`);
    });
    return () => {
      active = false;
      lifecycle.current++;
      stopOutput();
      stopExit();
      for (const terminal of renderers.current.values()) terminal.dispose();
      renderers.current.clear();
      for (const terminalId of ownedTerminals.current) void window.__TAURI__?.core.invoke("terminal_kill", { terminalId });
      ownedTerminals.current.clear();
    };
  }, [createTerminal, desktop]);

  useEffect(() => {
    if (visible && bridgeReady && !terminals.length && !error) void createTerminal();
  }, [bridgeReady, createTerminal, error, terminals.length, visible]);

  useEffect(() => {
    const handler = (event: Event) => runCommand((event as CustomEvent<TerminalCommand>).detail);
    window.addEventListener(terminalEventName, handler);
    return () => window.removeEventListener(terminalEventName, handler);
  }, [runCommand]);

  if (!desktop) return <div className="terminal-unavailable"><Icon name="terminal" size={22} /><strong>Terminal requires the desktop host</strong><p>The browser preview does not create or emulate a shell.</p></div>;

  return <section className="terminal-surface" aria-label="Integrated terminals">
    {toolbarTarget && createPortal(<div className="terminal-panel-toolbar" aria-label="Active terminal controls">
      {showActiveToolbar && activeTerminal && <span className="terminal-panel-active" title={`${activeTerminal.shell} · ${activeTerminal.cwd}`}><Icon name="terminal" size={14} /><span>{`${activeTerminal.shell} ${activeTerminalIndex}`}</span></span>}
      <IconButton aria-label="New terminal" title="New Terminal" onClick={() => void createTerminal()}><Icon name="plus" size={15} /></IconButton>
      {showActiveToolbar && <IconButton aria-label="Kill active terminal" title="Kill Terminal" disabled={!activeId} onClick={() => activeId && void closeTerminal(activeId)}><Icon name="trash" size={15} /></IconButton>}
    </div>, toolbarTarget)}
    <div className="terminal-stack">{terminals.map((terminal) => <TerminalViewport editing={editing} key={terminal.terminal_id} terminal={terminal} active={visible && activeId === terminal.terminal_id} register={registerRenderer} unregister={unregisterRenderer} scrollback={scrollback} />)}
      {error && <div className="terminal-empty error"><strong>Terminal unavailable</strong><code>{error}</code><button onClick={() => void createTerminal()}>Retry</button></div>}
    </div>
    {showInstancePane && <><Splitter label="Resize terminal list" axis="x" reverse value={instancePaneWidth} min={38} max={280} set={setInstancePaneWidth} />
    <aside className={`terminal-instance-pane ${compactInstances ? "compact" : ""}`} style={{ width: instancePaneWidth }} aria-label="Terminal instances">
      {!compactInstances && <header><span>Terminals</span></header>}
      <div role="tablist" aria-orientation="vertical">{terminals.map((terminal, index) => <div className="terminal-instance-row" key={terminal.terminal_id} data-active={visible && activeId === terminal.terminal_id}>
        <button role="tab" aria-selected={activeId === terminal.terminal_id} title={`${terminal.shell} · ${terminal.cwd}`} onClick={() => setActiveId(terminal.terminal_id)}><Icon name="terminal" size={14} /><span>{terminal.shell} {index + 1}</span>{terminal.status === "exited" && <small>{terminal.exitCode}</small>}</button>
        <IconButton aria-label={`Kill ${terminal.shell} ${index + 1}`} title="Kill Terminal" onClick={() => void closeTerminal(terminal.terminal_id)}><Icon name="trash" size={14} /></IconButton>
      </div>)}</div>
    </aside></>}
  </section>;
}

function TerminalViewport({ editing, terminal, active, register, unregister, scrollback }: { editing: EditingService; terminal: TerminalInstance; active: boolean; register: (terminalId: string, renderer: Terminal) => void; unregister: (terminalId: string) => void; scrollback: number }) {
  const host = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<Terminal | null>(null);
  const usable = useRef(active && terminal.status === "running");
  usable.current = active && terminal.status === "running";
  useEffect(() => {
    if (!host.current || !window.__TAURI__) return;
    const renderer = new Terminal({
      cursorBlink: true,
      convertEol: false,
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      fontSize: 13,
      lineHeight: 1.16,
      scrollback,
      allowProposedApi: false,
      theme: { background: "#0d0f12", foreground: "#d5d8de", cursor: "#d5d8de", selectionBackground: "#34445a", black: "#17191d", brightBlack: "#666c76" },
    });
    const fit = new FitAddon();
    renderer.loadAddon(fit);
    renderer.open(host.current);
    rendererRef.current = renderer;
    register(terminal.terminal_id, renderer);
    const sendResize = () => {
      if (!host.current || host.current.clientWidth < 20 || host.current.clientHeight < 20) return;
      try { fit.fit(); } catch { return; }
      void window.__TAURI__?.core.invoke("terminal_resize", { terminalId: terminal.terminal_id, rows: renderer.rows, cols: renderer.cols });
    };
    const observer = new ResizeObserver(sendResize);
    observer.observe(host.current);
    let inputVersion = 0;
    const input = renderer.onData((data) => { inputVersion++; void window.__TAURI__?.core.invoke("terminal_write", { terminalId: terminal.terminal_id, data: Array.from(encoder.encode(data)) }); });
    const binary = renderer.onBinary((data) => void window.__TAURI__?.core.invoke("terminal_write", { terminalId: terminal.terminal_id, data: Array.from(data, (character) => character.charCodeAt(0) & 0xff) }));
    const editingTarget = editing.register({
      element: host.current, focus: () => renderer.focus(),
      enabled: command => command === "copy" && renderer.hasSelection() || command === "selectAll" || command === "paste" && usable.current,
      execute: command => { if (command === "selectAll") renderer.selectAll(); },
      selection: () => {
        const version = inputVersion;
        return { text: renderer.getSelection(), current: () => usable.current && rendererRef.current === renderer && inputVersion === version,
          replace: text => renderer.paste(text) };
      },
    });
    const selection = renderer.onSelectionChange(editing.changed);
    renderer.attachCustomKeyEventHandler((event) => {
      const primary = navigator.platform.toLowerCase().includes("mac") ? event.metaKey : event.ctrlKey && event.shiftKey;
      const key = event.key.toLowerCase();
      if (primary && (key === "v" || key === "c" && renderer.hasSelection())) {
        event.preventDefault();
        if (event.type === "keydown") void editing.execute(key === "v" ? "paste" : "copy");
        return false;
      }
      return true;
    });
    requestAnimationFrame(sendResize);
    return () => { editingTarget.dispose(); selection.dispose(); observer.disconnect(); input.dispose(); binary.dispose(); unregister(terminal.terminal_id); rendererRef.current = null; renderer.dispose(); };
  }, [editing, register, terminal.terminal_id, unregister]);
  useEffect(() => editing.changed(), [editing, active, terminal.status]);
  useEffect(() => { if (rendererRef.current) rendererRef.current.options.scrollback = scrollback; }, [scrollback]);
  useEffect(() => { if (active) requestAnimationFrame(() => rendererRef.current?.focus()); }, [active]);
  return <div ref={host} className="terminal-viewport" hidden={!active} data-terminal-id={terminal.terminal_id} onMouseDown={() => rendererRef.current?.focus()} />;
}
