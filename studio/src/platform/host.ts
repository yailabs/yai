import { toDisposable, type Disposable } from "./lifecycle";

export interface HostCapabilities {
  kind: "desktop-native" | "web";
  nativeDesktop: boolean;
  terminalAvailable: boolean;
  windowControlsAvailable: boolean;
}

export interface HostClientAttachment {
  client_id: string;
  client_kind: "studio" | "cli" | "qualification";
  pid: number;
  connected_at_unix_ms: number;
  last_seen_unix_ms: number;
}

export interface HostTelemetry {
  schema: string;
  state: string;
  pid?: number;
  process_identity?: string;
  instance_id?: string;
  started_at_unix_ms?: number;
  uptime_ms: number;
  protocol: string;
  version: string;
  build: string;
  yai_home: string;
  yai_home_identity: string;
  transport: string;
  endpoint?: string;
  endpoint_posture: string;
  application_readiness: string;
  connected_clients: number;
  client_kinds: Record<string, number>;
  clients: HostClientAttachment[];
  event_sequence: number;
  last_activity_unix_ms: number;
  runtime_supervision: "not_integrated" | string;
}

export interface HostConnectionState {
  state: "not-used" | "starting" | "live" | "reconnecting" | "unavailable" | "stopped";
  telemetry?: HostTelemetry;
  reason?: string;
  resync_required: boolean;
}

export interface HostServices extends Disposable {
  capabilities: HostCapabilities;
  snapshot(): HostConnectionState;
  subscribe(listener: (state: HostConnectionState) => void): Disposable;
  status(): Promise<HostTelemetry | undefined>;
  start(): Promise<HostTelemetry>;
  stop(): Promise<HostTelemetry>;
  restart(): Promise<HostTelemetry>;
  closeWindow(): void;
}

class DesktopHostServices implements HostServices {
  readonly capabilities: HostCapabilities;
  private state: HostConnectionState;
  private readonly listeners = new Set<(state: HostConnectionState) => void>();
  private unlisten?: () => void;

  constructor(useYaiHost: boolean) {
    const nativeDesktop = Boolean(window.__TAURI__);
    this.capabilities = {
      kind: nativeDesktop ? "desktop-native" : "web",
      nativeDesktop,
      terminalAvailable: nativeDesktop,
      windowControlsAvailable: nativeDesktop,
    };
    this.state = { state: nativeDesktop && useYaiHost ? "starting" : "not-used", resync_required: false };
    if (nativeDesktop && useYaiHost) {
      void window.__TAURI__!.event.listen<HostConnectionState>("yai://host-state", ({ payload }) => {
        this.publish(payload);
      }).then((unlisten) => { this.unlisten = unlisten; });
      void this.status();
    }
  }

  snapshot() { return this.state; }

  subscribe(listener: (state: HostConnectionState) => void) {
    this.listeners.add(listener);
    listener(this.state);
    return toDisposable(() => this.listeners.delete(listener));
  }

  async status() {
    if (!window.__TAURI__) return undefined;
    try {
      const telemetry = await window.__TAURI__.core.invoke<HostTelemetry>("studio_host_status");
      this.publish({ state: telemetry.state === "running" ? "live" : "stopped", telemetry, resync_required: false });
      return telemetry;
    } catch (error) {
      this.publish({ state: "unavailable", reason: String(error), resync_required: false });
      return undefined;
    }
  }

  start() { return this.lifecycle("studio_host_start", "starting"); }
  stop() { return this.lifecycle("studio_host_stop", "stopped"); }
  restart() { return this.lifecycle("studio_host_restart", "reconnecting"); }

  closeWindow() {
    if (window.__TAURI__) void window.__TAURI__.core.invoke("desktop_close");
    else window.close();
  }

  dispose() {
    this.unlisten?.();
    this.unlisten = undefined;
    this.listeners.clear();
  }

  private async lifecycle(command: string, pending: HostConnectionState["state"]) {
    if (!window.__TAURI__) throw new Error("local_host_transport_unavailable");
    this.publish({ state: pending, telemetry: this.state.telemetry, resync_required: command !== "studio_host_stop" });
    const telemetry = await window.__TAURI__.core.invoke<HostTelemetry>(command);
    this.publish({ state: telemetry.state === "running" ? "live" : "stopped", telemetry, resync_required: command === "studio_host_restart" });
    return telemetry;
  }

  private publish(state: HostConnectionState) {
    this.state = state;
    for (const listener of this.listeners) listener(state);
  }
}

export function createHostServices(useYaiHost = true): HostServices {
  return new DesktopHostServices(useYaiHost);
}
