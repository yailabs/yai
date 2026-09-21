// One cancellable close path for chrome, menus and native window-manager close.
// Draft ownership stays in the Workbench; this host adapter knows no Case data.
export function requestWindowClose() {
  if (!window.dispatchEvent(new Event("yai:before-window-close", { cancelable: true }))) return;
  if (window.__TAURI__) void window.__TAURI__.core.invoke("desktop_close");
  else window.close();
}
