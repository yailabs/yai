import { useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { CasePresentation } from "../../clients/dataSource";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

type PolicyAction = "bind" | "replace" | "unbind";
const titles: Record<PolicyAction, string> = { bind: "Bind Policy", replace: "Replace Policy", unbind: "Unbind Policy" };

export function PolicyActions({ application, workspace, bindingRef, artifactRef, refresh }: {
  application?: ApplicationAccess; workspace: CasePresentation; bindingRef?: string; artifactRef?: string; refresh(): void | Promise<void>;
}) {
  useApplicationAvailability(application);
  const [pending, setPending] = useState<{ action: PolicyAction; generation: number }>();
  const binding = workspace.authority.policies.find(policy => policy.id === bindingRef);
  if (!application || (bindingRef && !binding)) return null;
  const offered: PolicyAction[] = binding ? ["replace", "unbind"] : ["bind"];
  const enabled = (action: PolicyAction) => workspace.case.case_status === "open" && application.supports(`policy.case.${action}`);
  return <div className="object-action-row">{offered.map(action => <Button key={action} disabled={!enabled(action)} title={application.supports(`policy.case.${action}`) ? undefined : application.reason(`policy.case.${action}`)} onClick={() => setPending({ action, generation: workspace.case.generation })}>{titles[action]}</Button>)}
    {pending && <ApplicationActionDialog title={titles[pending.action]} description={pending.action === "unbind" ? `Remove ${binding?.policy_key ?? "this policy"} from the current Case. This changes effective authority; the policy artifact and history remain retained.` : pending.action === "replace" ? `Replace the current ${binding?.policy_key ?? "policy"} binding with a published artifact from the same lineage. YAI checks compatibility and authority.` : "Bind an already-published policy artifact to this Case. YAI checks Tenant ownership, publication and current authority. This form does not publish policy."} submitLabel={titles[pending.action]} close={() => setPending(undefined)} enabled={enabled(pending.action) && pending.generation === workspace.case.generation} submit={form => {
      const common = { case_ref: workspace.case.case_ref, expected_generation: pending.generation, reason: String(form.get("reason")).trim() };
      if (pending.action === "unbind") return application.unbindPolicy({ ...common, binding_ref: bindingRef! });
      const artifact_ref = String(form.get("artifact")).trim();
      return pending.action === "replace" ? application.replacePolicy({ ...common, artifact_ref, prior_binding_ref: bindingRef! }) : application.bindPolicy({ ...common, artifact_ref });
    }} committed={refresh} resync={refresh}>
      {binding && <p className="action-scope">Current binding: {binding.policy_key} · version {binding.version}</p>}
      {pending.action !== "unbind" && <label>Published policy artifact<input name="artifact" autoFocus required defaultValue={artifactRef} placeholder="Exact artifact reference from YAI" /></label>}
      <label>Reason<textarea name="reason" autoFocus={pending.action === "unbind"} required rows={3} /></label>
      <small>Based on Case generation {pending.generation}. The backend refuses changes based on stale state.</small>
      {pending.generation !== workspace.case.generation && <p role="alert">The Case changed while this action was open. Close and reopen the action after inspecting the current binding.</p>}
    </ApplicationActionDialog>}
  </div>;
}
