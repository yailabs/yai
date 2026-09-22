import { PolicyActions } from "./PolicyActions";
import type { PlatformServices } from "../../platform/services";
import type { CasePresentation } from "../../clients/dataSource";
import { CollectionList } from "../../components/CollectionList";
import { Badge, EmptyState, PanelHeader } from "../../components/primitives";
import { lazy } from "react";
import type { SurfaceInput } from "../../workbench/surface/model";
import { sourceInput } from "../surfaces/inputs";
import { isPolicySource } from "./environment";

const PolicyIntake = lazy(() => import("./PolicyIntake").then(module => ({ default: module.PolicyIntake })));

export function AuthoritySurface({ workspace, inspect, selected, platform, openSurface }: { platform: PlatformServices; workspace: CasePresentation; selected?: string; inspect: (id: string) => void; openSurface(input: SurfaceInput): void }) {
  const authority = workspace.authority;
  const decision = authority.last_decision;
  return <div className="live-page authority-page"><header className="surface-title"><small>{workspace.case.display_name}</small><h1>Authority</h1><p>Bound policy, reviews, grants and the latest recorded decision.</p></header>
    <PolicyActions application={platform.application} workspace={workspace} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} />
    <PolicyIntake key={workspace.case.case_ref} application={platform.application} workspace={workspace} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} />
    <section className="authority-section"><PanelHeader title="Policy Sources" detail="Declared role · not automatic authority" /><CollectionList label="Policy Sources" selected={selected} items={workspace.environment.sources.filter(isPolicySource).map(source => ({ id: source.id, title: source.label, detail: source.roles.join(" · "), meta: source.posture ?? "declared", icon: "authority" }))} onSelect={id => openSurface(sourceInput(workspace, id))} empty="No policy-role Sources projected" /></section>
    {decision && <section className="authority-decision"><PanelHeader title="Latest decision" detail={typeof decision.recorded_at_generation === "number" ? `Generation ${decision.recorded_at_generation}` : undefined} actions={<Badge tone={decision.outcome === "allow" ? "success" : decision.outcome === "deny" ? "error" : "warning"}>{String(decision.outcome ?? "Recorded")}</Badge>} /><p>This recorded decision describes its original operation. It does not authorize a new action.</p><details><summary>Decision details</summary><dl className="object-facts">{Object.entries(decision).map(([key, value]) => <div key={key}><dt>{key.replaceAll("_", " ")}</dt><dd>{typeof value === "object" ? JSON.stringify(value) : String(value)}</dd></div>)}</dl></details></section>}
    <section className="authority-section"><PanelHeader title="Bound policies" detail={`${authority.policies.length} bindings`} /><CollectionList label="Policies" selected={selected} items={authority.policies.map(policy => ({ id: policy.id, title: policy.policy_key.replaceAll("_", " "), detail: policy.reason, meta: `Version ${policy.version}`, icon: "authority" }))} onSelect={inspect} empty="No policy is bound" /></section>
    <section className="authority-section"><PanelHeader title="Reviews" detail={`${authority.reviews.length} projected reviews`} /><CollectionList label="Reviews" selected={selected} items={authority.reviews.map(review => ({ id: review.id, title: review.operation_ref, detail: review.id, meta: review.status, icon: "review" }))} onSelect={inspect} empty="No reviews projected" /></section>
    <section className="authority-section"><PanelHeader title="Grants" detail={`${authority.grants.length} projected grants`} />{authority.grants.length ? authority.grants.map((grant, index) => <details className="authority-grant" key={index}><summary>{String(grant.id ?? grant.grant_ref ?? `Grant ${index + 1}`)}</summary><dl className="object-facts">{Object.entries(grant).map(([key, value]) => <div key={key}><dt>{key.replaceAll("_", " ")}</dt><dd>{typeof value === "object" ? JSON.stringify(value) : String(value)}</dd></div>)}</dl></details>) : <EmptyState title="No grants projected" body="Access continues to be evaluated by YAI for each operation." />}</section>
  </div>;
}
