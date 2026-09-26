import { useCallback, useEffect, useState, useSyncExternalStore } from "react";
import type { ApplicationAccess } from "../../clients/application";
import { isCurrentProviderCatalog, providerCatalogKey } from "../../clients/compute";
import { Badge, Button } from "../../components/primitives";

/** One shared observation for this window's workspace and status bar. */
export function DeploymentModelObservation({ application, tenant, target, model }: {
  application?: ApplicationAccess; tenant: string; target: string; model: string;
}) {
  const availability = useSyncExternalStore(
    useCallback(listener => application?.subscribe(listener).dispose ?? (() => {}), [application]),
    useCallback(() => application?.snapshot(), [application]));
  const observation = availability?.providerCatalogs?.[providerCatalogKey(tenant, target)];
  const [clock, setClock] = useState(() => Date.now());
  useEffect(() => {
    if (observation?.state !== "observed") return;
    const update = () => setClock(Date.now());
    update();
    const timer = window.setInterval(update, 10_000);
    document.addEventListener("visibilitychange", update);
    return () => { window.clearInterval(timer); document.removeEventListener("visibilitychange", update); };
  }, [observation]);
  const pending = observation?.state === "checking";
  const observed = observation?.state === "observed" ? observation : undefined;
  const current = isCurrentProviderCatalog(observation, clock);
  const unavailable = observation?.state === "unavailable" ? observation : undefined;
  const check = () => { void application?.discoverProviderModels({tenant_id: tenant, target_ref: target}).catch(() => {}); };
  const exposed = current && observed?.models.includes(model);
  const capacity = current && observed?.capacity?.model_id === model && exposed ? observed.capacity : undefined;
  return <section className="deployment-model-observation" aria-label="Exposed model observation">
    <header><h3>Exposed model</h3><Button disabled={pending || !application?.supports("provider.models")} onClick={check}>{pending ? "Checking catalog…" : "Check exposed model"}</Button></header>
    {unavailable ? <p role="alert">{unavailable.empty ? "No models exposed. YAI refused this empty catalog; no qualification or binding was performed." : unavailable.reason}</p> : observed ? <>
      <p role="status"><Badge tone={current ? exposed ? "info" : "error" : "warning"}>{current ? exposed ? "Model exposed" : "Model not exposed" : "Catalog old"}</Badge></p>
      <p>{!current ? "The last model-list observation is over one minute old. This alone does not block SEND or explain a failed execution; YAI qualification and health are separate. Check the exposed model again." : exposed ? "The exact registered model appeared in the endpoint catalog." : "The endpoint answered, but its catalog did not list this exact model."}</p>
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
