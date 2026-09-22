import { useState } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import type { HumanCheckpoint, WorkflowDefinition } from "../../clients/work";
import { ApplicationActionDialog } from "../../components/ApplicationActionDialog";
import { Button } from "../../components/primitives";
import { useApplicationAvailability } from "./applicationActions";

type Action = "define" | "bind" | "propose" | "adopt";
const titles = { define: "Define checkpoint Workflow", bind: "Bind Workflow", propose: "Propose Workflow checkpoint", adopt: "Adopt Workflow patch" };
const operations = { define: "workflow.define", bind: "workflow.bind", propose: "workflow.patch.propose", adopt: "workflow.patch.adopt" };
const checkpoint = (node_id: string, prompt: string, required_roles: string[]): HumanCheckpoint => ({ node_id, kind: "human_input", actor_slot: "operator", prompt, required_roles, input_kind: "text", max_bytes: 4096 });

export function WorkflowActions({ workspace, platform }: Pick<SurfaceRendererProps, "workspace" | "platform">) {
  const application = platform.application; useApplicationAvailability(application);
  const [pending, setPending] = useState<{ action: Action; topology?: string }>();
  const [definition, setDefinition] = useState<WorkflowDefinition>();
  const [patchRef, setPatchRef] = useState("");
  const [receipt, setReceipt] = useState<string>();
  const refresh = () => platform.commands.executeCommand("studio.case.refresh").then(() => undefined);
  const topology = workspace.work.resolution?.effective_topology_digest;
  return <section className="workflow-actions"><div className="object-action-row">{(["define", "bind", "propose", "adopt"] as const).map(action => <Button key={action} disabled={!application?.supports(operations[action]) || action === "propose" && !topology} onClick={() => setPending({ action, topology })}>{titles[action]}</Button>)}</div>
    {definition && <p className="operation-receipt">Definition retained: {definition.name} · {definition.nodes.length} checkpoints · <code>{definition.workflow_definition_id}</code>. Binding is explicit.</p>}
    {receipt && <p className="operation-receipt" role="status">{receipt}</p>}
    {pending && application && <ApplicationActionDialog title={titles[pending.action]} description={pending.action === "define" ? "Author a bounded sequence of human checkpoints. YAI validates and retains the definition; it is not bound automatically. This form does not create model or effect nodes." : pending.action === "bind" ? "Bind an existing definition with the operator slot mapped to your current Participant. YAI checks the definition and current Case binding. Other executor/resource/case slots require a richer binding form." : pending.action === "propose" ? "Propose an additional human checkpoint against the exact current topology. Proposal is candidate plan material; only explicit adoption changes effective workflow topology." : "Adopt an exact retained patch. YAI checks its base topology, authority and current completed work. A stale patch is refused."}
      submitLabel={pending.action === "define" ? "Retain definition" : pending.action === "bind" ? "Bind definition" : pending.action === "propose" ? "Propose checkpoint" : "Adopt patch"} close={() => setPending(undefined)} enabled={application.supports(operations[pending.action]) && (pending.action !== "propose" || pending.topology === topology)}
      submit={async form => {
        if (pending.action === "define") {
          const prompts = String(form.get("steps")).split(/\r?\n/).map(value => value.trim()).filter(Boolean);
          const required = String(form.get("roles")).split(",").map(value => value.trim()).filter(Boolean);
          const result = await application.defineWorkflow({ schema: "yai.workflow_definition.v1", tenant_id: workspace.case.tenant_ref!, workflow_key: String(form.get("key")).trim(), declared_version: String(form.get("version")).trim(), name: String(form.get("name")).trim(), description: String(form.get("description")).trim(), nodes: prompts.map((prompt, index) => checkpoint(`step-${index + 1}`, prompt, required)), edges: prompts.slice(1).map((_, index) => ({ from: `step-${index + 1}`, to: `step-${index + 2}`, kind: "always" })) });
          if (result.result_state === "success" && result.data) setDefinition(result.data); return result;
        }
        if (pending.action === "bind") return application.bindWorkflow({ case_ref: workspace.case.case_ref, definition_ref: String(form.get("definition")).trim(), executor_bindings: [{ slot: "operator", participant_id: workspace.case.participant_ref }], resource_bindings: [], case_bindings: [] });
        if (pending.action === "propose") {
          const id = String(form.get("node")).trim(); const after = String(form.get("after"));
          const result = await application.proposeWorkflowPatch({ case_ref: workspace.case.case_ref, patch: { schema: "yai.workflow_plan_patch.v1", base_effective_topology_digest: pending.topology!, operations: [{ operation: "add_node", node: checkpoint(id, String(form.get("prompt")).trim(), []) }, ...(after ? [{ operation: "add_edge" as const, edge: { from: after, to: id, kind: "always" as const } }] : [])] } });
          const patch = result.data?.transition.payload.data.patch;
          if (result.result_state === "success" && patch) { setPatchRef(patch.patch_id); setReceipt(`Patch proposed: ${patch.patch_id}. Not adopted.`); } return result;
        }
        const result = await application.adoptWorkflowPatch({ case_ref: workspace.case.case_ref, patch_ref: String(form.get("patch")).trim() });
        if (result.result_state === "success") setReceipt(`Patch adopted at generation ${result.data?.state.generation}.`); return result;
      }} committed={refresh} resync={refresh}>
      {pending.action === "define" && <><label>Name<input name="name" required autoFocus /></label><label>Workflow key<input name="key" required placeholder="operator-checkpoints" /></label><label>Version<input name="version" required defaultValue="1" /></label><label>Description<textarea name="description" rows={2} /></label><label>Checkpoints, one prompt per line<textarea name="steps" rows={5} required /></label><label>Required Participant roles, comma separated<input name="roles" placeholder="Optional exact roles" /></label><small>Each checkpoint accepts up to 4 KiB of text. Node IDs are step-1, step-2, … and edges follow the entered order.</small></>}
      {pending.action === "bind" && <label>Definition reference<input name="definition" required autoFocus defaultValue={definition?.workflow_definition_id} /></label>}
      {pending.action === "propose" && <><label>New node ID<input name="node" required autoFocus /></label><label>Prompt<textarea name="prompt" required rows={3} /></label><label>After node<select name="after"><option value="">Independent checkpoint</option>{workspace.work.nodes.map(node => <option key={node.node_id}>{node.node_id}</option>)}</select></label><small>Base topology: {pending.topology}</small>{pending.topology !== topology && <p role="alert">The topology changed. Reopen this action.</p>}</>}
      {pending.action === "adopt" && <label>Patch reference<input name="patch" required autoFocus defaultValue={patchRef} /></label>}
    </ApplicationActionDialog>}
  </section>;
}
