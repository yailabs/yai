import type { WorkbenchRenderContext } from "../../workbench/kernel/types";
import { ApplicationMenuBar } from "../../workbench/kernel/ApplicationMenuBar";
import { Icon } from "../../components/Icon";

export function IdentityAccess({ actions }: WorkbenchRenderContext) {
  return <button aria-label="Identity / Account" title="Local identity and Case Participant" onClick={() => actions.openSettings("identity")}><Icon name="people" size={20} /></button>;
}

export function ManageAccess({ platform }: WorkbenchRenderContext) {
  return <ApplicationMenuBar platform={platform} locations={["Manage"]} compact />;
}
