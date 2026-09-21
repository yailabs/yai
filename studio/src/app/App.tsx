import { useEffect, useState } from "react";
import { FixtureClient } from "../clients/fixture";
import { LiveClient } from "../clients/live";
import { FixtureDataSource, LiveDataSource, type CaseDataSource } from "../clients/dataSource";
import { createHostServices } from "../platform/host";
import { PlatformServices } from "../platform/services";
import { WorkbenchRegistry } from "../workbench/kernel/registry";
import { registerContributions } from "../workbench/kernel/contributions";
import { builtInContributions } from "../contrib/builtins";
import { StudioApplication } from "./StudioApplication";
import { ComponentGallery } from "../live/ComponentGallery";
import { DesktopWindowFrame } from "./DesktopWindowControls";
import "../styles/workbench.css";
import "../styles/foundation.css";

interface Composition {
  dataSource: CaseDataSource;
  platform: PlatformServices;
  registry: WorkbenchRegistry;
  dispose(): void;
}

function createComposition(): Composition {
  const fixture = import.meta.env.VITE_STUDIO_MODE === "fixture";
  const host = createHostServices(!fixture);
  const platform = new PlatformServices(host);
  const registry = new WorkbenchRegistry();
  const contributions = registerContributions(builtInContributions, { platform, workbench: registry });
  const dataSource: CaseDataSource = fixture
    ? new FixtureDataSource(new FixtureClient())
    : new LiveDataSource(new LiveClient());
  return { dataSource, platform, registry, dispose() { contributions.dispose(); registry.dispose(); platform.dispose(); } };
}

export function App() {
  const [composition, setComposition] = useState<Composition>();
  useEffect(() => {
    const created = createComposition();
    setComposition(created);
    return () => created.dispose();
  }, []);
  if (import.meta.env.DEV && new URLSearchParams(window.location.search).get("gallery") === "1") return <ComponentGallery />;
  if (!composition) return null;
  return <><StudioApplication dataSource={composition.dataSource} platform={composition.platform} registry={composition.registry} /><DesktopWindowFrame /></>;
}
