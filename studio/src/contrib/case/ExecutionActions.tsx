import { useEffect, useRef, useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { ExecutionObservation, ExecutionReference, ResourceAction } from "../../clients/execution";
import { executionKey, readExecutionRefs, rememberExecution } from "../../clients/execution";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button } from "../../components/primitives";
import { ConversationAttempt } from "./ConversationAttempt";
import { useApplicationAvailability } from "./applicationActions";

type Context = Pick<SurfaceRendererProps, "workspace" | "platform">;
const refresh = (platform: Context["platform"]) => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
const keyFor = (workspace: Context["workspace"]) => executionKey(workspace.case.case_ref, workspace.case.participant_ref);
const label = (ref: ExecutionReference) => ref.domain === "source_acquisition" ? `${ref.source_ref} · attempt ${ref.attempt}` : ref.domain === "cognitive_realization" ? ref.plan_ref : ref.domain === "controlled_effect" ? ref.operation_ref : ref.submission_ref;

export function ExecutionHistory({ workspace, platform }: Context) {
  const key = keyFor(workspace);
  const [refs, setRefs] = useState(() => readExecutionRefs(key));
  const [manual, setManual] = useState(false);
  const [domain, setDomain] = useState<ExecutionReference["domain"]>("runtime_work");
  useEffect(() => { setRefs(readExecutionRefs(key)); const update = () => setRefs(readExecutionRefs(key)); window.addEventListener("yai:execution-reference", update); return () => window.removeEventListener("yai:execution-reference", update); }, [key]);
  return <section className="work-section execution-history"><h2>Executions</h2><p>Exact submission references retained in this window. Observation rechecks current YAI authority and never dispatches work. This is not a complete execution catalog.</p><Button onClick={() => setManual(!manual)}>Observe exact execution…</Button>
    {manual && <form className="execution-observe-form" onSubmit={event => { event.preventDefault(); const form = new FormData(event.currentTarget); rememberExecution(key, domain === "source_acquisition" ? { domain, source_ref: String(form.get("reference")).trim(), attempt: Number(form.get("attempt")) } : domain === "cognitive_realization" ? { domain, plan_ref: String(form.get("reference")).trim() } : domain === "controlled_effect" ? { domain, operation_ref: String(form.get("reference")).trim() } : { domain, submission_ref: String(form.get("reference")).trim() }); setManual(false); }}><label>Execution family<select value={domain} onChange={event => setDomain(event.target.value as ExecutionReference["domain"])}><option value="controlled_effect">Controlled effect</option><option value="runtime_work">Runtime work</option><option value="cognitive_realization">Cognitive realization</option><option value="source_acquisition">Source acquisition</option><option value="resource_request">Resource request</option></select></label><label>Exact reference<input name="reference" required /></label>{domain === "source_acquisition" && <label>Attempt<input name="attempt" type="number" min={1} defaultValue={1} required /></label>}<Button type="submit">Observe reference</Button></form>}
    {refs.slice().reverse().map(ref => <ExecutionReceipt key={JSON.stringify(ref)} workspace={workspace} platform={platform} reference={ref} />)}
  </section>;
}

