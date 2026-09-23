import { ApplicationAccess } from "../clients/application";
import type { LiveClient } from "../clients/live";
import { EditingService } from "./editing";
import { CommandService } from "./commands";
import { ConfigurationService } from "./configuration";
import { ContextKeyService } from "./context";
import { KeybindingService } from "./keybindings";
import { DisposableStore } from "./lifecycle";
import { MenuService } from "./menus";
import type { HostServices } from "./host";
import { ThemeService } from "./theme";

export class PlatformServices extends DisposableStore {
  readonly context = this.add(new ContextKeyService());
  readonly editing = this.add(new EditingService(this.context));
  readonly commands = this.add(new CommandService(this.context));
  readonly menus = this.add(new MenuService(this.context));
  readonly keybindings = this.add(new KeybindingService(this.commands, this.context));
  readonly configuration = this.add(new ConfigurationService({
    "workbench.sidebar.width": 204,
    "workbench.auxiliary.width": 320,
    "workbench.panel.heightRatio": 0.36,
    "workbench.openPreview": true,
    "workbench.rail.hidden": [],
    "workbench.rail.order": [],
    "appearance.reducedMotion": false,
    "terminal.scrollback": 5000,
  }));
  readonly theme = this.add(new ThemeService());

  readonly application?: ApplicationAccess;

  constructor(readonly host: HostServices, applicationClient?: LiveClient) {
    super();
    this.add(host);
    if (applicationClient) this.application = this.add(new ApplicationAccess(applicationClient, host));
    this.context.update("studio.host.native", host.capabilities.nativeDesktop);
    this.context.update("terminal.available", host.capabilities.terminalAvailable);
    this.theme.apply();
  }
}
