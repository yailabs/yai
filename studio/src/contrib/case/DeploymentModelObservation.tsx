import { useCallback, useSyncExternalStore } from "react";
import type { ApplicationAccess } from "../../clients/application";
import { providerCatalogKey } from "../../clients/compute";
import { Badge, Button } from "../../components/primitives";

/** One shared observation for this window's workspace and status bar. */
export function DeploymentModelObservation({ application, tenant, target, model }: {
  application?: ApplicationAccess; tenant: string; target: string; model: string;
}) {
  const availability = useSyncExternalStore(
    useCallback(listener => application?.subscribe(listener).dispose ?? (() => {}), [application]),
    useCallback(() => application?.snapshot(), [application]));
  const observation = availability?.providerCatalogs?.[providerCatalogKey(tenant, target)];
  const pending = observation?.state === "checking";
  const observed = observation?.state === "observed" ? observation : undefined;
  const unavailable = observation?.state === "unavailable" ? observation : undefined;
  const check = () => { void application?.discoverProviderModels({tenant_id: tenant, target_ref: target}).catch(() => {}); };
  const exposed = observed?.models.includes(model);
  return <section className="deployment-model-observation" aria-label="Exposed model observation">
    <header><h3>Exposed model</h3><Button disabled={pending || !application?.supports("provider.models")} onClick={check}>{pending ? "Checking catalog…" : "Check exposed model"}</Button></header>
    {unavailable ? <p role="alert">{unavailable.empty ? "No models exposed. YAI refused this empty catalog; no qualification or binding was performed." : unavailable.reason}</p> : observed ? <>
      <p role="status"><Badge tone={exposed ? "success" : "warning"}>{exposed ? "Model exposed" : "Model not exposed"}</Badge></p>
      <p>{exposed ? "The exact registered model appeared in the endpoint catalog." : "The endpoint answered, but its catalog did not list this exact model."}</p>
      <small>Observed {new Date(observed.at).toLocaleString()} · {observed.models.length} exposed model(s)</small>
    </> : <p>Check the registered connection through YAI. No prompt is sent.</p>}
    <p>Catalog visibility is not proof of loaded engine residency, available capacity or successful inference.</p>
  </section>;
}
