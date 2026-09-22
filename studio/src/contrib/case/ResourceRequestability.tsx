import { useEffect, useState } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { ApplicationAccess, CaseCapabilityView } from "../../clients/application";
import { resourceOperationLabel } from "../../clients/application";
import { Badge } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

export function ResourceRequestability({ workspace, resourceRef, application }: { workspace: CasePresentation; resourceRef: string; application?: ApplicationAccess }) {
  const availability = useApplicationAvailability(application);
  const [result, setResult] = useState<{ identity: string; view?: CaseCapabilityView; error?: string }>();
  const identity = JSON.stringify([workspace.case.case_ref, workspace.case.generation, workspace.case.participant_ref]);
  useEffect(() => { let active = true; if (!application?.supports("case.capabilities")) return;
    void application.caseCapabilities({ case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref }).then(response => {
      if (!active) return;
      const view = response.data;
      const matches = view?.case_id === workspace.case.case_ref && view.case_generation === workspace.case.generation && view.participant_id === workspace.case.participant_ref;
      setResult(response.result_state === "success" && matches ? { identity, view } : { identity, error: response.error?.safe_message ?? "The capability projection did not match the current Case/Participant/generation." });
    }).catch(() => { if (active) setResult({ identity, error: "The current capability projection is unavailable. Reconnect or refresh the Case." }); }); return () => { active = false; };
  }, [application, availability, identity, workspace.case.case_ref, workspace.case.generation, workspace.case.participant_ref]);
  const current = result?.identity === identity ? result : undefined;
  const entries = current?.view?.entries.filter(entry => entry.resource.attachment_id === resourceRef) ?? [];
  const excluded = current?.view?.exclusions.filter(entry => entry.resource_id === resourceRef) ?? [];
  return <section className="work-section resource-requestability"><h2>Current requestability</h2><p>Participant: {workspace.case.participant_ref}. Requestable does not mean authorized; each exact operation still requires a current Decision.</p>
    {!application?.supports("case.capabilities") ? <p>{application?.reason("case.capabilities") ?? "Not exposed by this data source."}</p> : !current ? <p role="status">Checking the current YAI projection…</p> : current.error ? <p role="status">{current.error}</p> : <>
      {entries.map(entry => <details key={resourceOperationLabel(entry.operation_kind)}><summary>{resourceOperationLabel(entry.operation_kind)} <Badge tone="info">Requestable</Badge></summary><p>Current Decision required: {entry.requires_current_decision ? "Yes" : "No, as projected"}</p>{entry.policy_constraints.map((rule, index) => <p key={index}>{rule.kind.replaceAll("_", " ")} · {rule.effect ?? (rule.required == null ? "Current constraint" : rule.required ? "Required" : "Not required")} · {rule.resolution}</p>)}</details>)}
      {excluded.map(entry => <p className="policy-refusal" key={resourceOperationLabel(entry.operation_kind)}>{resourceOperationLabel(entry.operation_kind)} · {entry.reason}</p>)}
      {!entries.length && !excluded.length && <p>No current entry for this Resource is disclosed.</p>}
      <details><summary>Qualification basis</summary><code>{current.view?.effective_policy_id}</code><p>Generation {current.view?.case_generation} · {current.view?.view_id}</p></details>
    </>}
  </section>;
}
