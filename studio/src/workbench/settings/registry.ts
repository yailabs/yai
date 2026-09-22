import { toDisposable, type Disposable } from "../../platform/lifecycle";

export type SettingSection = "General" | "Appearance" | "Workbench" | "Editor" | "Terminal" | "Identity" | "YAI Host" | "Security" | "Advanced";
export type SettingScope = "local" | "case" | "host";

export interface SettingDefinition {
  id: string;
  title: string;
  description: string;
  section: SettingSection;
  scope: SettingScope;
  control: "boolean" | "number" | "select" | "information";
  defaultValue?: boolean | number | string;
  options?: readonly { value: string; label: string }[];
  available: boolean;
  unavailableReason?: string;
  order?: number;
}

export class SettingsRegistry implements Disposable {
  private readonly definitions = new Map<string, SettingDefinition>();

  register(definition: SettingDefinition) {
    if (this.definitions.has(definition.id)) throw new Error(`Duplicate setting: ${definition.id}`);
    this.definitions.set(definition.id, definition);
    return toDisposable(() => this.definitions.delete(definition.id));
  }

  entries() {
    return [...this.definitions.values()].sort((a, b) =>
      a.section.localeCompare(b.section) || (a.order ?? 0) - (b.order ?? 0) || a.title.localeCompare(b.title));
  }

  dispose() { this.definitions.clear(); }
}
