import { useEffect, useRef, useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import { Badge, Button } from "../../components/primitives";

/** Ephemeral catalog observation, never qualification, residency or authority. */
export function DeploymentModelObservation({ application, tenant, target, model }: {
  application?: ApplicationAccess; tenant: string; target: string; model: string;
}) {
  const [state, setState] = useState<{ target: string; models?: string[]; at?: number; error?: string }>();
  const [pending, setPending] = useState(false);
  const epoch = useRef(0);
  useEffect(() => () => { epoch.current++; }, [application, tenant, target]);
  const visible = state?.target === target ? state : undefined;
  const check = async () => {
    if (!application) return;
    const request = ++epoch.current;
    setPending(true); setState(undefined);
    try {
      const result = await application.discoverProviderModels({tenant_id: tenant, target_ref: target});
      if (request !== epoch.current) return;
      if (result.result_state === "success" && result.data?.target_ref === target) {
        setState({target, models: result.data.models, at: result.data.observed_at_unix_ms});
      } else setState({target, error: result.error?.code.startsWith("provider_catalog_empty") ? "No models exposed. YAI refused this empty catalog; no qualification or binding was performed." : result.error?.safe_message ?? "This Host cannot return a catalog for the selected target."});
    } catch { if (request === epoch.current) setState({target, error: "Catalog observation unavailable. Check the Host connection and retry."}); }
    finally { if (request === epoch.current) setPending(false); }
  };
  const exposed = visible?.models?.includes(model);
  return <section className="deployment-model-observation" aria-label="Exposed model observation">
    <header><h3>Exposed model</h3><Button disabled={pending || !application?.supports("provider.models")} onClick={check}>{pending ? "Checking catalog…" : "Check exposed model"}</Button></header>
    {visible?.error ? <p role="alert">{visible.error}</p> : visible?.models ? <>
      <p role="status"><Badge tone={exposed ? "success" : "warning"}>{exposed ? "Model exposed" : "Model not exposed"}</Badge></p>
      <p>{exposed ? "The exact registered model appeared in the endpoint catalog." : "The endpoint answered, but its catalog did not list this exact model."}</p>
      <small>Observed {visible.at ? new Date(visible.at).toLocaleString() : "at an unknown time"} · {visible.models.length} exposed model(s)</small>
    </> : <p>Check the registered connection through YAI. No prompt is sent.</p>}
    <p>Catalog visibility is not proof of loaded engine residency, available capacity or successful inference.</p>
  </section>;
}
