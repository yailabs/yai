const test = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const generated = path.resolve(__dirname, "../../build/studio-kernel-test");
const studio = path.join(generated, "studio/src");
const { ContextKeyService, when } = require(path.join(studio, "platform/context.js"));
const { CommandService } = require(path.join(studio, "platform/commands.js"));
const { MenuService } = require(path.join(studio, "platform/menus.js"));
const { KeybindingService } = require(path.join(studio, "platform/keybindings.js"));
const { NavigationService } = require(path.join(studio, "platform/navigation.js"));
const { SurfaceGroupService } = require(path.join(studio, "workbench/surface/model.js"));
const { WorkbenchRegistry } = require(path.join(studio, "workbench/kernel/registry.js"));
const { resolveMaterialSurfaceType } = require(path.join(studio, "contrib/surfaces/inputs.js"));

test("commands register, gate, execute and dispose without global state", async () => {
  const context = new ContextKeyService();
  const commands = new CommandService(context);
  let executions = 0;
  const registration = commands.registerCommand({ id: "studio.test", title: "Test", when: when.truthy("case.attached"), handler: () => executions++ });
  assert.equal(await commands.executeCommand("studio.test"), false);
  context.update("case.attached", true);
  assert.equal(await commands.executeCommand("studio.test"), true);
  assert.equal(executions, 1);
  assert.deepEqual(commands.entries(), [{ id: "studio.test", title: "Test", enabled: true }]);
  assert.throws(() => commands.registerCommand({ id: "studio.test", title: "Duplicate", handler() {} }), /already registered/);
  registration.dispose();
  assert.equal(commands.has("studio.test"), false);
});

test("context keys restore their previous value when scoped registration disposes", () => {
  const context = new ContextKeyService();
  context.update("view.active", "Overview");
  const scope = context.set("view.active", "Memory");
  assert.equal(context.get("view.active"), "Memory");
  scope.dispose();
  assert.equal(context.get("view.active"), "Overview");
});

test("menu contributions are ordered, hidden by context and disposable", () => {
  const context = new ContextKeyService();
  const menus = new MenuService(context);
  const later = menus.registerMenuItem({ id: "later", location: "View", group: "2", order: 0, command: "later" });
  menus.registerMenuItem({ id: "first", location: "View", group: "1", order: 0, command: "first", when: when.truthy("case.attached") });
  assert.deepEqual(menus.getMenu("View").map((item) => item.id), ["later"]);
  context.update("case.attached", true);
  assert.deepEqual(menus.getMenu("View").map((item) => item.id), ["first", "later"]);
  later.dispose();
  assert.deepEqual(menus.getMenu("View").map((item) => item.id), ["first"]);
});

test("keybindings dispatch commands and preserve terminal-owned chords", async () => {
  const context = new ContextKeyService();
  const commands = new CommandService(context);
  const keybindings = new KeybindingService(commands, context);
  let count = 0;
  commands.registerCommand({ id: "studio.test", title: "Test", handler: () => count++ });
  keybindings.registerKeybinding({ id: "test", command: "studio.test", key: "Mod+k" });
  let listener;
  const target = { addEventListener: (_name, value) => { listener = value; }, removeEventListener() {} };
  keybindings.attach(target);
  const event = (terminal) => ({ ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, key: "k", target: { closest: () => terminal ? {} : null }, preventDefault() {} });
  listener(event(false)); await new Promise((resolve) => setImmediate(resolve));
  assert.equal(count, 1);
  listener(event(true)); await new Promise((resolve) => setImmediate(resolve));
  assert.equal(count, 1);
});

test("surface group applies the same preview, pin and close behavior to heterogeneous inputs", () => {
  const surfaces = new SurfaceGroupService();
  surfaces.open({ id: "perspective:Overview", identity: "perspective:Overview", surfaceType: "case.perspective", title: "Overview", icon: "overview", viewId: "Overview", pinned: true });
  surfaces.open({ id: "surface:preview", identity: "material:notes", surfaceType: "material.markdown", title: "Notes", icon: "file", objectRef: "notes", pinned: false });
  surfaces.open({ id: "surface:preview", identity: "material:diagram", surfaceType: "material.image", title: "Diagram", icon: "file", objectRef: "diagram", pinned: false });
  assert.deepEqual(surfaces.snapshot().inputs.map((input) => input.title), ["Overview", "Diagram"]);
  surfaces.pin("surface:preview");
  surfaces.open({ id: "surface:preview", identity: "material:matrix", surfaceType: "data.table", title: "Matrix", icon: "file", objectRef: "matrix", pinned: false });
  surfaces.open({ id: "settings", identity: "settings", surfaceType: "studio.settings", title: "Settings", icon: "settings", pinned: true });
  assert.deepEqual(surfaces.snapshot().inputs.map((input) => input.surfaceType), ["case.perspective", "material.image", "data.table", "studio.settings"]);
  surfaces.close("settings");
  assert.equal(surfaces.snapshot().activeId, "surface:preview");
});

