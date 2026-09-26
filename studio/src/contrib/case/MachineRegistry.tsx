import { useEffect, useState } from "react";
import type { ApplicationAccess } from "../../clients/application";
import type { MachineAssetView } from "../../clients/machines";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Badge, Button, EmptyState } from "../../components/primitives";

type Action = "register" | "revoke";

/** Exact Tenant pins only. Network discovery and YVEX runtime control need their own owner. */
export function MachineRegistry({ application, tenant }: { application?: ApplicationAccess; tenant?: string }) {
  const [revision, setRevision] = useState(0);
  const [rows, setRows] = useState<MachineAssetView[]>([]);
  const [selected, setSelected] = useState<string>();
  const [detail, setDetail] = useState<MachineAssetView>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [action, setAction] = useState<Action>();
  const current = detail?.registration.asset_id === selected ? detail : rows.find(row => row.registration.asset_id === selected);
  useEffect(() => {
    if (!application || !tenant) { setLoading(false); return; }
    let alive = true;
    setLoading(true);
    void application.listMachines(tenant).then(result => {
      if (!alive) return;
      if (result.result_state === "success" && Array.isArray(result.data)) {
        setRows(result.data); setError(undefined);
        setSelected(old => old && result.data?.some(row => row.registration.asset_id === old) ? old : result.data?.[0]?.registration.asset_id);
      } else { setRows([]); setError(result.error?.safe_message ?? "Machine inventory unavailable."); }
    }).catch(() => { if (alive) { setRows([]); setError("Machine inventory connection unavailable."); } })
      .finally(() => { if (alive) setLoading(false); });
    return () => { alive = false; };
  }, [application, tenant, revision]);
  useEffect(() => {
    if (!application || !tenant || !selected || !application.supports("machine.get")) { setDetail(undefined); return; }
    let alive = true;
    void application.getMachine(tenant, selected).then(result => {
      if (alive) setDetail(result.result_state === "success" && result.data?.registration.asset_id === selected ? result.data : undefined);
    });
    return () => { alive = false; };
  }, [application, tenant, selected, revision]);
  const refresh = () => setRevision(value => value + 1);
  const supported = Boolean(application?.supports("machine.list"));
  return <section className="yvex-machines" aria-label="Tenant machines">
    <header className="yvex-section-heading"><div><h2>Machines</h2><p>Approved identities available to this Tenant. Registration does not contact the machine.</p></div><div><Button onClick={refresh} disabled={!supported || loading}>Refresh</Button><Button onClick={() => setAction("register")} disabled={!tenant || !application?.supports("machine.register")}>Register machine</Button></div></header>
    {error && <p className="yvex-inline-alert" role="alert">{error}</p>}
    {!supported && <p className="yvex-inline-alert">{application?.reason("machine.list") ?? "A native YAI Host is required."}</p>}
    {supported && !loading && !rows.length && <EmptyState title="No registered machines" body="The Tenant has no approved host-key pins. A provider endpoint does not identify its machine." />}
    {!!rows.length && <div className="yvex-machine-layout"><div className="yvex-machine-list" role="list" aria-label="Registered machine identities">
      {rows.map(row => <button key={row.registration.asset_id} role="listitem" aria-current={row.registration.asset_id === selected ? "true" : undefined} onClick={() => setSelected(row.registration.asset_id)}>
        <span><strong>{row.registration.address}</strong><small>{row.registration.management_user} · SSH {row.registration.port}</small></span><Badge tone={row.revocation ? "error" : "success"}>{row.revocation ? "Revoked" : "Pinned"}</Badge>
      </button>)}
    </div>{current && <section className="yvex-machine-detail" aria-label="Selected machine identity"><header><div><small>Tenant identity pin</small><h3>{current.registration.address}</h3></div><Badge tone={current.revocation ? "error" : "success"}>{current.revocation ? "Revoked" : "Pinned"}</Badge></header>
      <dl><div><dt>Management account</dt><dd>{current.registration.management_user}</dd></div><div><dt>SSH port</dt><dd>{current.registration.port}</dd></div><div><dt>Approved</dt><dd>{new Date(current.registration.approved_at_unix_ms).toLocaleString()}</dd></div><div><dt>Runtime state</dt><dd>Not observed</dd></div></dl>
      <p>YAI stores the independently approved host key. This does not prove reachability, YVEX installation, model residency or permission to start a remote process.</p>
      {!current.revocation && <Button onClick={() => setAction("revoke")} disabled={!application?.supports("machine.revoke")}>Revoke pin</Button>}
      <details><summary>Identity and approval evidence</summary><dl><div><dt>Asset</dt><dd>{current.registration.asset_id}</dd></div><div><dt>Device</dt><dd>{current.registration.device_identity}</dd></div><div><dt>Approval reference</dt><dd>{current.registration.approval_ref}</dd></div><div><dt>Approved by</dt><dd>{current.registration.approved_by_principal_id}</dd></div><div><dt>Public host key</dt><dd>{current.registration.host_public_key}</dd></div>{current.revocation && <div><dt>Revocation</dt><dd>{current.revocation.reason}</dd></div>}</dl></details>
    </section>}</div>}
    {action && application && tenant && <ApplicationActionDialog title={action === "register" ? "Register machine identity" : "Revoke machine identity"}
      description={action === "register" ? "Pin an SSH Ed25519 host key that you verified independently. YAI stores the identity; it will not scan the LAN, connect or start YVEX." : "This permanently revokes the selected Tenant pin. It does not stop a remote machine or YVEX process."}
      submitLabel={action === "register" ? "Register identity" : "Revoke pin"} close={() => setAction(undefined)} committed={refresh}
      submit={form => action === "register" ? application.registerMachine({ tenant_id: tenant, address: String(form.get("address")).trim(), port: Number(form.get("port")), management_user: String(form.get("user")).trim(), host_public_key: String(form.get("key")).trim(), approval_ref: String(form.get("approval")).trim() }) : application.revokeMachine({ tenant_id: tenant, asset_id: selected!, reason: String(form.get("reason")).trim() })}>
      {action === "register" ? <><label>Machine address<input name="address" required autoFocus placeholder="spark.internal" /></label><label>SSH port<input name="port" type="number" min="1" max="65535" defaultValue="22" required /></label><label>Management user<input name="user" required autoComplete="off" /></label><label>Verified OpenSSH Ed25519 host public key<input name="key" required placeholder="ssh-ed25519 AAAA…" autoComplete="off" /></label><label>Independent approval reference<input name="approval" required placeholder="Change or verification reference" /></label></> : <><p>Machine: {current?.registration.address ?? selected}</p><label>Reason<input name="reason" required autoFocus /></label></>}
    </ApplicationActionDialog>}
  </section>;
}
