import { useCallback, useEffect, useState, useSyncExternalStore } from "react";
import type { ApplicationAccess, ApplicationAvailability, TenantPresentation } from "../../clients/application";
import type { CasePresentation } from "../../clients/dataSource";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";

const absent: ApplicationAvailability = { state: "unavailable", reason: "This data source does not expose live application actions." };
export function useApplicationAvailability(application?: ApplicationAccess) {
  const subscribe = useCallback((listener: () => void) => application ? application.subscribe(listener).dispose : () => undefined, [application]);
  return useSyncExternalStore(subscribe, application?.snapshot ?? (() => absent));
}

export function NewCaseDialog({ application, close, created }: { application: ApplicationAccess; close(): void; created(caseRef: string): void | Promise<void> }) {
  useApplicationAvailability(application);
  const [tenants, setTenants] = useState<TenantPresentation[]>([]);
  const [error, setError] = useState<string>();
  const [loading, setLoading] = useState(true);
  const [caseRef, setCaseRef] = useState("");
  const [bootstrap, setBootstrap] = useState(false);
  const [revision, setRevision] = useState(0);
  useEffect(() => { let active = true; void application.tenants().then(result => {
    if (!active) return;
    if (result.result_state === "success" && Array.isArray(result.data) && result.data.every(item => typeof item?.tenant?.tenant_id === "string" && typeof item.membership === "string")) setTenants(result.data);
    else setError(result.error?.safe_message ?? "Authorized Tenants are unavailable.");
    setLoading(false);
  }); return () => { active = false; }; }, [application, revision]);
  if (bootstrap) return <ApplicationActionDialog title="Set up local identity" description="Enroll this authenticated local Principal and establish the Tenant/organization through YAI. No cloud account or Case is created by this action." submitLabel="Set up identity" close={() => setBootstrap(false)} enabled={application.supports("identity.bootstrap")} submit={form => application.bootstrapIdentity({ tenant_id: String(form.get("tenant")).trim(), organization_ref: String(form.get("organization")).trim() })} committed={() => { setError(undefined); setLoading(true); setRevision(value => value + 1); }}><label>Tenant reference<input autoFocus required name="tenant" placeholder="tenant:my-work" /></label><label>Organization reference<input required name="organization" placeholder="organization:my-work" /></label></ApplicationActionDialog>;
  return <ApplicationActionDialog title="New Case" description="Create durable Case continuity in one of your authorized Tenants. Sources, Resources and provider bindings are configured separately." submitLabel="Create Case" close={close} enabled={application.supports("case.create") && tenants.length > 0} submit={form => application.createCase({ tenant_id: String(form.get("tenant")), case_ref: caseRef.trim() })} committed={() => created(caseRef.trim())}>
    <label>Tenant<select name="tenant" required aria-label="Tenant">{tenants.map(({ tenant, membership }) => <option key={tenant.tenant_id} value={tenant.tenant_id}>{tenant.tenant_id} · {membership}</option>)}</select></label>
    <label>Case reference<input autoFocus required name="case" value={caseRef} onChange={event => setCaseRef(event.target.value)} placeholder="case:research-project" pattern="case:[A-Za-z0-9_.:-]+" /></label>
    <small>YAI derives the display name from this durable reference.</small>
    {error && <p role="alert">{error}</p>}{loading ? <p role="status">Loading authorized Tenants…</p> : !tenants.length && !error && <p>No authorized Tenant is available yet.</p>}
    {!loading && !tenants.length && application.supports("identity.bootstrap") && <Button onClick={() => setBootstrap(true)}>Set up local identity…</Button>}
  </ApplicationActionDialog>;
}

export function CaseLifecycleDialog({ application, workspace, action, close, refresh }: {
  application: ApplicationAccess; workspace: CasePresentation; action: "close" | "cancel"; close(): void; refresh(): void | Promise<void>;
}) {
  useApplicationAvailability(application);
  return <ApplicationActionDialog title={action === "close" ? "Close Case" : "Cancel Case"} description={`${workspace.case.display_name}: this changes the durable Case lifecycle. Closing a Studio tab or window is a separate action. YAI validates the current state and authority.`} submitLabel={action === "close" ? "Close this Case" : "Cancel this Case"} close={close} enabled={application.supports(`case.${action}`)} submit={form => application.caseLifecycle(action, { case_ref: workspace.case.case_ref, reason: String(form.get("reason")).trim() })} committed={refresh}>
    <label>Reason<textarea name="reason" required rows={3} autoFocus /></label>
  </ApplicationActionDialog>;
}

