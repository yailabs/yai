export interface HostCapabilities {
  kind: "desktop-native" | "web";
  nativeDesktop: boolean;
  terminalAvailable: boolean;
  windowControlsAvailable: boolean;
}

export interface HostServices {
  capabilities: HostCapabilities;
  closeWindow(): void;
}

export function createHostServices(): HostServices {
  const nativeDesktop = Boolean(window.__TAURI__);
  return {
    capabilities: {
      kind: nativeDesktop ? "desktop-native" : "web",
      nativeDesktop,
      terminalAvailable: nativeDesktop,
      windowControlsAvailable: nativeDesktop,
    },
    closeWindow() {
      if (window.__TAURI__) void window.__TAURI__.core.invoke("desktop_close");
      else window.close();
    },
  };
}
