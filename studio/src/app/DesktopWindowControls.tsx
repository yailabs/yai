import { Icon } from "../components/Icon";

function invoke(command: string) {
  return window.__TAURI__?.core.invoke(command);
}

export function DesktopWindowControls() {
  if (!window.__TAURI__) return null;
  return (
    <div className="desktop-window-controls" aria-label="Window controls">
      <button aria-label="Minimize window" onClick={() => void invoke("desktop_minimize")}>
        <Icon name="minimize" size={14} />
      </button>
      <button aria-label="Maximize or restore window" onClick={() => void invoke("desktop_toggle_maximize")}>
        <Icon name="maximize" size={14} />
      </button>
      <button className="close" aria-label="Close window" onClick={() => void invoke("desktop_close")}>
        <Icon name="close" size={14} />
      </button>
    </div>
  );
}
