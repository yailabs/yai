import { useEffect, useState } from "react";
import type { IdentityPresentation, TenantPresentation } from "../../clients/application";
import type { OperationResult } from "../../clients/live";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { EmptyState } from "../../components/primitives";
import { useApplicationAvailability } from "../case/applicationActions";

export function IdentitySettings({ workspace, platform }: Pick<SurfaceRendererProps, "workspace" | "platform">) {
  const availability = useApplicationAvailability(platform.application);
  const [result, setResult] = useState<OperationResult<IdentityPresentation>>();
  const [tenant, setTenant] = useState<TenantPresentation>();
  useEffect(() => {
    let current = true; setResult(undefined); setTenant(undefined);
    if (availability.state === "available" && platform.application) void platform.application.identity().then(value => { if (current) setResult(value); });
    if (availability.state === "available" && platform.application?.supports("tenant.get") && workspace.case.tenant_ref) void platform.application.tenant({ tenant_id: workspace.case.tenant_ref }).then(value => { if (current && value.result_state === "success") setTenant(value.data); });
    return () => { current = false; };
  }, [platform.application, availability, workspace.case.case_ref]);
  const person = workspace.overview.participants.find(item => item.id === workspace.case.participant_ref);
  return <section className="identity-settings" aria-label="Local identity">
    <p>The local authenticated Principal and the Participant linked to this Case have different roles. Connecting to the Host grants no Case authority.</p>
    {result?.data ? <dl className="object-facts"><div><dt>Principal</dt><dd>{result.data.principal.principal_id}</dd></div><div><dt>Authentication</dt><dd>{result.data.principal.authentication_method}</dd></div><div><dt>Case Tenant</dt><dd>{workspace.case.tenant_ref ?? "Not projected"}</dd></div><div><dt>Tenant membership</dt><dd>{result.data.tenants.find(item => item.tenant.tenant_id === workspace.case.tenant_ref)?.membership ?? "Not projected"}</dd></div></dl> : <EmptyState title={availability.state === "checking" ? "Checking identity" : "Principal identity unavailable"} body={result?.error?.safe_message ?? availability.reason ?? "The current data source does not expose authenticated identity."} />}
    <dl className="object-facts"><div><dt>Case Participant</dt><dd>{workspace.case.participant_ref}</dd></div><div><dt>Current roles</dt><dd>{person?.roles.join(", ") || "No roles projected"}</dd></div></dl>
    {tenant && <p>Organization: {tenant.tenant.organization_ref}. Your Tenant membership: {tenant.membership}.</p>}
    <p className="surface-note">Tenant membership changes require an already enrolled Principal. Principal enrollment and a subject picker are not yet available here; Case roles remain managed in Overview.</p>
  </section>;
}
