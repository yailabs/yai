import { useEffect, useRef, useState } from "react";
import type { AuxiliaryViewProps } from "../../workbench/kernel/types";
import { Badge, Button } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { conversationExecutionMessage, conversationStorageKey, makeConversationSend, type ConversationExecution, type ConversationSendInput } from "../../clients/conversation";
import { useApplicationAvailability } from "./applicationActions";
import { ExecutionContext } from "./ExecutionContext";

export function ConversationView(props: AuxiliaryViewProps) {
  return <Conversation key={`${props.workspace.case.case_ref}:${props.workspace.case.participant_ref}`} {...props} />;
}

function Conversation({ workspace, platform, actions }: AuxiliaryViewProps) {
  const application = platform.application;
  const availability = useApplicationAvailability(application);
  const { case_ref: caseRef, participant_ref: participant } = workspace.case;
  const key = conversationStorageKey(caseRef, participant);
  const read = () => { try { return JSON.parse(sessionStorage.getItem(key) ?? "{}"); } catch { return {}; } };
  const [draft, setDraft] = useState<string>(() => typeof read().draft === "string" ? read().draft : "");
  const [pending, setPending] = useState<ConversationSendInput | undefined>(() => {
    const value = read().pending;
    return value?.case_ref === caseRef && value?.participant_ref === participant && Array.isArray(value.parts) ? value : undefined;
  });
  const [executions, setExecutions] = useState<Record<string, ConversationExecution>>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [memorySearchMode, setMemorySearchMode] = useState<"standard" | "fast">("standard");
  const [memorySearchResult, setMemorySearchResult] = useState<string>();
  const [actionsOpen, setActionsOpen] = useState(false);
  const [tick, setTick] = useState(0);
  const log = useRef<HTMLDivElement>(null);
  const composer = useRef<HTMLTextAreaElement>(null);
  const actionsMenu = useRef<HTMLDivElement>(null);
  const followLatest = useRef(true);
  const alive = useRef(true);
  useEffect(() => {
    if (followLatest.current && log.current) log.current.scrollTop = log.current.scrollHeight;
  }, [workspace.conversation.turns, executions]);
  useEffect(() => {
    if (composer.current) { composer.current.style.height = "auto"; composer.current.style.height = `${Math.min(176, composer.current.scrollHeight)}px`; }
  }, [draft]);
  const inFlight = useRef(false);
  const draftRef = useRef(draft); draftRef.current = draft;
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
  useEffect(() => {
    if (!actionsOpen) return;
    const dismiss = (event: PointerEvent) => {
      if (!actionsMenu.current?.contains(event.target as Node)) setActionsOpen(false);
    };
    const escape = (event: KeyboardEvent) => { if (event.key === "Escape") setActionsOpen(false); };
    document.addEventListener("pointerdown", dismiss);
    document.addEventListener("keydown", escape);
    return () => { document.removeEventListener("pointerdown", dismiss); document.removeEventListener("keydown", escape); };
  }, [actionsOpen]);
  const persist = (text: string, intent?: ConversationSendInput) => sessionStorage.setItem(key, JSON.stringify({ draft: text, pending: intent }));
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").catch(() => undefined);
  const supported = Boolean(application?.supports("conversation.send") && application.supports("execution.get"));
  const fastSearchSupported = Boolean(application?.supports("semantic.fast_search.prepare"));
  const hasTarget = workspace.compute.targets.length > 0;
  const assignment = workspace.compute.cognitive_bindings?.find(item => item.participant_id === participant && item.role === "primary");
  const canSend = supported && Boolean(assignment) && hasTarget && workspace.case.case_status === "open";
  const accept = (value: ConversationExecution) => {
    if (value.case_ref !== caseRef || value.participant_ref !== participant) throw new Error("Conversation identity mismatch.");
    setExecutions(previous => ({ ...previous, [value.turn_ref]: value }));
  };

  // Read-only observation recovers committed results even after a new desktop
  // process. Only canonical request refs projected by YAI are observed.
  useEffect(() => {
    if (!application?.supports("execution.get")) return;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const refs = workspace.conversation.turns.slice(-32).filter(turn => turn.execution_request_ref);
    let inputs: Array<Parameters<typeof application.observeConversation>[0]["execution"]> = refs.map(turn => ({ domain: "cognitive_composition" as const, request_ref: turn.execution_request_ref! }));
    if (pending) inputs.push({ domain: "conversation", submission_ref: pending.submission_ref });
    const observe = async () => {
      const active: typeof inputs = [];
      for (const execution of inputs) {
        const result = await application.observeConversation({ case_ref: caseRef, participant_ref: participant, execution });
        if (disposed) return;
        if (result.result_state === "success" && result.data) {
          const value = result.data;
          if (value.case_ref !== caseRef || value.participant_ref !== participant || (execution.domain === "cognitive_composition" ? value.request_ref !== execution.request_ref : value.submission_ref !== execution.submission_ref)) {
            setError("Conversation identity mismatch. Result withheld."); continue;
          }
          accept(value);
          // Unresolved means no active carrier or terminal evidence is known.
          // It is not a queued retry. Events/manual checks may observe it again.
          if (!value.primary_result && ["admitted", "running"].includes(value.posture)) active.push(execution);
          if (value.observed_generation > workspace.case.generation) void refresh();
          if (execution.domain === "conversation" && pending) {
            // The canonical Turn proves acknowledgement, including after a lost response.
            const sent = new TextDecoder().decode(new Uint8Array(pending.parts[0].bytes));
            const nextDraft = draftRef.current === sent ? "" : draftRef.current;
            try { persist(nextDraft); setDraft(nextDraft); setPending(undefined); setError(undefined); } catch { setError("Cannot update local delivery receipt."); }
          }
        } else if (result.result_state === "unauthorized") {
          setExecutions(previous => Object.fromEntries(Object.entries(previous).filter(([, value]) => execution.domain === "cognitive_composition" ? value.request_ref !== execution.request_ref : value.submission_ref !== execution.submission_ref)));
          setError(result.error?.safe_message ?? "Execution is no longer visible to this Participant.");
        } else if (result.result_state === "transport_unavailable" || result.result_state === "stale") active.push(execution);
      }
      inputs = active;
      if (!disposed && active.length) timer = setTimeout(() => void observe(), 1500);
    };
    void observe();
    return () => { disposed = true; clearTimeout(timer); };
  }, [application, availability, caseRef, participant, workspace.case.generation, workspace.conversation.turns, pending, tick]);

  const submit = async (retry = false) => {
    if (!application || inFlight.current || !canSend) return;
    inFlight.current = true; setBusy(true); setError(undefined);
    try {
      const input = retry && pending ? pending : makeConversationSend(caseRef, participant, workspace.conversation.turns.at(-1)?.thread_ref, workspace.case.generation, draft, memorySearchMode);
      // Persist the exact envelope before dispatch. A timeout cannot create a new Turn on retry.
      persist(draft, input); setPending(input);
      const result = await application.sendConversation(input);
      if (!alive.current) return;
      if (result.result_state === "success" && result.data) {
        if (result.data.execution.submission_ref !== input.submission_ref) throw new Error("Conversation submission identity mismatch.");
        if (result.data.memory_search) {
          setMemorySearchResult(result.data.memory_search.requested === "fast" && result.data.memory_search.effective === "standard"
            ? "Fast Search unavailable; YAI used qualified standard memory."
            : result.data.memory_search.effective === "fast" ? "Fast Search active for this request." : undefined);
        }
        accept(result.data.execution);
        const sent = new TextDecoder().decode(new Uint8Array(input.parts[0].bytes));
        const next = draftRef.current === sent ? "" : draftRef.current;
        persist(next); setDraft(next); setPending(undefined); void refresh(); setTick(value => value + 1);
      } else if (["transport_unavailable", "error"].includes(result.result_state)) {
        setError(result.error?.safe_message ?? "Delivery acknowledgement unavailable. Observe or retry this exact submission.");
      } else {
        persist(draftRef.current); setPending(undefined);
        setError(result.error?.safe_message ?? `Send refused: ${result.result_state}.`);
        if (result.result_state === "stale") void refresh();
      }
    } catch (failure) { if (alive.current) setError(failure instanceof Error ? failure.message : "Delivery acknowledgement unavailable."); }
    finally { inFlight.current = false; if (alive.current) setBusy(false); }
  };

  return <div className="conversation-view conversation-operational">
    <div className="conversation-model-bar"><button type="button" className="conversation-model-choice" onClick={() => actions.openPerspective("Compute")} title="Configure the conversation model"><Icon name="compute" size={14} /><span>{assignment ? "Conversation model" : "Choose a model"}</span><Icon name="chevron" size={12} /></button></div>
    <div ref={log} onScroll={() => { const node = log.current; if (node) followLatest.current = node.scrollHeight - node.scrollTop - node.clientHeight < 64; }} className="conversation-messages" role="log" aria-label="Case conversation" aria-live="polite">
      {!workspace.conversation.turns.length && <div className="conversation-welcome"><span className="conversation-monogram" aria-hidden="true">YAI</span><h2>Let’s work on this Case.</h2><p>Ask a question. Explore an idea.</p></div>}
      {workspace.conversation.turns.map(turn => { const execution = executions[turn.id]; return <div className="conversation-exchange" key={turn.id}>
        <article className="real-turn turn-human"><header><strong>{turn.participant_ref === participant ? "You" : turn.participant_ref}</strong><small title={`Committed at Case state version ${turn.generation}`}>Committed</small></header>{turn.parts.map((part, index) => <p key={index}>{part.text ?? `[${part.modality} · ${part.media_type}]`}</p>)}</article>
        {execution && <article className="real-turn turn-ai"><header><strong>Model</strong>{!execution.primary_result && <Badge tone={ ["admitted", "running"].includes(execution.posture) ? "info" : "warning"}>{execution.posture.replaceAll("_", " ")}</Badge>}</header>
          <p>{conversationExecutionMessage(execution)}</p>
          {execution.posture === "unresolved" && <Button type="button" onClick={() => setTick(value => value + 1)}>Check status</Button>}
          <details><summary>Execution details</summary><code>{execution.request_ref}</code>{execution.primary_result && <code>{execution.primary_result.result_id}</code>}{execution.attempt_outcomes.map((outcome, index) => <pre key={index}>{JSON.stringify(outcome, null, 2)}</pre>)}{application && <ExecutionContext application={application} execution={execution} generation={workspace.case.generation} />}</details>
        </article>}
      </div>; })}
    </div>
    <form className="conversation-composer" onSubmit={event => { event.preventDefault(); if (!pending) void submit(); }}>
      {hasTarget && !assignment && <p>Assign a conversation model in <button type="button" className="object-link" onClick={() => actions.openPerspective("Compute")}>Compute</button>.</p>}
      {!hasTarget && <p>Connect a model in <button type="button" className="object-link" onClick={() => actions.openPerspective("Compute")}>Compute</button> to send messages.</p>}
      {!supported && <p>{application?.reason("conversation.send") ?? "Sending requires the native YAI Host."}</p>}
      <div className="conversation-input-shell"><textarea ref={composer} aria-label="Message to the Case" placeholder="Ask about this Case…" value={draft} rows={1} maxLength={65536} onChange={event => { const text = event.target.value; setDraft(text); try { persist(text, pending); } catch { setError("Local draft storage is unavailable."); } }} onKeyDown={event => { if ((event.ctrlKey || event.metaKey) && event.key === "Enter") { event.preventDefault(); if (!pending) void submit(); } }} />
      </div>
      {error && <p role="alert" className="conversation-error">{error}</p>}
      {memorySearchResult && <p className="conversation-memory-result" role="status">{memorySearchResult}</p>}
      {pending ? <div className="object-action-row"><Button type="button" disabled={busy || !supported} onClick={() => setTick(value => value + 1)}>Check delivery</Button><Button type="button" disabled={busy || !canSend} onClick={() => void submit(true)}>Retry exact submission</Button></div> : <div className="conversation-send-row">
        <div className="conversation-composer-actions" ref={actionsMenu}>
          <button type="button" className="conversation-plus" aria-label="Conversation tools" aria-expanded={actionsOpen} aria-controls="conversation-tools-menu" onClick={() => setActionsOpen(value => !value)}><Icon name="plus" size={18} /></button>
          {actionsOpen && <div className="conversation-tools-menu" id="conversation-tools-menu" role="menu" aria-label="Conversation tools">
            <div className="conversation-tools-heading">Memory search</div>
            <button type="button" role="menuitemradio" aria-checked={memorySearchMode === "standard"} onClick={() => { setMemorySearchMode("standard"); setActionsOpen(false); }}><Icon name="memory" size={17} /><span><strong>Standard</strong><small>Qualified Recall and Working State</small></span>{memorySearchMode === "standard" && <Icon name="check" size={15} />}</button>
            <button type="button" role="menuitemradio" aria-checked={memorySearchMode === "fast"} disabled={!fastSearchSupported} onClick={() => { setMemorySearchMode("fast"); setActionsOpen(false); }}><Icon name="search" size={17} /><span><strong>Fast Search</strong><small>{fastSearchSupported ? "Optional System Model; standard fallback if unavailable" : "Not supported by this YAI Host"}</small></span>{memorySearchMode === "fast" && <Icon name="check" size={15} />}</button>
            <div className="conversation-tools-divider" />
            <button type="button" role="menuitem" onClick={() => { setActionsOpen(false); actions.openPerspective("Memory"); }}><Icon name="memory" size={17} /><span><strong>Open Memory</strong><small>Recall, Working State and history</small></span></button>
            <button type="button" role="menuitem" onClick={() => { setActionsOpen(false); actions.openPerspective("Environment"); }}><Icon name="sources" size={17} /><span><strong>Browse sources</strong><small>Case sources and resources</small></span></button>
          </div>}
        </div>
        <small className="conversation-memory-mode">{memorySearchMode === "fast" ? "Fast Search · fallback available" : "Ctrl/⌘ Enter"}</small>
        <Button className="conversation-send" aria-label={busy ? "Sending…" : "Send"} title="Send message" type="submit" disabled={busy || !canSend || !draft.trim()}><span aria-hidden="true">{busy ? "…" : "↑"}</span></Button>
      </div>}
    </form>
  </div>;
}
