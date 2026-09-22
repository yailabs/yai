import { useState } from "react";
import type { ApplicationAccess, SourceDeclarationInput } from "../../clients/application";
import type { CasePresentation } from "../../clients/dataSource";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

const actions = [
  { id: "discover", operation: "discovery.enumerate", label: "Discover a file or subtree", field: "Relative path" },
  { id: "database_query", operation: "database.query", label: "Named database query", field: "Qualified query name" },
  { id: "http_fetch", operation: "http.fetch", label: "Named HTTP request", field: "Qualified request name" },
] as const;
type Action = typeof actions[number]["id"];

export function DeclareSourceAction({ application, workspace, resourceRef, refresh }: {
  application?: ApplicationAccess; workspace: CasePresentation; resourceRef?: string; refresh(): void | Promise<void>;
}) {
  useApplicationAvailability(application);
  const [open, setOpen] = useState(false);
  const resources = workspace.environment.resources.filter(resource => resource.operations.some(operation => actions.some(action => action.operation === operation)));
  const enabled = Boolean(application?.supports("source.declare") && workspace.case.case_status === "open" && resources.length);
  if (!application) return null;
  return <div className="environment-actions"><Button disabled={!enabled} title={!application.supports("source.declare") ? application.reason("source.declare") : !resources.length ? "No attached Resource exposes a supported Source operation." : undefined} onClick={() => setOpen(true)}>Declare Source</Button>
    {!resources.length && <span>Attach a Resource through YAI before declaring its Sources.</span>}
    {open && <DeclareSourceDialog application={application} workspace={workspace} resourceRef={resourceRef} resources={resources} close={() => setOpen(false)} refresh={refresh} />}
  </div>;
}

function DeclareSourceDialog({ application, workspace, resourceRef, resources, close, refresh }: {
  application: ApplicationAccess; workspace: CasePresentation; resourceRef?: string;
  resources: CasePresentation["environment"]["resources"]; close(): void; refresh(): void | Promise<void>;
}) {
  const [selected, setSelected] = useState(resources.some(resource => resource.id === resourceRef) ? resourceRef! : resources[0].id);
  const resource = resources.find(item => item.id === selected)!;
  const offered = actions.filter(action => resource.operations.includes(action.operation));
  const [chosen, setChosen] = useState<Action>(offered[0].id);
  const action = offered.find(item => item.id === chosen) ?? offered[0];
  const [roles, setRoles] = useState<SourceDeclarationInput["roles"]>(["knowledge"]);
  const [bootstrap, setBootstrap] = useState(false);
  const policyBootstrap = roles.includes("policy") && action.id === "discover" && bootstrap;
  return <ApplicationActionDialog title="Declare Source" description="Define a governed relationship to material through an attached Resource. This records the declaration; it does not acquire bytes, publish policy or grant permission to execute." submitLabel="Declare Source" close={close} enabled={application.supports("source.declare") && roles.length > 0} submit={form => application.declareSource({
    case_ref: workspace.case.case_ref, participant_ref: workspace.case.participant_ref, resource_ref: selected,
    logical_name: String(form.get("name")).trim(), perimeter: String(form.get("perimeter")).trim(), roles,
    action: action.id === "discover" ? { action: "discover", path: String(form.get("target")).trim() } : { action: action.id, name: String(form.get("target")).trim() },
    bootstrap_policy: policyBootstrap, media_type: String(form.get("media")).trim(),
  })} committed={refresh}>
    <label>Name<input name="name" autoFocus required placeholder="studio-readme" /></label>
    <label>Perimeter label<input name="perimeter" required placeholder="repository-documentation" /></label>
    <small>The perimeter label groups declarations; the Resource scope still confines access.</small>
    <label>Resource<select aria-label="Resource" value={selected} onChange={event => setSelected(event.target.value)}>{resources.map(item => <option key={item.id} value={item.id}>{item.label ?? item.id}</option>)}</select></label>
    <label>Operation<select aria-label="Operation" value={action.id} onChange={event => setChosen(event.target.value as Action)}>{offered.map(item => <option key={item.id} value={item.id}>{item.label}</option>)}</select></label>
    {action.id === "discover" ? <label>{action.field}<input key={selected} name="target" required placeholder="studio/README.md" /></label> : <label>{action.field}<select aria-label={action.field} key={selected} name="target" required><option value="">Select an exposed name</option>{resource.names.map(name => <option key={name}>{name}</option>)}</select></label>}
    {resource.read_prefixes.length > 0 && <p className="action-scope">Resource read scope: {resource.read_prefixes.join(", ")}</p>}
    <label>Qualified media type<input name="media" required placeholder="text/markdown" /></label>
    <fieldset className="source-role-options"><legend>Source roles</legend>{(["knowledge", "operational", "policy"] as const).map(role => <label key={role}><input type="checkbox" checked={roles.includes(role)} onChange={event => setRoles(current => event.target.checked ? [...current, role] : current.filter(item => item !== role))} />{role}</label>)}</fieldset>
    {roles.includes("policy") && action.id === "discover" && <label className="checkbox-setting"><input type="checkbox" checked={bootstrap} onChange={event => setBootstrap(event.target.checked)} />Policy bootstrap (policy material only)</label>}
    <small>YAI checks Participant, Resource scope and declaration validity. Acquisition is a separate governed operation.</small>
  </ApplicationActionDialog>;
}

export function RevokeSourceAction({ application, workspace, sourceRef, refresh }: {
  application?: ApplicationAccess; workspace: CasePresentation; sourceRef: string; refresh(): void | Promise<void>;
}) {
  useApplicationAvailability(application);
  const [open, setOpen] = useState(false);
  const source = workspace.environment.sources.find(item => item.id === sourceRef);
  if (!application || !source) return null;
  const enabled = application.supports("source.revoke") && workspace.case.case_status === "open" && source.posture !== "revoked";
  return <div className="environment-actions"><Button disabled={!enabled} title={!application.supports("source.revoke") ? application.reason("source.revoke") : source.posture === "revoked" ? "This Source is already revoked." : undefined} onClick={() => setOpen(true)}>Revoke Source</Button>
    {open && <ApplicationActionDialog title="Revoke Source" description={`${source.label}: revoke this governed Source relationship. Retained history is not erased and the underlying file is not deleted. YAI validates current authority.`} submitLabel="Revoke Source" close={() => setOpen(false)} enabled={enabled} submit={form => application.revokeSource({ case_ref: workspace.case.case_ref, source_ref: sourceRef, reason: String(form.get("reason")).trim() })} committed={refresh}><label>Reason<textarea name="reason" autoFocus required rows={3} /></label></ApplicationActionDialog>}
  </div>;
}
