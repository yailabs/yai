import { useEffect, useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { ConversationExecution } from "../../clients/conversation";
import type { PreparedContext } from "../../clients/executionContext";
import { Button } from "../../components/primitives";
import "./ExecutionContext.css";

/** Explicit archived read: never invoke Recall anew or retry a provider request. */
export function ExecutionContext({ application, execution, generation }: {
  application: ApplicationAccess; execution: ConversationExecution; generation: number;
}) {
  const [requested, setRequested] = useState(0);
  const [state, setState] = useState<{ key: string; data?: PreparedContext; error?: string }>();
  const key = `${execution.case_ref}:${execution.participant_ref}:${execution.request_ref}:${generation}`;
  useEffect(() => {
    if (!requested) return;
    let current = true;
    setState(undefined);
    void application.observeConversation({ case_ref: execution.case_ref, participant_ref: execution.participant_ref,
      execution: { domain: "cognitive_composition", request_ref: execution.request_ref }, include_context: true }).then(result => {
      if (!current) return;
      const value = result.data;
      if (result.result_state !== "success" || !value?.prepared_context) {
        setState({ key, error: result.error?.safe_message ?? "Archived context is unavailable." }); return;
      }
      if (value.case_ref !== execution.case_ref || value.participant_ref !== execution.participant_ref
        || value.request_ref !== execution.request_ref || value.observed_generation !== generation
        || value.prepared_context.observed_generation !== generation) {
        setState({ key, error: "Context changed. Refresh the Case before inspecting it again." }); return;
      }
      setState({ key, data: value.prepared_context });
    }).catch(() => { if (current) setState({ key, error: "Context inspection is unavailable. No request was resent." }); });
    return () => { current = false; };
  }, [application, key, requested, generation, execution.case_ref, execution.participant_ref, execution.request_ref]);
  const visible = state?.key === key ? state : undefined;
  return <section className="execution-context" aria-label="Model context">
    <Button disabled={Boolean(requested && !visible)} onClick={() => setRequested(value => value + 1)}>
      {requested && !visible ? "Inspecting context…" : "Inspect model context"}
    </Button>
    {visible?.error && <p role="alert">{visible.error}</p>}
    {visible?.data && <>
      {!visible.data.invocations.length && <p>No prepared invocation is retained for this execution.</p>}
      {visible.data.invocations.map(entry => { const input = entry.input_observation; const capacity = input?.capacity; const working = entry.working_state;
        return <div key={entry.invocation_ref}>
          <p>Prepared at generation {entry.lineage.case_generation}. Disclosure checked at generation {visible.data!.observed_generation}.</p>
          {entry.unavailable_reason ? <p>{entry.unavailable_reason.replaceAll("_", " ")}</p> : <>
            <dl>
              <dt>Selected evidence/state</dt><dd>{working?.bounds.selected_items ?? "Unknown"} items</dd>
              <dt>Omitted</dt><dd>{working?.bounds.omitted_items ?? "Unknown"} items</dd>
              <dt>Serialized request</dt><dd>{input ? `${input.serialized_request_bytes.toLocaleString()} bytes` : "Not retained"}</dd>
              <dt>Input tokens</dt><dd>{capacity?.input_tokens?.toLocaleString() ?? "Unknown"}</dd>
              <dt>Sequence capacity</dt><dd>{capacity?.sequence_capacity_tokens.toLocaleString() ?? "Unknown"}</dd>
              <dt>Effective output budget</dt><dd>{capacity?.effective_output_tokens?.toLocaleString() ?? "Unknown"}</dd>
              <dt>Capacity check</dt><dd>{capacity?.token_capacity_compatible == null ? "Unknown" : capacity.token_capacity_compatible ? "Fits observed capacity" : "Does not fit"}</dd>
            </dl>
            {input?.refusal && <p role="status">Not dispatched: {input.refusal.replaceAll("_", " ")}</p>}
            {input && <p>Observed {new Date(input.observed_at_unix_ms).toLocaleString()}. Capacity is not a resource reservation.</p>}
            <details><summary>Required state, recalled evidence and omissions</summary>
              {working?.entries.map(item => <div className="execution-context-entry" key={item.entry_id}><strong>{item.value.kind.replaceAll("_", " ")}</strong><small>{item.posture.replaceAll("_", " ")}</small><code>{item.entry_id}</code></div>)}
              <details><summary>Full Working State evidence</summary><pre>{JSON.stringify(working, null, 2)}</pre></details>
            </details>
            <details><summary>Exact frame and transport evidence</summary><pre>{JSON.stringify({ lineage: entry.lineage, frame: entry.frame, input }, null, 2)}</pre></details>
          </>}
        </div>;
      })}
      {visible.data.omitted_invocations > 0 && <p>{visible.data.omitted_invocations} additional invocations are outside this bounded observation.</p>}
    </>}
  </section>;
}
