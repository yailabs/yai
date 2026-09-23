import { useEffect, useRef, useState } from "react";
import type { AuxiliaryViewProps } from "../../workbench/kernel/types";
import { Badge, Button } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { conversationExecutionMessage, conversationStorageKey, makeConversationSend, type ConversationExecution, type ConversationSendInput } from "../../clients/conversation";
import { useApplicationAvailability } from "./applicationActions";

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
  const [tick, setTick] = useState(0);
  const log = useRef<HTMLDivElement>(null);
  const composer = useRef<HTMLTextAreaElement>(null);
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
  const persist = (text: string, intent?: ConversationSendInput) => sessionStorage.setItem(key, JSON.stringify({ draft: text, pending: intent }));
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").catch(() => undefined);
  const supported = Boolean(application?.supports("conversation.send") && application.supports("execution.get"));
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
    const observe = async () => {
      let active = false;
      const refs = workspace.conversation.turns.slice(-32).filter(turn => turn.execution_request_ref);
      const queries = refs.map(turn => ({ domain: "cognitive_composition" as const, request_ref: turn.execution_request_ref! }));
      const inputs: Array<Parameters<typeof application.observeConversation>[0]["execution"]> = [...queries];
      if (pending) inputs.push({ domain: "conversation", submission_ref: pending.submission_ref });
      for (const execution of inputs) {
        const result = await application.observeConversation({ case_ref: caseRef, participant_ref: participant, execution });
        if (disposed) return;
        if (result.result_state === "success" && result.data) {
          const value = result.data;
          if (value.case_ref !== caseRef || value.participant_ref !== participant || (execution.domain === "cognitive_composition" ? value.request_ref !== execution.request_ref : value.submission_ref !== execution.submission_ref)) {
            setError("Conversation identity mismatch. Result withheld."); continue;
          }
          accept(value);
          active ||= ["admitted", "running", "unresolved"].includes(value.posture);
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
        } else if (result.result_state === "transport_unavailable" || result.result_state === "stale") active = true;
      }
      if (!disposed && active) timer = setTimeout(() => void observe(), 1500);
    };
    void observe();
    return () => { disposed = true; clearTimeout(timer); };
  }, [application, availability, caseRef, participant, workspace.case.generation, workspace.conversation.turns, pending, tick]);

  const submit = async (retry = false) => {
    if (!application || inFlight.current || !canSend) return;
    inFlight.current = true; setBusy(true); setError(undefined);
    try {
      const input = retry && pending ? pending : makeConversationSend(caseRef, participant, workspace.conversation.turns.at(-1)?.thread_ref, workspace.case.generation, draft);
      // Persist the exact envelope before dispatch. A timeout cannot create a new Turn on retry.
      persist(draft, input); setPending(input);
      const result = await application.sendConversation(input);
      if (!alive.current) return;
      if (result.result_state === "success" && result.data) {
        if (result.data.execution.submission_ref !== input.submission_ref) throw new Error("Conversation submission identity mismatch.");
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
        <article className="real-turn turn-human"><header><strong>{turn.participant_ref === participant ? "You" : turn.participant_ref}</strong><small title={`Committed at generation ${turn.generation}`}>Committed</small></header>{turn.parts.map((part, index) => <p key={index}>{part.text ?? `[${part.modality} · ${part.media_type}]`}</p>)}</article>
        {execution && <article className="real-turn turn-ai"><header><strong>Model</strong>{!execution.primary_result && <Badge tone={ ["admitted", "running"].includes(execution.posture) ? "info" : "warning"}>{execution.posture.replaceAll("_", " ")}</Badge>}</header>
          <p>{conversationExecutionMessage(execution)}</p>
          <details><summary>Execution details</summary><code>{execution.request_ref}</code>{execution.primary_result && <code>{execution.primary_result.result_id}</code>}{execution.attempt_outcomes.map((outcome, index) => <pre key={index}>{JSON.stringify(outcome, null, 2)}</pre>)}</details>
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
      {pending ? <div className="object-action-row"><Button type="button" disabled={busy || !supported} onClick={() => setTick(value => value + 1)}>Check delivery</Button><Button type="button" disabled={busy || !canSend} onClick={() => void submit(true)}>Retry exact submission</Button></div> : <div className="conversation-send-row"><small title="Draft stays on this device until sent">Ctrl/⌘ Enter</small><Button className="conversation-send" aria-label={busy ? "Sending…" : "Send"} title="Send message" type="submit" disabled={busy || !canSend || !draft.trim()}><span aria-hidden="true">{busy ? "…" : "↑"}</span></Button></div>}
    </form>
  </div>;
}
