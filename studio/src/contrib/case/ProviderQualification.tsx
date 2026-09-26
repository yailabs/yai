import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
import type { PlatformServices } from "../../platform/services";
import type { ProviderProbeExecution, ProviderProbeInput } from "../../clients/compute";
import { Badge, Button } from "../../components/primitives";

type Mode = "text" | "tools" | "embedding";
interface Receipt { submission: string; mode: Mode }
const modes = new Set<Mode>(["text", "tools", "embedding"]);
const elapsed = (since: number, now: number) => {
  const seconds = Math.max(0, Math.floor((now - since) / 1000));
  return `${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, "0")}s`;
};
const outcome = (run: ProviderProbeExecution) => run.posture === "running" ? "Still checking"
  : run.posture === "completed" && !run.run.evidence?.exact_model_addressed ? "Completed · exact model not proven"
  : run.posture === "completed" && run.run.qualification?.capabilities.length ? "Completed · evidence recorded"
  : run.posture === "completed" ? "Completed · no capability proven"
  : run.posture === "interrupted" ? "Interrupted · no automatic retry" : "Failed · no capability proven";
function readReceipt(key: string): Receipt | undefined {
  try {
    const item = JSON.parse(localStorage.getItem(key) ?? "null");
    if (item && typeof item.submission === "string" && item.submission.startsWith("studio-probe:") && item.submission.length <= 128 && modes.has(item.mode)) return { submission: item.submission, mode: item.mode };
  } catch { /* Invalid local presentation state grants no authority. */ }
}
function request(target: string, receipt: Receipt): ProviderProbeInput {
  return { target_ref: target, submission_ref: receipt.submission, embedding: receipt.mode === "embedding", qualify: true,
    realization_shapes: receipt.mode === "embedding" ? [] : receipt.mode === "tools" ? ["text_to_text", "text_functions_to_text_or_call", "text_to_json_object"] : ["text_to_text"] };
}
function matches(result: ProviderProbeExecution, input: ProviderProbeInput): boolean {
  return ["running", "completed", "failed", "interrupted"].includes(result.posture) && Boolean(result.run?.request) && result.schema === "yai.provider_probe_execution.v1" && result.target_ref === input.target_ref && result.submission_ref === input.submission_ref
    && result.run.request.target_id === input.target_ref && result.run.request.submission_ref === input.submission_ref
    && result.run.request.embedding === input.embedding && result.run.request.qualify === input.qualify
    && JSON.stringify([...result.run.request.realization_shapes].sort()) === JSON.stringify([...input.realization_shapes].sort())
    && (result.run.request.valid_for_ms ?? null) === (input.valid_for_ms ?? null);
}

function refusal(error?: { code: string; safe_message: string }): string {
  switch (error?.code) {
    case "provider_probe_already_in_flight": return "Another check already owns this target. Wait for its result before starting a new check.";
    case "provider_probe_circuit_cooldown_active": return "YAI is waiting before another check after repeated failures. No new probe was dispatched.";
    case "application_execution_capacity_pending": return "YAI has reached its active execution limit. No probe was admitted; retry explicitly when capacity is available.";
    case "execution_carrier_active_requires_observation": return "This check is already being admitted. Observe its retained reference instead of starting another.";
    case "provider_probe_run_retention_limit": return "The retained-check limit has been reached. No new probe was dispatched.";
    case "provider_probe_submission_conflict": return "This recovery reference belongs to a different request. No replacement was dispatched.";
    case "provider_probe_run_not_found": return "No admitted check was found for this reference. You may explicitly retry the exact request.";
    default: return error?.safe_message ?? "This check cannot currently be observed.";
  }
}