export function ExecutionReceipt({ workspace, platform, reference, compact = false }: Context & { reference: ExecutionReference; compact?: boolean }) {
  const application = platform.application; const availability = useApplicationAvailability(application);
  const identity = JSON.stringify([workspace.case.case_ref, workspace.case.participant_ref, workspace.case.generation, reference]);
  const current = useRef(identity); current.current = identity; const sequence = useRef(0);
  const [observation, setObservation] = useState<{ identity: string; value: ExecutionObservation }>();
  const result = observation?.identity === identity ? observation.value : undefined; const [error, setError] = useState<string>(); const [busy, setBusy] = useState(false); const [stop, setStop] = useState(false);
  const [resume, setResume] = useState<{ identity: string; submission: string; run: string; digest: string }>();
  const observe = async () => {
    if (!application) return; const stamp = identity; const request = ++sequence.current; setBusy(true); setError(undefined);
    try { const response = await application.execution({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, execution: reference });
      if (stamp !== current.current || request !== sequence.current) return;
      if (response.result_state === "success" && response.data?.case_ref === workspace.case.case_ref && response.data.participant_ref === workspace.case.participant_ref && (reference.domain !== "cognitive_realization" || response.data.plan_ref === reference.plan_ref) && (reference.domain !== "controlled_effect" || response.data.operation_ref === reference.operation_ref)) setObservation({ identity: stamp, value: response.data });
      else { setObservation(undefined); setError(response.error?.safe_message ?? "No currently authorized observation for this exact reference."); }
    } catch {
      if (stamp === current.current && request === sequence.current) { setObservation(undefined); setError("Execution observation is unavailable."); }
    } finally { if (request === sequence.current) setBusy(false); }
  };
  // A remounted/reconnected receipt can only observe, never replay its submission.
  useEffect(() => { void observe(); }, [application, availability, identity]);
  const posture = result?.progress?.status ?? result?.state ?? (typeof result?.posture === "string" ? result.posture : result?.posture?.state);
  const detail = typeof result?.posture === "object" ? result.posture : undefined;
  return <article className="execution-receipt"><header><strong>{compact && reference.domain === "source_acquisition" ? `Acquisition · attempt ${reference.attempt}` : reference.domain.replaceAll("_", " ")}</strong>{posture && <Badge tone={/refused|denied|failed|rejected/.test(posture) ? "error" : /indeterminate|unresolved|waiting|inaccessible/.test(posture) ? "warning" : /completed|acquired|provider_result_recorded/.test(posture) ? "success" : "info"}>{posture.replaceAll("_", " ")}</Badge>}<Button disabled={busy || !application?.supports("execution.get")} onClick={() => void observe()}>{busy ? "Observing…" : "Refresh observation"}</Button></header>{compact ? <details><summary>Exact attempt</summary><code>{label(reference)}</code>{result?.progress_ref && <code>{result.progress_ref}</code>}</details> : <code>{label(reference)}</code>}
    {posture === "delivery_indeterminate" && <p role="status">The recorded attempt has no confirmed result and its execution is no longer observed as active. Refresh only checks retained evidence; it does not restart the attempt.</p>}
    {posture === "unresolved" && <p role="status">YAI cannot establish whether this attempt is still active. Its retained phase is not proof of a running process. No new attempt is sent.</p>}
    {error && <p role="alert">{error} No new submission was sent.</p>}
    {result && reference.domain !== "controlled_effect" && <dl className="object-facts">{Object.entries({ Execution: compact ? undefined : result.execution_ref ?? result.operation_ref ?? result.progress_ref, "Runner posture": result.runner?.posture, "Stop requested": result.runner ? String(result.runner.stop_requested) : undefined, "Current Source phase": result.current_source_phase, "Effect outcome": detail?.outcome, "Effect started": detail?.external_execution_started == null ? undefined : String(detail.external_execution_started), Result: detail?.result_ref, Receipt: detail?.receipt_ref, Review: detail?.review_ref }).filter(([, value]) => value != null).map(([name, value]) => <div key={name}><dt>{name}</dt><dd>{value}</dd></div>)}</dl>}
    {reference.domain === "controlled_effect" && result && <section aria-label="Controlled effect evidence">
      <p>{!result.progress ? "No effect admission outcome is recorded yet." : result.progress.status === "awaiting_review" ? "Waiting for Review. This observation grants no permission to execute." : result.progress.status === "indeterminate" ? "The recorded effect has no final outcome. Observation never repeats it." : "Current retained effect evidence; historical permission does not authorize another dispatch."}</p>
      <dl className="object-facts"><div><dt>Outcome</dt><dd>{result.progress?.outcome?.replaceAll("_", " ") ?? "Not recorded"}</dd></div></dl>
      <details><summary>Effect identities</summary><dl className="object-facts">{Object.entries({ Decision: result.progress?.decision_id, Review: result.progress?.review_id, Effect: result.progress?.effect_id, Receipt: result.progress?.receipt_id }).filter(([, value]) => value != null).map(([name, value]) => <div key={name}><dt>{name}</dt><dd>{value}</dd></div>)}</dl></details>
    </section>}
    {reference.domain === "cognitive_realization" && result && <>
      {result.provider_result && <section aria-label="Retained cognitive result"><p>Recorded model output · candidate material</p><pre className="cognitive-result">{result.provider_result.output}</pre><details><summary>Result identity</summary><code>{result.provider_result.result_id}</code></details></section>}
      {result.attempt_outcomes?.map((outcome, index) => <ConversationAttempt key={outcome.outcome_id ?? index} outcome={outcome} />)}
    </>}
    {reference.domain === "runtime_work" && result?.runner && <Button disabled={!application?.supports("case.stop") || result.runner.stop_requested || result.runner.posture !== "running"} onClick={() => setStop(true)}>Stop this run…</Button>}
    {reference.domain === "runtime_work" && result?.runner?.posture === "operator_stopped" && <Button disabled={busy || !application?.supports("case.resume") || !result.runner.checkpoint_digest} onClick={() => setResume({ identity, submission: `request:studio:${crypto.randomUUID()}`, run: result.runner!.run_ref, digest: result.runner!.checkpoint_digest })}>Resume stopped work…</Button>}
    {resume && application && reference.domain === "runtime_work" && <ApplicationActionDialog title="Resume stopped work" description="Continue the exact observed run under new total limits. Previously consumed budgets and delivery history remain; this does not restart the task or replay an uncertain provider request. YAI rechecks the checkpoint and current authority." submitLabel="Submit continuation" close={() => setResume(undefined)} enabled={resume.identity === identity} submit={form => {
      rememberExecution(keyFor(workspace), { domain: "runtime_work", submission_ref: resume.submission });
      return application.resumeCase({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, previous_submission_ref: reference.submission_ref, submission_ref: resume.submission, run_ref: resume.run, checkpoint_digest: resume.digest, budgets: { max_invocations: Number(form.get("invocations")), max_operations: Number(form.get("operations")), max_runtime_ms: Number(form.get("seconds")) * 1000, max_semantic_units: 16384, max_resident_items: 48, max_estimated_input_units: 32768, max_provider_retries: 0, stop_on_deny: true, continue_after_malformed: false } });
    }} committed={() => refresh(platform)} resync={() => refresh(platform)}><label>Total invocation limit<input name="invocations" type="number" min={1} max={16} defaultValue={2} required /></label><label>Total operation limit<input name="operations" type="number" min={1} max={16} defaultValue={2} required /></label><label>Maximum runtime, seconds<input name="seconds" type="number" min={1} max={300} defaultValue={30} required /></label><p>Limits include work already consumed. Insufficient remaining budget may stop the continuation without another invocation.</p><small>Recovery reference: {resume.submission}</small>{resume.identity !== identity && <p role="alert">The Case changed. Close and refresh the observation before continuing.</p>}</ApplicationActionDialog>}
    {stop && application && reference.domain === "runtime_work" && result?.runner && <ApplicationActionDialog title="Stop this run" description="Request cooperative stop of this exact runner. This neither cancels nor closes the Case, and does not claim an external effect has been undone." submitLabel="Request stop" close={() => setStop(false)} submit={() => application.stopCase({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, submission_ref: reference.submission_ref, run_ref: result.runner!.run_ref })} committed={observe}><code>{result.runner.run_ref}</code></ApplicationActionDialog>}
  </article>;
}

