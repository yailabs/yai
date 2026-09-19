const test = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const generated = path.resolve(__dirname, "../../build/studio-kernel-test");
const studio = path.join(generated, "studio/src");
const { ContextKeyService, when } = require(path.join(studio, "platform/context.js"));
const { CommandService } = require(path.join(studio, "platform/commands.js"));
const { MenuService } = require(path.join(studio, "platform/menus.js"));
const { KeybindingService } = require(path.join(studio, "platform/keybindings.js"));
const { EditorGroupService } = require(path.join(studio, "workbench/editor/model.js"));
const { WorkbenchRegistry } = require(path.join(studio, "workbench/kernel/registry.js"));

test("commands register, gate, execute and dispose without global state", async () => {
  const context = new ContextKeyService();
  const commands = new CommandService(context);
  let executions = 0;
  const registration = commands.registerCommand({ id: "studio.test", title: "Test", when: when.truthy("case.attached"), handler: () => executions++ });
  assert.equal(await commands.executeCommand("studio.test"), false);
  context.update("case.attached", true);
  assert.equal(await commands.executeCommand("studio.test"), true);
  assert.equal(executions, 1);
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

test("editor group reuses preview input, pins and restores an adjacent input on close", () => {
  const editors = new EditorGroupService();
  editors.open({ id: "perspective:Overview", type: "perspective", title: "Overview", perspective: "Overview", pinned: true });
  editors.open({ id: "preview:material", type: "material", title: "One", resourceRef: "one", pinned: false });
  editors.open({ id: "preview:material", type: "material", title: "Two", resourceRef: "two", pinned: false });
  assert.deepEqual(editors.snapshot().inputs.map((input) => input.title), ["Overview", "Two"]);
  editors.pin("preview:material");
  assert.equal(editors.snapshot().activeId, "material:two");
  editors.close("material:two");
  assert.equal(editors.snapshot().activeId, "perspective:Overview");
});

test("Workbench registries expose registered regions and remove disposed contributions", () => {
  const registry = new WorkbenchRegistry();
  const component = () => null;
  registry.registerViewContainer({ id: "Memory", title: "Memory", icon: "memory", order: 1 });
  registry.registerViewContainer({ id: "Overview", title: "Overview", icon: "overview", order: 0 });
  const view = registry.registerView({ id: "Memory.timeline", containerId: "Memory", title: "Timeline", order: 0, component });
  registry.registerEditor({ type: "perspective", component });
  registry.registerPanelView({ id: "Terminal", title: "Terminal", order: 0, component });
  registry.registerAuxiliaryView({ id: "Inspector", title: "Inspector", order: 0, component });
  registry.registerInspector({ kind: "default", component });
  assert.deepEqual(registry.viewContainers().map((item) => item.id), ["Overview", "Memory"]);
  assert.equal(registry.viewsFor("Memory").length, 1);
  assert.equal(registry.editor("perspective").component, component);
  assert.equal(registry.panelViews()[0].id, "Terminal");
  assert.equal(registry.auxiliaryViews()[0].id, "Inspector");
  assert.equal(registry.inspector("source").component, component);
  view.dispose();
  assert.equal(registry.viewsFor("Memory").length, 0);
});
