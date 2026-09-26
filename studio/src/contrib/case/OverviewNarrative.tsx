import { lazy, Suspense, useEffect, useRef, useState } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { PlatformServices } from "../../platform/services";
import { makeConversationSend, conversationExecutionMessage, type ConversationSendInput, type ConversationExecution } from "../../clients/conversation";
import { Button, Badge } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";
import { ExecutionContext } from "./ExecutionContext";

const NarrativeText = lazy(() => import("./NarrativeText"));

interface NarrativePointer {
  case_ref: string;
  participant_ref: string;
  submission_ref: string;
  expected_generation: number;
}

export function OverviewNarrative(props: { workspace: CasePresentation; platform: PlatformServices; configure(): void; openWorkingState(): void; inspect(ref: string): void }) {
  return <Narrative key={`${props.workspace.case.case_ref}:${props.workspace.case.participant_ref}`} {...props}/>;
}

function Narrative({ workspace, platform, configure, openWorkingState, inspect }: { workspace: CasePresentation; platform: PlatformServices; configure(): void; openWorkingState(): void; inspect(ref: string): void }) {
  const app = platform.application;
  const availability = useApplicationAvailability(app);
  const {case_ref, participant_ref, generation} = workspace.case;
  const key = `yai.studio.narrative.v1:${case_ref}:${participant_ref}`;
  const pointerKey = `yai.studio.narrative.pointer.v1:${case_ref}:${participant_ref}`;
  const [request, setRequest] = useState<ConversationSendInput | undefined>(() => {
    try {
      const value = JSON.parse(sessionStorage.getItem(key) ?? "null");
      return value?.case_ref === case_ref && value?.participant_ref === participant_ref && typeof value?.submission_ref === "string" && Array.isArray(value?.parts) ? value : undefined;
    } catch { return undefined; }
  });
  const [pointer, setPointer] = useState<NarrativePointer | undefined>(() => {
    try {
      const value = JSON.parse(localStorage.getItem(pointerKey) ?? "null");
      if (value?.case_ref === case_ref && value?.participant_ref === participant_ref && typeof value?.submission_ref === "string" && value.submission_ref.length > 0 && Number.isSafeInteger(value?.expected_generation) && value.expected_generation >= 0) return value;
    } catch { /* Local persistence may be unavailable; the current window still works. */ }
    return request && {case_ref, participant_ref, submission_ref: request.submission_ref, expected_generation: request.expected_generation};
  });
  useEffect(() => {
    if (!pointer) return;
    try { localStorage.setItem(pointerKey, JSON.stringify(pointer)); } catch { /* Keep the in-memory pointer for this window. */ }
  }, [pointer, pointerKey]);
  const submissionRef = request?.submission_ref ?? pointer?.submission_ref;
  const [result, setResult] = useState<ConversationExecution>();
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [observed, setObserved] = useState<number>();
  const [tick, setTick] = useState(0);
  const alive = useRef(true), dispatching = useRef(false);
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
  const supported = app?.supports("conversation.send") && app.supports("execution.get");
  const assigned = workspace.compute.targets.length > 0 && workspace.compute.cognitive_bindings?.some(binding => binding.participant_id === participant_ref && binding.role === "primary");
  const modelContextAdmitted = workspace.overview.participants.find(item => item.id === participant_ref && item.is_current)?.model_context_admitted;
  const canGenerate = supported && assigned && workspace.case.case_status === "open" && modelContextAdmitted === true;
  // Restored requests are observed through current authority; model text is never cached locally.
  useEffect(() => {
    setResult(undefined);
    if (!submissionRef || !app?.supports("execution.get")) return;
    let disposed = false; let timer: ReturnType<typeof setTimeout>;
    const observe = async () => {
      const response = await app.observeConversation({case_ref, participant_ref, execution:{domain:"conversation", submission_ref:submissionRef}});
      if (disposed) return;
      const value = response.data;
      if (response.result_state === "success" && value && value.case_ref === case_ref && value.participant_ref === participant_ref && value.submission_ref === submissionRef) {
        setResult(value); setError(undefined); setObserved(Date.now());
        if (!value.primary_result && ["admitted", "running"].includes(value.posture)) timer = setTimeout(() => void observe(), 2000);
      } else {
        setResult(undefined); setError(response.error?.safe_message ?? "Narrative observation unavailable or identity mismatch.");
      }
    };
    void observe();
    return () => { disposed = true; clearTimeout(timer); };
  }, [app, availability, case_ref, participant_ref, submissionRef, generation, tick]);
  const submit = async (retry: boolean) => {
    if (!app || !canGenerate || dispatching.current) return;
    dispatching.current = true; setBusy(true); setError(undefined);
    try {
      const input = retry && request ? request : makeConversationSend(case_ref, participant_ref, undefined, generation,
        "Spiega in italiano la storia operativa di questo Case per un essere umano. Usa solo le evidenze del Case effettivamente incluse nel tuo contesto. Organizza: scopo e perimetro; situazione attuale; cosa è successo; workflow e dipendenze; policy e Review; risorse e servizi; risultati verificati; blocchi, dati mancanti e prossimi passi. Cita i riferimenti esatti disponibili a supporto delle affermazioni. Distingui fatti, interpretazioni e raccomandazioni. Non inventare disponibilità, consumi o attività completate. Se il contesto non basta dichiaralo. Non eseguire azioni: questa richiesta produce soltanto una spiegazione candidata, non una decisione o autorizzazione.");
      sessionStorage.setItem(key, JSON.stringify(input)); setRequest(input); setResult(undefined);
      const nextPointer = {case_ref, participant_ref, submission_ref:input.submission_ref, expected_generation:input.expected_generation};
      try { localStorage.setItem(pointerKey, JSON.stringify(nextPointer)); } catch { /* The canonical Turn remains discoverable in Conversation. */ }
      setPointer(nextPointer);
      const response = await app.sendConversation(input);
      if (!alive.current) return;
      if (response.result_state !== "success") {
        setError(response.error?.safe_message ?? `Request ${response.result_state}. Check the exact submission before retrying.`);
        if (["unauthorized", "stale", "not_implemented"].includes(response.result_state)) {
          sessionStorage.removeItem(key); setRequest(undefined); setPointer(undefined);
          try { localStorage.removeItem(pointerKey); } catch { /* No local pointer was persisted. */ }
        }
      }
      setTick(value => value + 1);
      await platform.commands.executeCommand("studio.case.refresh");
    } catch (failure) { if (alive.current) setError(failure instanceof Error ? failure.message : "Acknowledgement unavailable"); }
    finally { dispatching.current = false; if (alive.current) setBusy(false); }
  };
  const unfinished = submissionRef && (!result || ["admitted", "running", "unresolved", "delivery_indeterminate"].includes(result.posture));
  return <section className="overview-narrative" aria-label="Model explanation">
    <header><div><small>Model interpretation · not operational authority</small><h2>The story of this Case</h2></div>{result && <Badge>{result.posture.replaceAll("_", " ")}</Badge>}</header>
    {!result && <p>Generate a grounded explanation of the purpose, history, workflow, evidence and remaining obstacles. The request uses the Case’s governed Conversation path and is retained in its history.</p>}
    {!assigned && <p>A primary conversation model must be bound before generating the explanation. <button className="object-link" onClick={configure}>Configure in Compute</button></p>}
    {assigned && modelContextAdmitted !== true && <p>Model context needs admission before YAI can share Case evidence with the model. <button type="button" className="object-link" onClick={openWorkingState}>Open Working State</button></p>}
    {!supported && <p className="surface-note">{app?.reason("conversation.send") ?? "Native YAI Host required."}</p>}
    {result && <div className="narrative-text">{result.primary_result ? <Suspense fallback={<p>Rendering explanation…</p>}><NarrativeText text={result.primary_result.output} inspect={inspect} references={[workspace.case.case_ref, ...workspace.environment.sources.map(item => item.id), ...workspace.environment.resources.map(item => item.id), ...workspace.environment.files.map(item => item.id), ...workspace.knowledge.units.map(item => item.id), ...workspace.authority.policies.map(item => item.id), ...workspace.work.nodes.map(item => item.node_id), ...workspace.memory.timeline.map(item => item.id)]}/></Suspense> : conversationExecutionMessage(result)}</div>}
    {result?.primary_result && <p className="surface-note">Response observed {observed ? new Date(observed).toLocaleString() : "now"}. Verify its claims against the cited evidence.{pointer && generation !== pointer.expected_generation ? " The Case has changed since this request, including its execution history; this explanation is not a live snapshot." : ""}</p>}
    {error && <p role="alert">{error}</p>}
    <div className="object-action-row"><Button disabled={!canGenerate || busy || Boolean(unfinished)} onClick={() => void submit(false)}>{submissionRef ? "Generate a new explanation" : "Generate explanation"}</Button>{submissionRef && <Button disabled={busy || !supported} onClick={() => setTick(value => value + 1)}>Check explanation</Button>}{request && !result && <Button disabled={!canGenerate || busy} onClick={() => void submit(true)}>Retry exact explanation request</Button>}</div>
    {result && app && <details><summary>Evidence and execution context</summary><code>{result.request_ref}</code>{result.primary_result && <code>{result.primary_result.result_id}</code>}<ExecutionContext application={app} execution={result} generation={generation}/></details>}
  </section>;
}
