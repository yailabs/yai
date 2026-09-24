import { useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import type { OperationResult } from "../clients/live";
import { useModalFocus } from "./useModalFocus";
import { Button } from "./primitives";

/** Shared submission feedback. Fields and typed operations remain contribution-owned. */
export function ApplicationActionDialog({ title, description, submitLabel, children, close, submit, committed, resync, enabled = true, dismissOnSuccess = true }: {
  title: string; description: string; submitLabel: string; children: ReactNode;
  close(): void; submit(form: FormData): Promise<OperationResult<unknown>>;
  committed(): void | Promise<void>; enabled?: boolean; dismissOnSuccess?: boolean;
  resync?(): void | Promise<void>;
}) {
  const [pending, setPending] = useState(false);
  const [result, setResult] = useState<OperationResult<unknown>>();
  const submitting = useRef(false);
  const root = useModalFocus(() => { if (!submitting.current) close(); });
  const uncertain = result?.result_state === "transport_unavailable";
  const explanation = result?.error?.code === "case_close_requires_cancellation" ? "YAI requires a Case cancellation before final closure. Cancel the Case first, then request closure; YAI will check remaining blockers."
    : ["normalization_failure_result_mismatch", "operation_provider_lineage_mismatch"].includes(result?.error?.code ?? "") ? "YAI could not verify the exact candidate provenance. No Operation was recorded; refresh the Case and inspect the execution."
    : result?.error?.code === "workflow_human_input_bounds_invalid" ? "The input exceeds the bounds declared by this Workflow node. Adjust it and submit again."
    : result?.error?.safe_message ?? result?.result_state;
  const send = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (submitting.current || !enabled || uncertain) return;
    const form = new FormData(event.currentTarget);
    submitting.current = true; setPending(true); setResult(undefined);
    try {
      const response = await submit(form); setResult(response);
      if (response.result_state === "success") { await committed(); if (dismissOnSuccess) close(); }
    } catch {
      setResult({ operation_ref: "studio.action", correlation_ref: "studio:unconfirmed", result_state: "transport_unavailable", error: { code: "action_confirmation_unavailable", message: "action_confirmation_unavailable", safe_message: "The current state could not be confirmed. Refresh the Case before submitting another action.", result_state: "transport_unavailable" } });
    } finally { submitting.current = false; setPending(false); }
  };
  return createPortal(<div className="modal-backdrop application-action-backdrop" role="presentation"><section className="application-action-dialog" ref={root} role="dialog" aria-modal="true" aria-label={title}>
    <header><h2>{title}</h2><p>{description}</p></header>
    <form onSubmit={event => void send(event)}><div className="application-action-fields"><fieldset disabled={pending}>{children}</fieldset></div>
      {result && result.result_state !== "success" && <div className="action-result" role="alert"><strong>{uncertain ? "Confirmation was lost" : "YAI did not accept the action"}</strong><p>{explanation}</p>{uncertain && <p>The action may have reached YAI. Check current Case state before trying again; Studio will not resubmit it automatically.</p>}<details><summary>Technical details</summary><code>{result.operation_ref} · {result.error?.code ?? result.result_state}<br />{result.correlation_ref}</code></details></div>}
      <footer><Button type="button" disabled={pending} onClick={close}>{uncertain ? "Close and inspect state" : "Cancel"}</Button>{result?.result_state === "stale" && resync && <Button type="button" disabled={pending} onClick={() => { close(); void resync(); }}>Close and refresh Case</Button>}<Button type="submit" disabled={pending || !enabled || uncertain}>{pending ? "Waiting for YAI…" : submitLabel}</Button></footer>
    </form>
  </section></div>, document.body);
}
