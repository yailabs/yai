import type { Disposable } from "./lifecycle";

export type StudioTheme = "yai-dark";

export class ThemeService implements Disposable {
  private current: StudioTheme = "yai-dark";

  apply(theme: StudioTheme = this.current) {
    this.current = theme;
    document.documentElement.dataset.studioTheme = theme;
  }

  value() { return this.current; }

  dispose() {
    delete document.documentElement.dataset.studioTheme;
  }
}