/** Retains only the request reference. Results are re-observed through current authority. */
export function ProviderQualificationCheck({ platform, tenant, target, onCompleted }: { platform: PlatformServices; tenant: string; target: string; onCompleted?: () => void | Promise<unknown> }) {
  const host = useSyncExternalStore(useCallback(listener => platform.host.subscribe(listener).dispose, [platform]),
    useCallback(() => platform.host.snapshot(), [platform]));
  const profile = host.telemetry?.yai_home_identity;
  if (!profile) return <p>Connect to YAI to check this deployment.</p>;
  const key = `yai.studio.provider-probe.v1:${JSON.stringify([profile, tenant, target])}`;
  return <Qualification key={key} storageKey={key} platform={platform} target={target} onCompleted={onCompleted} />;
}
function Qualification({ storageKey, platform, target, onCompleted }: { storageKey: string; platform: PlatformServices; target: string; onCompleted?: () => void | Promise<unknown> }) {
  const application = platform.application;
  const availability = useSyncExternalStore(useCallback(listener => application?.subscribe(listener).dispose ?? (() => {}), [application]),
    useCallback(() => application?.snapshot(), [application]));
  const [receipt, setReceipt] = useState(() => readReceipt(storageKey));
  const [mode, setMode] = useState<Mode>(receipt?.mode ?? "text");
  const [result, setResult] = useState<ProviderProbeExecution>();
  const [clock, setClock] = useState(() => Date.now());
  const [error, setError] = useState<string>();
  const [submitting, setSubmitting] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const [history, setHistory] = useState<{ catalog: NonNullable<typeof availability>["catalog"]; runs?: ProviderProbeExecution[]; error?: string }>();
  const epoch = useRef(0);
  const sending = useRef(false);
  const notified = useRef<string | undefined>(undefined);
  const complete = useRef(onCompleted); complete.current = onCompleted;
  useEffect(() => () => { epoch.current++; }, []);
  useEffect(() => {
    if (result?.posture !== "running") return;
    setClock(Date.now());
    const timer = window.setInterval(() => setClock(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, [result?.posture, result?.submission_ref]);
  const available = application?.supports("provider.probe") && application.supports("provider.probe.get");
  const listAvailable = application?.supports("provider.probe.list");
  const currentHistory = available && history?.catalog === availability?.catalog ? history : undefined;
  const anotherRunning = currentHistory?.runs?.some(run => run.posture === "running");
  const canStart = available && listAvailable && Boolean(currentHistory?.runs) && !anotherRunning;
  useEffect(() => {
    if (!application || !listAvailable || submitting) return;
    let active = true, timer: number | undefined;
    const read = async () => {
      try {
        const reply = await application.listProviderProbes(target);
        if (!active) return;
        if (reply.result_state !== "success" || !reply.data) {
          setHistory({ catalog: availability?.catalog, error: refusal(reply.error) }); return;
        }
        const data = reply.data;
        if (data.schema !== "yai.provider_probe_list.v1" || data.target_ref !== target || !Array.isArray(data.runs) || data.runs.length > 64
          || data.runs.some(run => run.schema !== "yai.provider_probe_execution.v1" || run.target_ref !== target
            || run.run?.request?.target_id !== target || run.submission_ref !== run.run?.request?.submission_ref
            || !["running", "completed", "failed", "interrupted"].includes(run.posture)
            || !Number.isFinite(run.run?.owner?.started_at_unix_ms))) {
          setHistory({ catalog: availability?.catalog, error: "Provider history identity mismatch. Results withheld." }); return;
        }
        setHistory({ catalog: availability?.catalog, runs: data.runs });
        if (data.runs.some(run => run.posture === "running")) timer = window.setTimeout(read, 3000);
      } catch { if (active) setHistory({ catalog: availability?.catalog, error: "Cannot observe retained checks. Refresh before starting a new one." }); }
    };
    void read();
    return () => { active = false; window.clearTimeout(timer); };
  }, [application, listAvailable, availability?.catalog, target, receipt, submitting, refresh, result?.posture]);
  useEffect(() => {
    if (!available) { setResult(undefined); return; }
    if (!receipt || !application || submitting) return;
    let current = true, timer: number | undefined;
    const input = request(target, receipt);
    const observe = async () => {
      try {
        const reply = await application.observeProviderProbe({ target_ref: target, submission_ref: receipt.submission });
        if (!current) return;
        if (reply.result_state !== "success" || !reply.data) { setResult(undefined); setError(refusal(reply.error)); return; }
        if (!matches(reply.data, input)) { setResult(undefined); setError("Provider check identity mismatch. Result withheld."); return; }
        setError(undefined); setResult(reply.data);
        if (reply.data.posture !== "running" && notified.current !== receipt.submission) {
          notified.current = receipt.submission;
          void Promise.resolve(complete.current?.()).catch(() => {});
        }
        if (reply.data.posture === "running") timer = window.setTimeout(observe, 1500);
      } catch {
        if (current) { setResult(undefined); setError("Connection lost. Observe this exact check when YAI reconnects; it has not been sent again."); }
      }
    };
    void observe();
    return () => { current = false; window.clearTimeout(timer); };
  }, [application, available, availability?.catalog, receipt, target, submitting, refresh]);
  const submit = async (next: Receipt) => {
    if (!application || !available || sending.current) return;
    sending.current = true;
    const token = ++epoch.current;
    // Failure to retain recovery identity must happen before dispatch.
    try { localStorage.setItem(storageKey, JSON.stringify(next)); }
    catch { sending.current = false; setError("Cannot retain the recovery reference. No request was sent."); return; }
    try {
      setReceipt(next); setSubmitting(true); setResult(undefined); setError(undefined);
      const input = request(target, next);
      const reply = await application.probeProvider(input);
      if (epoch.current !== token) return;
      if (reply.result_state === "success" && reply.data && matches(reply.data, input)) setResult(reply.data);
      else setError(reply.error?.safe_message ?? "Check acknowledgement unavailable. Observe the retained request before starting another.");
    } catch {
      if (epoch.current === token) setError("Could not confirm the check. No automatic redispatch will occur.");
    } finally { sending.current = false; if (epoch.current === token) setSubmitting(false); }
  };
  const visible = available ? result : undefined;
  const terminal = visible && visible.posture !== "running";
  const measured = visible?.run.evidence;
  const qualification = visible?.run.qualification;
  return <section className="provider-qualification" aria-label="Deployment qualification">
    <header><h3>Check this deployment</h3>{visible && <Badge tone={visible.posture === "completed" && measured?.exact_model_addressed && qualification?.capabilities.length ? "success" : "warning"}>{visible.posture === "running" ? "Checking" : visible.posture === "completed" ? !measured?.exact_model_addressed ? "Exact model not proven" : qualification?.capabilities.length ? "Evidence recorded" : "No capability proven" : visible.posture === "interrupted" ? "Interrupted" : "Check invalidated"}</Badge>}</header>
    <p>A small test request checks this deployment without sharing Case content. Each completed check replaces the current qualification with only the capabilities it proves. A successful text check also closes YAI’s provider circuit; a failed one can block Case chat. This does not send a chat message.</p>
    <div className="provider-check-controls"><label>Check capabilities<select value={mode} disabled={submitting || Boolean(receipt && !terminal)} onChange={event => setMode(event.target.value as Mode)}><option value="text">Text conversation</option><option value="tools">Text, JSON and tool roundtrip</option><option value="embedding">Embeddings</option></select></label>
      <Button disabled={!canStart || submitting || Boolean(receipt && !terminal)} onClick={() => void submit({ submission: `studio-probe:${crypto.randomUUID()}`, mode })}>{submitting ? "Submitting…" : receipt ? "Run a new check" : "Check & qualify"}</Button>
      {receipt && <Button disabled={!available || submitting} onClick={() => setRefresh(value => value + 1)}>Refresh result</Button>}
      {receipt && !visible && !submitting && <Button disabled={!available} onClick={() => void submit(receipt)}>Retry exact request</Button>}
    </div>
    {!available && <p>This Host does not currently expose qualification start and observation.</p>}
    {!listAvailable && <p>The connected Host does not expose retained-check recovery yet.</p>}
    {currentHistory?.error && <p role="alert">{currentHistory.error}</p>}
    {anotherRunning && !receipt && <p role="status">A retained check is still running. Observe it below; no new check has been started.</p>}
    <Button disabled={!listAvailable || submitting} onClick={() => setRefresh(value => value + 1)}>Refresh check history</Button>
    {error && <p role="alert">{error}</p>}
    {visible?.posture === "running" && <div className="provider-check-running" role="status"><strong>Synthetic check running · {elapsed(visible.run.owner.started_at_unix_ms, clock)}</strong><p>This tests the deployment with a small YAI request, separate from the Case conversation. YAI has not returned a finer progress phase or result. This view observes the same request automatically; it does not resend it.</p><small>Started {new Date(visible.run.owner.started_at_unix_ms).toLocaleString()} · {visible.submission_ref}</small></div>}
    {terminal && <p role="status">This check {visible.posture === "completed" ? "completed" : visible.posture}. {qualification?.capabilities.length ? "Its measured capabilities are listed below." : "No new capability was proven."} This does not establish current model residency or complete a Case conversation.</p>}
    {visible?.posture === "interrupted" && <p>The carrier was interrupted. This request has not been dispatched again.</p>}
    {visible?.run.failure_code && <p role="alert">{visible.run.failure_code.replaceAll("_", " ")}. No new qualification was granted.</p>}
    {measured && <><dl className="provider-check-facts"><div><dt>Exact model</dt><dd>{measured.exact_model_addressed ? "Addressed" : "Not proven"}</dd></div><div><dt>Measured</dt><dd>{new Date(measured.completed_at_unix_ms).toLocaleString()}</dd></div></dl>
      {qualification?.capabilities.length ? <ul>{qualification.capabilities.map(item => <li key={item.capability}>{item.capability.replaceAll("_", " ")}</li>)}</ul> : <p>No qualified capability was recorded by this check.</p>}
      {measured.failure_codes.length > 0 && <details><summary>Reported limitations</summary><ul>{measured.failure_codes.map(code => <li key={code}>{code.replaceAll("_", " ")}</li>)}</ul></details>}
    </>}
    {currentHistory?.runs && <details className="provider-check-history"><summary>Previous checks ({currentHistory.runs.length})</summary>
      {!currentHistory.runs.length && <p>No retained check for this deployment.</p>}
      {currentHistory.runs.map(run => <details key={run.submission_ref}><summary>{run.run.request.embedding ? "Embedding" : "Text"} check · {run.posture === "running" ? `Still checking · ${elapsed(run.run.owner.started_at_unix_ms, clock)}` : outcome(run)} · {new Date(run.run.owner.started_at_unix_ms).toLocaleString()}</summary>
        <code>{run.submission_ref}</code><p>{run.run.request.embedding ? "Embeddings" : run.run.request.realization_shapes.join(", ") || "Text and JSON"} · {run.run.request.qualify ? "Qualification requested" : "Observation only"}</p>
        {run.run.evidence && <p>Exact model: {run.run.evidence.exact_model_addressed ? "proven" : "not proven"}. {run.run.evidence.failure_codes.join(", ") || "No reported failure."}</p>}
        {run.run.failure_code && <p>{run.run.failure_code.replaceAll("_", " ")}</p>}
      </details>)}
    </details>}
    {receipt && <details><summary>Check ID and recovery</summary><code>{receipt.submission}</code><p>Use this ID to reopen the same result. It does not start another check.</p></details>}
  </section>;
}
