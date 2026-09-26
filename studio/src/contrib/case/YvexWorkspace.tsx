import { useCallback, useEffect, useState, useSyncExternalStore, type ComponentProps } from "react";
import { MachineRegistry } from "./MachineRegistry";
import { ProviderWorkspace } from "./ProviderWorkspace";
import { isCurrentProviderCatalog, providerCatalogKey } from "../../clients/compute";
import { Badge, Button, EmptyState } from "../../components/primitives";
import { Icon } from "../../components/Icon";

type Props = Omit<ComponentProps<typeof ProviderWorkspace>, "scope">;
type Section = "Overview" | "Models" | "Machines" | "Lifecycle" | "Connection" | "Observability";
const sections: Section[] = ["Overview", "Models", "Machines", "Lifecycle", "Connection", "Observability"];
const at = (value?: number) => value ? new Date(value).toLocaleString() : "Not observed";

/** First-party YVEX workspace. Native stages stay separate from YAI provider compatibility. */
export function YvexWorkspace(props: Props) {
  const { application, workspace, targets, actions, qualification, controls } = props;
  const [section, setSection] = useState<Section>("Overview");
  const [selected, setSelected] = useState<string>();
  const [machines, setMachines] = useState<number>();
  const [machineError, setMachineError] = useState<string>();
  const [checking, setChecking] = useState(false);
  const [clock, setClock] = useState(() => Date.now());
  const tenant = workspace.case.tenant_ref;
  const target = targets.find(item => item.id === selected) ?? targets[0];
  const availability = useSyncExternalStore(
    useCallback(listener => application?.subscribe(listener).dispose ?? (() => {}), [application]),
    useCallback(() => application?.snapshot(), [application]));
  const catalog = target && tenant ? availability?.providerCatalogs?.[providerCatalogKey(tenant, target.id)] : undefined;
  useEffect(() => { const timer = window.setInterval(() => setClock(Date.now()), 10_000); return () => window.clearInterval(timer); }, []);
  useEffect(() => {
    if (!tenant || !application?.supports("machine.list")) return;
    let alive = true;
    void application.listMachines(tenant).then(result => {
      if (!alive) return;
      if (result.result_state === "success" && Array.isArray(result.data)) { setMachines(result.data.filter(item => !item.revocation).length); setMachineError(undefined); }
      else { setMachines(undefined); setMachineError(result.error?.safe_message ?? "Machine identities unavailable."); }
    });
    return () => { alive = false; };
  }, [application, tenant, section]);
  const fresh = isCurrentProviderCatalog(catalog, clock);
  const exposed = fresh && catalog?.state === "observed" && Boolean(target && catalog.models.includes(target.model_id));
  const posture = typeof target?.posture === "object" ? target.posture : undefined;
  const health = posture?.health;
  const capability = posture?.qualification?.capabilities.some(item => item.capability === "chattext");
  const bound = Boolean(target && workspace.compute.targets.some(item => item.id === target.id));
  const routeBlocked = Boolean(target && bound && (!capability || health?.circuit === "open"));
  const checkModel = async () => {
    if (!tenant || !target || !application || checking) return;
    setChecking(true);
    try { await application.discoverProviderModels({ tenant_id: tenant, target_ref: target.id }); }
    finally { setChecking(false); }
  };
  const modelState = !catalog ? "Not checked" : catalog.state === "checking" ? "Checking" : catalog.state === "observed" && !fresh ? "Catalog old" : exposed ? "Exposed" : catalog.state === "unavailable" ? "Check failed" : "Not exposed";
  return <div className="yvex-workspace" data-surface-type="platform.yvex">
    <header className="yvex-header"><div><span className="yvex-eyebrow">Computational platform</span><h1><Icon name="processor" /> YVEX</h1><p>From verified model material to a serving engine, with Case use governed by YAI.</p></div><div className="yvex-header-actions"><Button onClick={props.connect} disabled={!props.canConnect}>Connect deployment</Button><Button onClick={() => { props.refresh(); setClock(Date.now()); }} disabled={props.refreshing}>{props.refreshing ? "Refreshing…" : "Refresh"}</Button></div></header>
    <nav className="yvex-sections" aria-label="YVEX workspace sections">{sections.map(name => <button type="button" key={name} aria-current={section === name ? "page" : undefined} onClick={() => setSection(name)}>{name}</button>)}</nav>
    <div className={section === "Connection" ? "yvex-body yvex-body-connection" : "yvex-body"}>
      {section === "Overview" && <>
        <div className="yvex-overview-heading"><div><span className="yvex-eyebrow">Current YAI connection</span><h2>{target?.provider_key ?? "No compatible deployment"}</h2><p>{target ? `Model endpoint ${target.endpoint}` : "Connect an OpenAI-compatible YVEX target to use it in a Case."}</p></div>{target && <Badge tone={routeBlocked ? "warning" : health?.effective_posture === "healthy" ? "success" : "neutral"}>{routeBlocked ? "Case route blocked" : health?.effective_posture ?? "Health unknown"}</Badge>}</div>
        <div className="yvex-readout" aria-label="YVEX operational boundaries"><div><span>Machine identity</span><strong>{machines === undefined ? "Unknown" : `${machines} pinned`}</strong><small>{machineError ?? "Tenant registrations; not a runtime status"}</small></div><div><span>Serving model</span><strong>{modelState}</strong><small>{catalog?.state === "observed" ? `Catalog observed ${at(catalog.at)}` : "A model list is not engine telemetry"}</small></div><div><span>Current Case route</span><strong>{!bound ? "Not bound" : routeBlocked ? "Blocked" : "Selectable"}</strong><small>{target ? "A small check cannot guarantee a Case answer" : "Choose a deployment in Providers"}</small></div><div><span>Native YVEX host</span><strong>Not connected</strong><small>No server logs, load state or device memory from this HTTP target</small></div></div>
        <div className="yvex-path" aria-label="Model lifecycle"><h3>Model lifecycle</h3><ol>{[["Acquire", "Verified source"], ["Prepare", "Package + artifact"], ["Compile", "Deployment specialization"], ["Load", "Engine generation"], ["Serve", "Session + result"]].map(([name, detail]) => <li key={name}><strong>{name}</strong><span>{detail}</span><Badge tone={name === "Serve" && exposed ? "info" : "neutral"}>{name === "Serve" && exposed ? "Catalog visible" : "Not projected"}</Badge></li>)}</ol><p>The endpoint exposes its model catalog and YAI provider observations. It does not expose the source, artifact, compiler or host lifecycle.</p></div>
        <div className="yvex-next"><Button onClick={() => setSection("Models")}>View serving models</Button><Button onClick={() => setSection("Machines")}>Machine identities</Button><Button onClick={() => setSection("Connection")}>Connection &amp; checks</Button><Button onClick={() => actions.openPerspective("Compute")}>Case Compute</Button></div>
      </>}
      {section === "Models" && <section className="yvex-models"><header className="yvex-section-heading"><div><h2>Models</h2><p>Exact identities observed through registered YAI-compatible endpoints.</p></div><Button onClick={() => void checkModel()} disabled={!target || checking || !application?.supports("provider.models")}>{checking ? "Checking…" : "Check exposed model"}</Button></header>
        {!targets.length && <EmptyState title="No YVEX-compatible deployment" body="Connect a deployment in Providers. Model acquisition and compilation require a native management connection." />}
        {!!targets.length && <div className="yvex-model-table" role="table" aria-label="Serving model identities"><div role="row" className="yvex-model-table-head"><span role="columnheader">Deployment / exact model</span><span role="columnheader">Catalog</span><span role="columnheader">Input capacity</span><span role="columnheader">Case</span></div>{targets.map(item => {
          const observation = tenant ? availability?.providerCatalogs?.[providerCatalogKey(tenant, item.id)] : undefined;
          const current = isCurrentProviderCatalog(observation, clock);
          const visible = current && observation?.state === "observed" && observation.models.includes(item.model_id);
          return <button key={item.id} type="button" role="row" aria-current={item.id === target?.id ? "true" : undefined} onClick={() => { setSelected(item.id); actions.inspect(item.id); }}><span role="cell"><strong>{item.provider_key}</strong><small>{item.model_id}</small></span><span role="cell">{visible ? "Exposed" : observation?.state === "observed" ? "Old observation" : "Not checked"}</span><span role="cell">{visible ? observation.capacity?.input_capacity_tokens?.toLocaleString() ?? "Unknown" : "Unknown"}</span><span role="cell">{workspace.compute.targets.some(bound => bound.id === item.id) ? "Bound" : "Not bound"}</span></button>;
        })}</div>}
        <p className="yvex-clarification">This is the serving catalog, not YVEX's complete source/package registry. Pull, stop, resume, prepare, compile, load and unload cannot be represented as working buttons until their native management contract reaches YAI Application and CLI.</p>
      </section>}
      {section === "Machines" && <MachineRegistry application={application} tenant={tenant} />}
      {section === "Lifecycle" && <section className="yvex-lifecycle"><header className="yvex-section-heading"><div><h2>Sources, artifacts &amp; compiler</h2><p>YVEX's native pipeline has separate identities and admission at every stage.</p></div></header><div className="yvex-contract-table" role="table" aria-label="Native lifecycle contract posture"><div role="row"><strong role="columnheader">Stage</strong><strong role="columnheader">YVEX owner</strong><strong role="columnheader">Studio connection</strong></div>{[["Model source", "HF/local pull, exact revision, verification", "No typed YAI management route"], ["Representations", "Immutable package and tensor lineage", "No typed YAI management route"], ["Compiler", "Preparation, specialization, artifact evidence", "No typed YAI management route"], ["Engine", "Load, unload, compiled context, residency", "No typed YAI management route"], ["Session", "Generation, cancellation, media result", "Case text execution only via governed YAI"]].map(([name, owner, connection]) => <div role="row" key={name}><span role="cell">{name}</span><span role="cell">{owner}</span><span role="cell">{connection}</span></div>)}</div><p className="yvex-clarification">The current public remote bootstrap permits bounded identity/status reads with an independently approved host key. The newly published YAI machine registry stores that pin; it does not yet consume the remote YVEX transport. Studio does not run YVEX CLI or infer a compiler state from the model name.</p></section>}
      {section === "Connection" && <div className="yvex-connection"><ProviderWorkspace {...props} scope="yvex" /></div>}
      {section === "Observability" && <section className="yvex-observability"><header className="yvex-section-heading"><div><h2>Observability</h2><p>Separate measured YAI provider evidence from native YVEX server telemetry.</p></div></header>{target ? <><div className="yvex-readout"><div><span>YAI health</span><strong>{health?.effective_posture ?? "Unknown"}</strong><small>{health?.observed_at_unix_ms ? `Report ${at(health.observed_at_unix_ms)}` : "No observation"}</small></div><div><span>Provider circuit</span><strong>{health?.circuit ?? "Unknown"}</strong><small>Owned by YAI</small></div><div><span>Text interface check</span><strong>{capability ? "Passed" : "Missing"}</strong><small>Protocol check, not Case answer quality</small></div><div><span>Native logs</span><strong>Unavailable</strong><small>No qualified log stream to this Studio</small></div></div>{qualification(target.id)}<div className="yvex-next">{controls(target.id)}<Button onClick={() => actions.openPerspective("Telemetry")}>YAI Host telemetry</Button></div></> : <EmptyState title="No deployment to observe" body="Connect a compatible target to see YAI's dated observations." />}</section>}
    </div>
  </div>;
}
