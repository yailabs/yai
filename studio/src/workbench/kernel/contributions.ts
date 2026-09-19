import type { Disposable } from "../../platform/lifecycle";
import type { PlatformServices } from "../../platform/services";
import type { WorkbenchRegistry } from "./registry";

export interface ContributionContext {
  platform: PlatformServices;
  workbench: WorkbenchRegistry;
}

export interface StudioContribution {
  id: string;
  register(context: ContributionContext): void | Disposable | readonly Disposable[];
}

export function registerContributions(
  contributions: readonly StudioContribution[],
  context: ContributionContext,
) {
  const disposables: Disposable[] = [];
  for (const contribution of contributions) {
    const registered = contribution.register(context);
    if (Array.isArray(registered)) disposables.push(...registered);
    else if (registered) disposables.push(registered as Disposable);
  }
  return { dispose: () => disposables.reverse().forEach((value) => value.dispose()) };
}