export function AcquireSourceAction({ workspace, platform, sourceRef }: Context & { sourceRef: string }) {
  const application = platform.application; useApplicationAvailability(application);
  const source = workspace.environment.sources.find(item => item.id === sourceRef);
  const [pending, setPending] = useState<{ resume: boolean; generation: number; attempt: number; progress?: string }>();
  if (!source) return null;
  const resumable = ["inaccessible", "needsprocessing", "needs_processing", "awaitingreview", "awaiting_review"].includes(source.posture ?? "");
  return <section className="source-acquisition"><div className="object-action-row"><Button disabled={!application?.supports("source.acquire") || source.posture === "acquiring" || source.posture === "revoked" || resumable} onClick={() => setPending({ resume: false, generation: workspace.case.generation, attempt: (source.attempt ?? 0) + 1 })}>Acquire Source…</Button><Button disabled={!application?.supports("source.resume") || !resumable || !source.progress_ref || !source.attempt} onClick={() => setPending({ resume: true, generation: workspace.case.generation, attempt: source.attempt!, progress: source.progress_ref })}>Resume acquisition…</Button></div><p className="surface-note">Acquisition is governed by YAI. Refresh checks status without restarting work.</p>
    {pending && application && <ApplicationActionDialog title={pending.resume ? "Resume acquisition" : "Acquire Source"} description="YAI checks current generation, Participant, Resource and policy before acquiring material. The exact attempt is retained locally before dispatch so a lost response can be observed from Work > Executions." submitLabel={pending.resume ? "Resume exact attempt" : "Acquire exact Source"} close={() => setPending(undefined)} enabled={pending.generation === workspace.case.generation} submit={() => {
      rememberExecution(keyFor(workspace), { domain: "source_acquisition", source_ref: sourceRef, attempt: pending.attempt });
      const input = { case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, source_ref: sourceRef, attempt: pending.attempt, expected_generation: pending.generation };
      return pending.resume ? application.resumeSource({ ...input, previous_progress_ref: pending.progress! }) : application.acquireSource(input);
    }} committed={() => refresh(platform)} resync={() => refresh(platform)}><p>{source.label} · attempt {pending.attempt} · Case state version {pending.generation}</p>{pending.generation !== workspace.case.generation && <p role="alert">The Case changed. Reopen the action.</p>}</ApplicationActionDialog>}
    {source.attempt != null && source.attempt > 0 && <ExecutionReceipt compact workspace={workspace} platform={platform} reference={{ domain: "source_acquisition", source_ref: sourceRef, attempt: source.attempt }} />}
  </section>;
}