test("qualified media types resolve without file-extension guessing", () => {
  assert.equal(resolveMaterialSurfaceType("text/markdown"), "material.markdown");
  assert.equal(resolveMaterialSurfaceType("image/png"), "material.image");
  assert.equal(resolveMaterialSurfaceType("application/pdf"), "material.pdf");
  assert.equal(resolveMaterialSurfaceType("application/json"), "material.structured-text");
  assert.equal(resolveMaterialSurfaceType("audio/wav"), "material.audio");
  assert.equal(resolveMaterialSurfaceType("video/webm"), "material.video");
  assert.equal(resolveMaterialSurfaceType("application/vnd.yai.table+json"), "data.table");
  assert.equal(resolveMaterialSurfaceType("application/octet-stream"), "material.unavailable");
});

test("surface and Inspector locations share the Workbench navigation history", () => {
  const navigation = new NavigationService();
  const timeline = { caseRef: "case:qualification", view: "Memory", surfaceId: "surface:timeline", selection: "event:1", auxiliary: "Inspector" };
  const source = { caseRef: "case:qualification", view: "Environment", surfaceId: "surface:source", selection: "source:repo", auxiliary: "Inspector" };
  navigation.push(timeline);
  navigation.push(source);
  assert.deepEqual(navigation.back(), timeline);
  assert.deepEqual(navigation.forward(), source);
});

test("Workbench registries expose registered regions and remove disposed contributions", () => {
  const registry = new WorkbenchRegistry();
  const component = () => null;
  const surface = { id: "perspective:Memory", identity: "perspective:Memory", surfaceType: "case.perspective", title: "Memory", icon: "memory", pinned: true };
  registry.registerViewContainer({ id: "Memory", title: "Memory", icon: "memory", order: 1, surface });
  registry.registerViewContainer({ id: "Overview", title: "Overview", icon: "overview", order: 0, surface: { ...surface, id: "perspective:Overview", identity: "perspective:Overview", title: "Overview", icon: "overview" } });
  const view = registry.registerView({ id: "Memory.timeline", containerId: "Memory", title: "Timeline", order: 0, component });
  registry.registerSurfaceRenderer({ type: "case.perspective", role: "projection", capabilities: ["navigable"], component });
  registry.registerSurfaceRenderer({ type: "material.markdown", role: "content", capabilities: ["previewable", "searchable"], component });
  registry.registerSurfaceRenderer({ type: "material.image", role: "content", capabilities: ["previewable", "zoomable"], component });
  registry.registerSurfaceRenderer({ type: "data.table", role: "projection", capabilities: ["previewable", "selectable"], component });
  registry.registerPanelView({ id: "Terminal", title: "Terminal", order: 0, component });
  registry.registerAuxiliaryView({ id: "Inspector", title: "Inspector", order: 0, component });
  registry.registerInspector({ kind: "default", component });
  assert.deepEqual(registry.viewContainers().map((item) => item.id), ["Overview", "Memory"]);
  assert.equal(registry.viewsFor("Memory").length, 1);
  assert.equal(registry.surfaceRenderer("case.perspective").component, component);
  assert.deepEqual(registry.surfaceRenderer("case.perspective").capabilities, ["navigable"]);
  assert.equal(registry.surfaceRenderer("material.image").role, "content");
  assert.equal(registry.surfaceRenderer("material.image").type, "material.image");
  assert.equal(registry.surfaceRenderer("data.table").type, "data.table");
  assert.equal(registry.panelViews()[0].id, "Terminal");
  assert.equal(registry.auxiliaryViews()[0].id, "Inspector");
  assert.equal(registry.inspector("source").component, component);
  view.dispose();
  assert.equal(registry.viewsFor("Memory").length, 0);
});

test("settings registry owns ordered definitions and disposal", () => {
  const registry = new WorkbenchRegistry();
  const local = registry.settings.register({ id: "workbench.preview", title: "Preview", description: "Reuse preview", section: "Workbench", scope: "local", control: "boolean", defaultValue: true, available: true });
  registry.settings.register({ id: "host.status", title: "Host", description: "Resident host posture", section: "YAI Host", scope: "host", control: "information", available: false, unavailableReason: "Not implemented" });
  assert.deepEqual(registry.settings.entries().map((item) => item.id), ["workbench.preview", "host.status"]);
  local.dispose();
  assert.deepEqual(registry.settings.entries().map((item) => item.id), ["host.status"]);
});