export function ParticipantAccessDialog({ application, caseRef, close, attached }: { application: ApplicationAccess; caseRef: string; close(): void; attached(): void | Promise<void> }) {
  useApplicationAvailability(application);
  const [step, setStep] = useState<"participant" | "link">("participant");
  const [participant, setParticipant] = useState("participant:operator");
  return <ApplicationActionDialog key={step} title="Connect your Participant" description={`${caseRef} exists, but your authenticated identity is not linked to a Participant. Each step is a separate governed YAI action; creating or linking a Participant does not grant resource execution permission.`} submitLabel={step === "participant" ? "Add Participant role" : "Link my identity and open"} close={close} dismissOnSuccess={step === "link"} enabled={application.supports(step === "participant" ? "participant.role.add" : "participant.principal.link")} submit={form => step === "participant" ? application.addParticipantRole({ case_ref: caseRef, participant_ref: participant.trim(), role: String(form.get("role")).trim() }) : application.linkCurrentPrincipal({ case_ref: caseRef, participant_ref: participant.trim() })} committed={step === "participant" ? () => setStep("link") : attached}>
    <label>Participant reference<input autoFocus required value={participant} onChange={event => setParticipant(event.target.value)} pattern="participant:[A-Za-z0-9_.:-]+" /></label>
    {step === "participant" ? <><label>Role<input required name="role" defaultValue="operator" /></label><Button type="button" onClick={() => setStep("link")}>Link an existing Participant</Button></> : <p>Link the local identity authenticated by the Host to this exact Participant. YAI checks Tenant authority.</p>}
  </ApplicationActionDialog>;
}

export function ReviewActions({ application, workspace, reviewRef, refresh }: { application?: ApplicationAccess; workspace: CasePresentation; reviewRef: string; refresh(): void | Promise<void> }) {
  useApplicationAvailability(application);
  const [action, setAction] = useState<"approve" | "deny" | "defer">();
  const review = workspace.authority.reviews.find(item => item.id === reviewRef);
  const pending = workspace.case.case_status === "open" && (review?.status === "pending" || review?.status === "deferred");
  if (!application) return null;
  return <><div className="object-action-row">{(["approve", "deny", "defer"] as const).map(name => <Button key={name} disabled={!pending || !application.supports(`review.${name}`)} title={!pending ? "This Review is not pending in an open Case." : application.supports(`review.${name}`) ? undefined : application.reason(`review.${name}`)} onClick={() => setAction(name)}>{name[0].toUpperCase() + name.slice(1)} review</Button>)}</div>{action && <ApplicationActionDialog title={`${action[0].toUpperCase() + action.slice(1)} review`} description="YAI checks your current participant authority and the exact review. Resolving a review does not itself execute an external effect." submitLabel="Submit decision" close={() => setAction(undefined)} enabled={pending && application.supports(`review.${action}`)} submit={form => application.resolveReview(action, { case_ref: workspace.case.case_ref, review_ref: reviewRef, participant_ref: workspace.case.participant_ref, reason: String(form.get("reason")).trim() })} committed={refresh}><label>Reason<textarea autoFocus required name="reason" rows={3} /></label></ApplicationActionDialog>}</>;
}

export function WorkflowInputAction({ application, workspace, nodeRef, refresh }: { application?: ApplicationAccess; workspace: CasePresentation; nodeRef: string; refresh(): void | Promise<void> }) {
  useApplicationAvailability(application);
  const [open, setOpen] = useState(false);
  const definition = workspace.work.definition?.nodes?.find((node): node is { node_id: string; kind: string; prompt?: string; max_bytes?: number } => Boolean(node && typeof node === "object" && "node_id" in node && node.node_id === nodeRef && "kind" in node && node.kind === "human_input"));
  if (!application) return null;
  return <><Button disabled={!application.supports("workflow.input.record")} title={application.supports("workflow.input.record") ? undefined : application.reason("workflow.input.record")} onClick={() => setOpen(true)}>Provide input</Button>{open && <ApplicationActionDialog title="Workflow input" description={`Record participant input for ${nodeRef}. YAI checks the current Workflow node and admission rules.`} submitLabel="Record input" close={() => setOpen(false)} enabled={application.supports("workflow.input.record")} submit={form => application.recordWorkflowInput({ case_ref: workspace.case.case_ref, node_ref: nodeRef, value: String(form.get("value")) })} committed={refresh}>{typeof definition?.prompt === "string" && <p>{definition.prompt}</p>}<label>Input<textarea autoFocus required name="value" rows={5} /></label>{typeof definition?.max_bytes === "number" && <small>Maximum {definition.max_bytes} UTF-8 bytes, validated by YAI.</small>}</ApplicationActionDialog>}</>;
}