export function RunCaseAction({ workspace, platform }: Context) {
  const application = platform.application; useApplicationAvailability(application); const [pending, setPending] = useState<string>();
  return <><Button disabled={!application?.supports("case.run") || !workspace.environment.resources.length} onClick={() => setPending(`request:studio:${crypto.randomUUID()}`)}>Run bounded work…</Button>
    {pending && application && <ApplicationActionDialog title="Run bounded work" description="Submit one exact task to the resident scheduler. Queue admission does not imply provider execution or completion. Current policy and provider qualification remain YAI-owned." submitLabel="Submit work" close={() => setPending(undefined)} submit={form => {
      rememberExecution(keyFor(workspace), { domain: "runtime_work", submission_ref: pending });
      return application.runCase({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, resource_ref: String(form.get("resource")), submission_ref: pending, task: String(form.get("task")), budgets: { max_invocations: Number(form.get("invocations")), max_operations: Number(form.get("operations")), max_runtime_ms: Number(form.get("seconds")) * 1000, max_semantic_units: 16384, max_resident_items: 48, max_estimated_input_units: 32768, max_provider_retries: 0, stop_on_deny: true, continue_after_malformed: false } });
    }} committed={() => refresh(platform)} resync={() => refresh(platform)}><label>Task<textarea name="task" required rows={4} autoFocus /></label><label>Resource<select name="resource" aria-label="Resource">{workspace.environment.resources.map(item => <option key={item.id} value={item.id}>{item.label ?? item.id} · {item.kind}</option>)}</select></label><label>Maximum invocations<input name="invocations" type="number" min={1} max={16} defaultValue={1} required /></label><label>Maximum operations<input name="operations" type="number" min={1} max={16} defaultValue={1} required /></label><label>Maximum runtime, seconds<input name="seconds" type="number" min={1} max={300} defaultValue={30} required /></label><small>No provider retry; stop on denial or malformed output. Recovery reference: {pending}.</small></ApplicationActionDialog>}
  </>;
}

