import { CommandService } from "./commands";
import { ConfigurationService } from "./configuration";
import { ContextKeyService } from "./context";
import { KeybindingService } from "./keybindings";
import { DisposableStore } from "./lifecycle";
import { MenuService } from "./menus";
import { NavigationService } from "./navigation";
import type { HostServices } from "./host";
import { ThemeService } from "./theme";

export class PlatformServices extends DisposableStore {
  readonly context = this.add(new ContextKeyService());
  readonly commands = this.add(new CommandService(this.context));
  readonly menus = this.add(new MenuService(this.context));
  readonly keybindings = this.add(new KeybindingService(this.commands, this.context));
  readonly configuration = this.add(new ConfigurationService({
    "workbench.sidebar.width": 204,
    "workbench.auxiliary.width": 320,
    "workbench.panel.heightRatio": 0.36,
  }));
  readonly navigation = this.add(new NavigationService());
  readonly theme = this.add(new ThemeService());

  constructor(readonly host: HostServices) {
    super();
    this.context.update("studio.host.native", host.capabilities.nativeDesktop);
    this.context.update("terminal.available", host.capabilities.terminalAvailable);
    this.theme.apply();
  }
}
