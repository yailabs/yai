import { useCallback, useEffect, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { Icon } from "../components/Icon";
import { IconButton } from "../components/primitives";

type TerminalCommand = "new" | "kill" | "clear" | "focus";
interface TerminalCreated { terminal_id: string; shell: string; cwd: string }
interface TerminalOutput { terminal_id: string; data: number[] }
interface TerminalExit { terminal_id: string; exit_code: number; signal?: string }
interface TerminalInstance extends TerminalCreated { status: "running" | "exited"; exitCode?: number }

const terminalEventName = "yai:terminal-command";
const encoder = new TextEncoder();

export function dispatchTerminalCommand(command: TerminalCommand) {
  window.dispatchEvent(new CustomEvent(terminalEventName, { detail: command }));
}

export function TerminalPanel() {
  const desktop = Boolean(window.__TAURI__);
  const [terminals, setTerminals] = useState<TerminalInstance[]>([]);
  const [activeId, setActiveId] = useState<string>();
  const [error, setError] = useState<string>();
  const renderers = useRef(new Map<string, Terminal>());
  const pendingOutput = useRef(new Map<string, Uint8Array[]>());
  const creating = useRef(false);

  const createTerminal = useCallback(async () => {
    if (!window.__TAURI__ || creating.current) return;
    creating.current = true;
    setError(undefined);
    try {
      const created = await window.__TAURI__.core.invoke<TerminalCreated>("terminal_create", { rows: 24, cols: 80 });
      setTerminals((current) => [...current, { ...created, status: "running" }]);
      setActiveId(created.terminal_id);
    } catch (reason) {
      setError(String(reason));
    } finally {
      creating.current = false;
    }
  }, []);

  const closeTerminal = useCallback(async (terminalId: string) => {
    if (window.__TAURI__) {
      try { await window.__TAURI__.core.invoke("terminal_kill", { terminalId }); } catch { /* already exited */ }
    }
    renderers.current.get(terminalId)?.dispose();
    renderers.current.delete(terminalId);
    pendingOutput.current.delete(terminalId);
    setTerminals((current) => {
      const next = current.filter((terminal) => terminal.terminal_id !== terminalId);
      setActiveId((active) => active === terminalId ? next.at(-1)?.terminal_id : active);
      return next;
    });
  }, []);

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
      if (active) await createTerminal();
    })().catch((reason) => {
      if (active) setError(`Terminal event bridge unavailable: ${String(reason)}`);
    });
    return () => {
      active = false;
      stopOutput();
      stopExit();
      for (const terminal of renderers.current.values()) terminal.dispose();
      renderers.current.clear();
      void window.__TAURI__?.core.invoke("terminal_dispose_all");
    };
  }, [createTerminal, desktop]);

  useEffect(() => {
    const handler = (event: Event) => runCommand((event as CustomEvent<TerminalCommand>).detail);
    window.addEventListener(terminalEventName, handler);
    return () => window.removeEventListener(terminalEventName, handler);
  }, [runCommand]);

  if (!desktop) return <div className="terminal-unavailable"><Icon name="terminal" size={22} /><strong>Terminal requires the desktop host</strong><p>The browser preview does not create or emulate a shell.</p></div>;

  return <section className="terminal-surface" aria-label="Integrated terminals">
    <header className="terminal-instance-bar">
      <div role="tablist" aria-label="Terminal instances">{terminals.map((terminal, index) => <button key={terminal.terminal_id} role="tab" aria-selected={activeId === terminal.terminal_id} title={terminal.cwd} onClick={() => setActiveId(terminal.terminal_id)}><Icon name="terminal" size={14} /><span>{terminal.shell} {index + 1}</span>{terminal.status === "exited" && <small>{terminal.exitCode}</small>}</button>)}</div>
      <IconButton aria-label="New terminal" title="New Terminal" onClick={() => void createTerminal()}><Icon name="plus" size={15} /></IconButton>
      <IconButton aria-label="Kill active terminal" title="Kill Terminal" disabled={!activeId} onClick={() => activeId && void closeTerminal(activeId)}><Icon name="trash" size={15} /></IconButton>
    </header>
    <div className="terminal-stack">{terminals.map((terminal) => <TerminalViewport key={terminal.terminal_id} terminal={terminal} active={activeId === terminal.terminal_id} register={registerRenderer} unregister={unregisterRenderer} />)}
      {!terminals.length && !error && <div className="terminal-empty"><strong>No terminal</strong><button onClick={() => void createTerminal()}>New Terminal</button></div>}
      {error && <div className="terminal-empty error"><strong>Terminal unavailable</strong><code>{error}</code><button onClick={() => void createTerminal()}>Retry</button></div>}
    </div>
  </section>;
}

function TerminalViewport({ terminal, active, register, unregister }: { terminal: TerminalInstance; active: boolean; register: (terminalId: string, renderer: Terminal) => void; unregister: (terminalId: string) => void }) {
  const host = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<Terminal | null>(null);
  useEffect(() => {
    if (!host.current || !window.__TAURI__) return;
    const renderer = new Terminal({
      cursorBlink: true,
      convertEol: false,
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      fontSize: 13,
      lineHeight: 1.16,
      scrollback: 5000,
      allowProposedApi: false,
      theme: { background: "#0d0f12", foreground: "#d5d8de", cursor: "#d5d8de", selectionBackground: "#34445a", black: "#17191d", brightBlack: "#666c76" },
    });
    const fit = new FitAddon();
    renderer.loadAddon(fit);
    renderer.open(host.current);
    rendererRef.current = renderer;
    renderer.writeln(`\x1b[90mLocal PTY · ${terminal.shell} · ${terminal.cwd}\x1b[0m`);
    register(terminal.terminal_id, renderer);
    const sendResize = () => {
      if (!host.current || host.current.clientWidth < 20 || host.current.clientHeight < 20) return;
      try { fit.fit(); } catch { return; }
      void window.__TAURI__?.core.invoke("terminal_resize", { terminalId: terminal.terminal_id, rows: renderer.rows, cols: renderer.cols });
    };
    const observer = new ResizeObserver(sendResize);
    observer.observe(host.current);
    const input = renderer.onData((data) => void window.__TAURI__?.core.invoke("terminal_write", { terminalId: terminal.terminal_id, data: Array.from(encoder.encode(data)) }));
    const binary = renderer.onBinary((data) => void window.__TAURI__?.core.invoke("terminal_write", { terminalId: terminal.terminal_id, data: Array.from(data, (character) => character.charCodeAt(0) & 0xff) }));
    renderer.attachCustomKeyEventHandler((event) => {
      const primary = navigator.platform.toLowerCase().includes("mac") ? event.metaKey : event.ctrlKey && event.shiftKey;
      if (primary && event.key.toLowerCase() === "c" && renderer.hasSelection()) {
        void navigator.clipboard.writeText(renderer.getSelection());
        return false;
      }
      if (primary && event.key.toLowerCase() === "v") {
        void navigator.clipboard.readText().then((text) => renderer.paste(text));
        return false;
      }
      return true;
    });
    requestAnimationFrame(sendResize);
    return () => { observer.disconnect(); input.dispose(); binary.dispose(); unregister(terminal.terminal_id); rendererRef.current = null; renderer.dispose(); };
  }, [register, terminal.terminal_id, unregister]);
  useEffect(() => { if (active) requestAnimationFrame(() => rendererRef.current?.focus()); }, [active]);
  return <div ref={host} className="terminal-viewport" hidden={!active} data-terminal-id={terminal.terminal_id} onMouseDown={() => rendererRef.current?.focus()} />;
}
