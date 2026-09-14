import { scenarios } from "../../../tests/fixtures/studio/scenarios";
import {
  compositionSections,
  informationByScenario,
  startCenter,
} from "../../../tests/fixtures/studio/information";
import type { ScenarioId, StudioClient } from "./presentation";

export class FixtureClient implements StudioClient {
  readonly mode = "fixture" as const;
  catalog() {
    return startCenter;
  }
  composition() {
    return compositionSections;
  }
  scenarios() {
    return scenarios.map(({ fixture }) => ({
      id: fixture.id,
      label: fixture.label,
    }));
  }
  workspace(id: ScenarioId) {
    const workspace = scenarios.find((item) => item.fixture.id === id);
    if (!workspace) throw new Error(`Unknown fixture: ${id}`);
    return { ...workspace, information: informationByScenario[id] };
  }
}
