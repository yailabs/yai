import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
import type { PlatformServices } from "../../platform/services";
import type { ProviderProbeExecution, ProviderProbeInput } from "../../clients/compute";
import { Badge, Button } from "../../components/primitives";

type Mode = "text" | "tools" | "embedding";
interface Receipt { submission: string; mode: Mode }
const modes = new Set<Mode>(["text", "tools", "embedding"]);
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
  const [error, setError] = useState<string>();
  const [submitting, setSubmitting] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const epoch = useRef(0);
  const sending = useRef(false);
  const notified = useRef<string | undefined>(undefined);
  const complete = useRef(onCompleted); complete.current = onCompleted;
  useEffect(() => () => { epoch.current++; }, []);
  const available = application?.supports("provider.probe") && application.supports("provider.probe.get");
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
    <p>Small synthetic requests through YAI. No Case content is sent; trust and Case binding stay separate.</p>
    <div className="provider-check-controls"><label>Check capabilities<select value={mode} disabled={submitting || Boolean(receipt && !terminal)} onChange={event => setMode(event.target.value as Mode)}><option value="text">Text conversation</option><option value="tools">Text, JSON and tool roundtrip</option><option value="embedding">Embeddings</option></select></label>
      <Button disabled={!available || submitting || Boolean(receipt && !terminal)} onClick={() => void submit({ submission: `studio-probe:${crypto.randomUUID()}`, mode })}>{submitting ? "Submitting…" : receipt ? "Run a new check" : "Check & qualify"}</Button>
      {receipt && <Button disabled={!available || submitting} onClick={() => setRefresh(value => value + 1)}>Observe result</Button>}
      {receipt && !visible && !submitting && <Button disabled={!available} onClick={() => void submit(receipt)}>Retry exact request</Button>}
    </div>
    {!available && <p>This Host does not currently expose qualification start and observation.</p>}
    {error && <p role="alert">{error}</p>}
    {visible?.posture === "running" && <p role="status">YAI is checking the endpoint. You can leave this view and return to observe the same request.</p>}
    {visible?.posture === "interrupted" && <p>The carrier was interrupted. This request has not been dispatched again.</p>}
    {visible?.run.failure_code && <p role="alert">{visible.run.failure_code.replaceAll("_", " ")}. No new qualification was granted.</p>}
    {measured && <><dl className="provider-check-facts"><div><dt>Exact model</dt><dd>{measured.exact_model_addressed ? "Addressed" : "Not proven"}</dd></div><div><dt>Measured</dt><dd>{new Date(measured.completed_at_unix_ms).toLocaleString()}</dd></div></dl>
      {qualification?.capabilities.length ? <ul>{qualification.capabilities.map(item => <li key={item.capability}>{item.capability.replaceAll("_", " ")}</li>)}</ul> : <p>No qualified capability was recorded by this check.</p>}
      {measured.failure_codes.length > 0 && <details><summary>Reported limitations</summary><ul>{measured.failure_codes.map(code => <li key={code}>{code.replaceAll("_", " ")}</li>)}</ul></details>}
    </>}
    {receipt && <details><summary>Recovery reference</summary><code>{receipt.submission}</code><p>Observation uses current Tenant authority. This reference is not permission to execute.</p></details>}
  </section>;
}
