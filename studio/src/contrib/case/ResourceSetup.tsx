import { useState } from "react";
import type { ResourceImportInput } from "../../clients/application";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

type Family = ResourceImportInput["definition"]["address"]["kind"];
const families: Record<Family, string> = { filesystem: "Filesystem reads", discovery: "Directory discovery", sqlite: "SQLite query", http_service: "HTTP service", mcp: "MCP catalog", process_runner: "Bounded process runner" };
const operations: Record<Family, string[]> = { filesystem: ["filesystem_read", "filesystem_search"], discovery: ["discover"], sqlite: ["database_query"], http_service: ["http_fetch"], mcp: ["mcp_catalog"], process_runner: ["process_run"] };

export function AttachResourceAction({ workspace, platform }: Pick<SurfaceRendererProps, "workspace" | "platform">) {
  const application = platform.application;
  useApplicationAvailability(application);
  const [open, setOpen] = useState(false);
  const [kind, setKind] = useState<Family>("filesystem");
  const local = ["filesystem", "discovery", "sqlite", "process_runner"].includes(kind);
  const named = kind === "sqlite" || kind === "http_service" || kind === "process_runner";
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  return <><Button disabled={!application?.supports("resource.import")} onClick={() => setOpen(true)}>Attach Resource…</Button>
    {open && application && <ApplicationActionDialog title="Attach Resource" description="Configure a bounded capability on the YAI Host. YAI resolves physical identity and checks current ownership. Attachment neither executes the operation nor proves availability. YAI applies policy and applicable Review checks to each request." submitLabel="Attach Resource" close={() => setOpen(false)} committed={refresh} resync={refresh} submit={form => {
      const text = (key: string) => String(form.get(key) ?? "").trim();
      const endpoint = { endpoint: text("endpoint"), allowed_ip_addresses: text("ips").split(/\s*,\s*|\s+/).filter(Boolean), credential_ref: text("credential") || null };
      const address: ResourceImportInput["definition"]["address"] = kind === "sqlite"
        ? { kind, root: text("root"), path: text("path"), queries: { [text("name")]: text("query") } }
        : kind === "process_runner" ? { kind, root: text("root"), runners: { [text("name")]: { executable: text("executable"), executable_digest: text("digest"), argv: String(form.get("argv") ?? "") === "" ? [] : String(form.get("argv")).split(/\r?\n/), working_directory: text("directory"), environment: {}, timeout_ms: Number(form.get("timeout")) } } }
        : kind === "http_service" ? { kind, endpoint, paths: { [text("name")]: text("path") } }
        : kind === "mcp" ? { kind, endpoint } : { kind, root: text("root") };
      return application.importResource({ case_ref: workspace.case.case_ref, definition: {
        schema: "yai.resource_definition.v1", attachment_id: text("ref"), policy_owner: workspace.case.participant_ref,
        participant_ids: [workspace.case.participant_ref], operations: [...operations[kind], ...(kind === "discovery" && form.get("admission") ? ["admit_content", "content_read"] : [])],
        read_prefixes: local && kind !== "process_runner" ? [text(kind === "sqlite" ? "path" : "prefix")] : [], names: named ? [text("name")] : [],
        max_output_bytes: Number(form.get("bytes")), max_items: Number(form.get("items")), review_requirement: "require_review", address,
      } });
    }}>
      <label>Resource reference<input name="ref" required placeholder="resource:infrastructure-evidence" autoFocus /></label>
      <label>Family<select aria-label="Resource family" value={kind} onChange={event => setKind(event.target.value as Family)}>{Object.entries(families).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label>
      {local ? <label>Root directory on the YAI Host<input name="root" required placeholder="/absolute/path" /></label> : <><label>Endpoint<input name="endpoint" type="url" required placeholder="http://127.0.0.1:8080" /></label><label>Allowed IP addresses<input name="ips" required placeholder="127.0.0.1" /></label><label>Credential reference, optional<input name="credential" placeholder="Reference only; never a secret" /></label></>}
      {(kind === "filesystem" || kind === "discovery") && <label>Allowed relative path prefix<input name="prefix" required placeholder="evidence" /></label>}
      {kind === "discovery" && <><label className="checkbox-setting"><input name="admission" type="checkbox" />Allow admission and reading of retained Case content</label><small>Required to acquire Sources and open their retained material. This scope does not grant permission to execute a request.</small></>}
      {named && <label>Bound operation name<input name="name" required placeholder="service_status" /></label>}
      {kind === "sqlite" && <><label>Database path relative to root<input name="path" required placeholder="inventory.sqlite" /></label><label>Bound read query<textarea name="query" required rows={3} placeholder="SELECT name, status FROM services" /></label></>}
      {kind === "http_service" && <label>Bound relative HTTP path<input name="path" required placeholder="health" /></label>}
      {kind === "process_runner" && <><label>Absolute executable on the YAI Host<input name="executable" required placeholder="/usr/bin/tool" /></label><label>Expected executable digest<input name="digest" required pattern="sha256:[a-f0-9]{64}" placeholder="sha256:…" /></label><small>Use the digest of the approved binary. YAI verifies it before dispatch; Studio does not read or hash Host files.</small><label>Arguments, one exact argument per line<textarea name="argv" rows={3} placeholder="--check" /></label><small>Arguments are passed literally, without shell expansion, with an empty environment. Require Review through an explicit process.run policy rule in Authority.</small><label>Working directory relative to root<input name="directory" required placeholder="work" /></label><label>Timeout, milliseconds<input name="timeout" type="number" min={1} max={60000} defaultValue={5000} required /></label></>}
      <label>Maximum output bytes<input name="bytes" type="number" min={1} defaultValue={65536} required /></label>
      <label>Maximum items<input name="items" type="number" min={1} defaultValue={100} required /></label>
      <p>Scope: current Participant. After attachment, declare a Source or submit a governed Resource request. If confirmation is lost, refresh Environment before retrying the same reference and configuration.</p>
    </ApplicationActionDialog>}
  </>;
}
