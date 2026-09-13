import { scenarios } from "../../../tests/fixtures/studio/scenarios";
import type { ScenarioId, StudioClient } from "./presentation";

export class FixtureClient implements StudioClient {
  readonly mode = "fixture" as const;
  scenarios() {
    return scenarios.map(({ fixture }) => ({
      id: fixture.id,
      label: fixture.label,
    }));
  }
  workspace(id: ScenarioId) {
    const workspace = scenarios.find((item) => item.fixture.id === id);
    if (!workspace) throw new Error(`Unknown fixture: ${id}`);
    return workspace;
  }
}