const supported = new Set(["filesystem_read", "filesystem_search", "discover", "process_run", "database_query", "http_fetch", "mcp_catalog"]);
const actionForOperation: Record<string, string> = { "filesystem.read": "filesystem_read", "filesystem.search": "filesystem_search", "discovery.enumerate": "discover", "process.run": "process_run", "database.query": "database_query", "database.mutate": "database_mutation", "http.fetch": "http_fetch", "mcp.catalog": "mcp_catalog" };
export function RequestResourceAction({ workspace, platform, resourceRef }: Context & { resourceRef: string }) {
  const application = platform.application; useApplicationAvailability(application); const resource = workspace.environment.resources.find(item => item.id === resourceRef);
  const [pending, setPending] = useState<{ ref: string; generation: number }>(); const [kind, setKind] = useState("");
  const operations = resource?.operations.map(item => actionForOperation[item]).filter(item => supported.has(item)) ?? [];
  if (!resource) return null;
  return <><Button disabled={!application?.supports("resource.request") || !resource.configuration_digest || !operations.length} onClick={() => { setKind(operations[0]); setPending({ ref: `request:studio:${crypto.randomUUID()}`, generation: workspace.case.generation }); }}>Request Resource operation…</Button>
    {pending && application && <ApplicationActionDialog title="Request Resource operation" description="This can perform the selected external read or controlled effect. YAI checks the exact bound configuration, current policy, review and effect-time fences. Names refer to pre-bound operations, never arbitrary commands, SQL or URLs." submitLabel="Submit governed request" close={() => setPending(undefined)} enabled={pending.generation === workspace.case.generation} submit={form => {
      const action: ResourceAction = kind === "mcp_catalog" ? { action: kind } : kind === "filesystem_search" ? { action: kind, path: String(form.get("path")), needle: String(form.get("needle")) } : kind === "filesystem_read" || kind === "discover" ? { action: kind, path: String(form.get("path")) } : { action: kind as "process_run" | "database_query" | "database_mutation" | "http_fetch", name: String(form.get("name")) };
      rememberExecution(keyFor(workspace), { domain: "resource_request", submission_ref: pending.ref });
      return application.requestResource({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, resource_ref: resource.id, submission_ref: pending.ref, expected_generation: pending.generation, request: { schema: "yai.resource_request.v1", configuration_digest: resource.configuration_digest!, action } });
    }} committed={() => refresh(platform)} resync={() => refresh(platform)}><label>Operation<select value={kind} onChange={event => setKind(event.target.value)}>{operations.map(op => <option key={op} value={op}>{op.replaceAll("_", " ")}</option>)}</select></label>{["filesystem_read", "filesystem_search", "discover"].includes(kind) && <label>Exact relative path<input name="path" required defaultValue={resource.read_prefixes[0] ?? ""} /></label>}{kind === "filesystem_search" && <label>Search text<input name="needle" required /></label>}{["database_query", "process_run", "http_fetch"].includes(kind) && <label>Bound operation name<select name="name" required>{resource.names.map(name => <option key={name}>{name}</option>)}</select></label>}<p>Case state version {pending.generation} · {resource.review_requirement}</p><small>Recovery reference: {pending.ref}</small>{pending.generation !== workspace.case.generation && <p role="alert">The Case changed. Reopen the action.</p>}</ApplicationActionDialog>}
  </>;
}

export function AttachProcessAction({ workspace, platform }: Context) {
  const application = platform.application; useApplicationAvailability(application); const [open, setOpen] = useState(false);
  return <><Button disabled={!application?.supports("resource.attach_process")} onClick={() => setOpen(true)}>Attach process…</Button>{open && application && <ApplicationActionDialog title="Attach process Resource" description="YAI captures the exact current PID/process identity. This attaches bounded signal capability; it does not send a signal. Current same-user and Case authority checks still apply." submitLabel="Attach process" close={() => setOpen(false)} submit={form => application.attachProcess({ case_ref: workspace.case.case_ref, attachment_ref: String(form.get("ref")).trim(), pid: Number(form.get("pid")), policy_owner_participant_ref: workspace.case.participant_ref, actions: form.getAll("signals") as Array<"terminate" | "suspend" | "resume">, review_requirement: "require_review" })} committed={() => refresh(platform)}><label>Resource reference<input name="ref" required placeholder="resource:controlled-process" autoFocus /></label><label>Exact PID<input name="pid" type="number" min={1} required /></label><fieldset><legend>Bound signal actions</legend>{["suspend", "resume", "terminate"].map(action => <label key={action}><input type="checkbox" name="signals" value={action} />{action}</label>)}</fieldset><p>Review is required. An attached process is distinct from a preconfigured process runner.</p></ApplicationActionDialog>}</>;
}
