import { useEffect, useRef, useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { ProviderModelsInput } from "../../clients/compute";
import { Button } from "../../components/primitives";

/** Discovery is presentation metadata. Only provider.register retains a target. */
export function ProviderRegistrationFields({ application, tenant, yvex }: {
  application: ApplicationAccess; tenant: string; yvex: boolean;
}) {
  const [endpoint, setEndpoint] = useState("");
  const [locality, setLocality] = useState<ProviderModelsInput["locality"]>("loopback");
  const [credential, setCredential] = useState("none");
  const [models, setModels] = useState<string[]>([]);
  const [model, setModel] = useState("");
  const [manual, setManual] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string>();
  const epoch = useRef(0);
  useEffect(() => () => { epoch.current++; }, []);
  const invalidate = () => {
    epoch.current++; setPending(false); setModels([]); setModel(""); setError(undefined);
  };
  const discover = async () => {
    const request = ++epoch.current;
    setPending(true); setModels([]); setModel(""); setError(undefined);
    try {
      const result = await application.discoverProviderModels({ tenant_id: tenant, endpoint: endpoint.trim(), locality, credential_ref: credential.trim() });
      if (epoch.current !== request) return;
      if (result.result_state === "success" && result.data) {
        setModels(result.data.models); setManual(false);
        if (result.data.models.length === 1) setModel(result.data.models[0]);
      } else setError(result.error?.safe_message ?? "Model discovery unavailable.");
    } catch { if (epoch.current === request) setError("Model discovery unavailable. Check the connection and try again."); }
    finally { if (epoch.current === request) setPending(false); }
  };
  return <>
    <label>Provider / runtime label<input autoFocus required name="key" placeholder={yvex ? "YVEX deployment" : "Local inference"} /></label>
    <label>Endpoint<input required type="url" name="endpoint" placeholder="http://127.0.0.1:8001" value={endpoint} onChange={event => { invalidate(); setEndpoint(event.target.value); }} /></label>
    <label>Locality<select name="locality" value={locality} onChange={event => { invalidate(); setLocality(event.target.value as ProviderModelsInput["locality"]); }}><option value="loopback">Loopback</option><option value="private_network">Private network</option><option value="remote">Remote</option></select></label>
    <Button type="button" disabled={pending || !endpoint.trim() || !application.supports("provider.models")} onClick={discover}>{pending ? "Discovering…" : "Discover exposed models"}</Button>
    {error && <p role="alert">{error}</p>}
    {models.length > 0 && <p role="status">{models.length} exposed model(s). Discovery does not qualify inference.</p>}
    {manual ? <label>Exact model identity<input required name="model" value={model} onChange={event => setModel(event.target.value)} placeholder="Exact serving ID supplied by the operator" /></label>
      : <label>Exposed model<select required name="model" value={model} onChange={event => setModel(event.target.value)}><option value="">{models.length ? "Choose a model" : "Discover models first"}</option>{models.map(id => <option key={id} value={id}>{id}</option>)}</select></label>}
    <label className="checkbox-setting"><input type="checkbox" checked={manual} onChange={event => { invalidate(); setManual(event.target.checked); }} />Enter an exact model ID manually</label>
    <details><summary>Advanced connection options</summary>
      <label>Credential reference<input required name="credential" value={credential} onChange={event => { invalidate(); setCredential(event.target.value); }} /></label>
      <label>Compatibility extension<select name="extension" defaultValue={yvex ? "yvex.http.v1" : ""}><option value="">None</option><option value="yvex.http.v1">yvex.http.v1</option></select></label>
    </details>
  </>;
}
