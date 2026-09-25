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
  const capacity = observed?.capacity?.model_id === model && exposed ? observed.capacity : undefined;
  return <section className="deployment-model-observation" aria-label="Exposed model observation">
    <header><h3>Exposed model</h3><Button disabled={pending || !application?.supports("provider.models")} onClick={check}>{pending ? "Checking catalog…" : "Check exposed model"}</Button></header>
    {unavailable ? <p role="alert">{unavailable.empty ? "No models exposed. YAI refused this empty catalog; no qualification or binding was performed." : unavailable.reason}</p> : observed ? <>
      <p role="status"><Badge tone={exposed ? "success" : "warning"}>{exposed ? "Model exposed" : "Model not exposed"}</Badge></p>
      <p>{exposed ? "The exact registered model appeared in the endpoint catalog." : "The endpoint answered, but its catalog did not list this exact model."}</p>
      {capacity && <div className="deployment-capacity" aria-label="Observed public model capacity">
        <div><strong>{capacity.input_capacity_tokens.toLocaleString()}</strong><span>Input tokens</span></div>
        <div><strong>{capacity.sequence_capacity_tokens.toLocaleString()}</strong><span>Sequence ceiling</span></div>
      </div>}
      <small>Observed {new Date(observed.at).toLocaleString()} · {observed.models.length} exposed model(s)</small>
      {capacity && <small>YVEX public catalog · exact model and engine generation {capacity.engine_generation}. This observation does not reserve capacity or prove a later request will fit.</small>}
    </> : <p>Check the registered connection through YAI. No prompt is sent.</p>}
    <p>Catalog visibility is not proof of current engine residency, resource availability or successful inference.</p>
  </section>;
}
