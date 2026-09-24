import type { ProviderAttemptObservation } from "../../clients/conversation";

const deliveryLabels: Record<string, string> = {
  not_dispatched: "Not sent",
  definitively_rejected: "Rejected before execution",
  delivery_indeterminate: "Delivery uncertain",
  response_invalid: "Response could not be accepted",
  result_received: "Result received",
  cancelled: "Cancelled before dispatch",
};

/** Retained transport facts only. This view neither probes nor retries. */
export function ConversationAttempt({ outcome }: { outcome: ProviderAttemptObservation }) {
  const recorded = outcome.recorded_at_unix_ms;
  return <section className="conversation-attempt" aria-label="Provider attempt">
    <header><strong>{deliveryLabels[outcome.delivery ?? ""] ?? "Delivery not classified"}</strong>
      {outcome.attempt_number != null && <span>Attempt {outcome.attempt_number}</span>}</header>
    {outcome.delivery === "delivery_indeterminate" && <p>The provider may have executed this request. This observation does not authorize sending it again.</p>}
    {outcome.delivery === "response_invalid" && <p>A response arrived but did not satisfy the response contract. It is not a completed model answer.</p>}
    <dl>
      <dt>Transport stage</dt><dd>{outcome.stage?.replaceAll("_", " ") ?? "Not recorded"}</dd>
      <dt>HTTP response</dt><dd>{outcome.response_status ?? "Not recorded"}</dd>
      <dt>Request bytes written</dt><dd>{outcome.request_bytes_written?.toLocaleString() ?? "Not recorded"}</dd>
      {outcome.failure_class && <><dt>Recorded failure</dt><dd>{outcome.failure_class.replaceAll("_", " ")}</dd></>}
      <dt>Recorded at</dt><dd>{recorded != null && Number.isFinite(recorded) ? new Date(recorded).toLocaleString() : "Not recorded"}</dd>
    </dl>
    <details><summary>Exact transport evidence</summary><pre>{JSON.stringify(outcome, null, 2)}</pre></details>
  </section>;
}
