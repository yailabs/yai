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
const { SurfaceBufferService } = require(path.join(studio, "workbench/surface/buffers.js"));
const { WorkbenchRegistry } = require(path.join(studio, "workbench/kernel/registry.js"));
const { rendererChoices, resolveMaterialSurfaceType } = require(path.join(studio, "contrib/surfaces/inputs.js"));
const { materialIdentity, validateMaterialRead, MaterialReadFence } = require(path.join(studio, "contrib/surfaces/materialIdentity.js"));
const { detectEditorLanguage } = require(path.join(studio, "contrib/surfaces/editorLanguage.js"));
const { buildFileTree } = require(path.join(studio, "contrib/case/environment.js"));

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
  assert.deepEqual(menus.getMenu("View").map((item) => item.checked), [false, false]);
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

test("Open With exposes trusted alternatives from qualified media type", () => {
  assert.deepEqual(rendererChoices("text/markdown").map((item) => item.title), ["Text Editor", "Markdown Preview"]);
  assert.deepEqual(rendererChoices("image/svg+xml").map((item) => item.title), ["SVG/Text Editor", "Image Preview"]);
  assert.deepEqual(rendererChoices("text/csv").map((item) => item.title), ["Text Editor", "Table"]);
});

test("qualified file paths form a hierarchy without a filesystem scan", () => {
  const files = [
    { id: "cargo", path: "application/yai-application/Cargo.toml" },
    { id: "readme", path: "studio/README.md" },
    { id: "roadmap", path: "studio/ROADMAP.md" },
  ];
  const tree = buildFileTree(files);
  assert.deepEqual(tree.map((node) => node.name), ["application", "studio"]);
  assert.equal(tree[0].children[0].children[0].fileId, "cargo");
  assert.deepEqual(tree[1].children.map((node) => node.fileId), ["readme", "roadmap"]);
});

test("surface buffers retain dirty edits across renderer unmounts and revert explicitly", () => {
  const buffers = new SurfaceBufferService();
  const source = (path, digest, generation = 4) => ({ key: `${path}:${digest}:${generation}`, caseRef: "case:q", objectRef: `file:${path}`, sourceRef: "source:repo", revisionRef: `revision:${digest}`, path, digest, generation });
  assert.equal(buffers.initialize("material:readme", "alpha", source("README.md", "a")).dirty, false);
  buffers.update("material:readme", "alpha beta");
  assert.equal(buffers.snapshot("material:readme").dirty, true);
  buffers.initialize("material:readme", "remote revision", source("README.md", "b", 5));
  assert.equal(buffers.snapshot("material:readme").value, "alpha beta");
  assert.equal(buffers.snapshot("material:readme").stale, true);
  buffers.reload("material:readme");
  assert.equal(buffers.snapshot("material:readme").value, "remote revision");
  assert.equal(buffers.snapshot("material:readme").source.digest, "b");
  assert.equal(buffers.snapshot("material:readme").stale, false);
  buffers.update("material:readme", "local again");
  buffers.revert("material:readme");
  assert.equal(buffers.snapshot("material:readme").value, "remote revision");
  assert.equal(buffers.snapshot("material:readme").dirty, false);
  buffers.discard("material:readme");
  assert.equal(buffers.snapshot("material:readme"), undefined);
});

test("surface buffers fence preview, pinned, dirty and renderer-switch identities", () => {
  const buffers = new SurfaceBufferService();
  const source = (objectRef, path, digest) => ({ key: `${objectRef}:${path}:${digest}`, caseRef: "case:q", objectRef, sourceRef: "source:repo", revisionRef: `revision:${digest}`, path, digest, generation: 9 });
  const a = source("file:a", "one/README.md", "a");
  const b = source("file:b", "two/README.md", "b");
  buffers.initialize("material:file:a", "A_SENTINEL", a);
  buffers.initialize("material:file:b", "B_SENTINEL", b);
  assert.equal(buffers.snapshot("material:file:a").value, "A_SENTINEL");
  assert.equal(buffers.snapshot("material:file:b").value, "B_SENTINEL");
  buffers.update("material:file:a", "DIRTY_A");
  buffers.initialize("material:file:a", "A_NEW", source("file:a", "one/README.md", "a2"));
  assert.equal(buffers.snapshot("material:file:a").value, "DIRTY_A");
  assert.equal(buffers.snapshot("material:file:a").stale, true);
  buffers.initialize("material:file:b", "B_SENTINEL", b);
  assert.equal(buffers.snapshot("material:file:b").value, "B_SENTINEL");
});

test("exact material identity rejects mismatched fields and stale asynchronous reads", () => {
  const expected = materialIdentity({ objectRef: "file:b", caseRef: "case:q", sourceRef: "source:b", revisionRef: "revision:b", path: "b.json", digest: "digest-b", generation: 12, mediaType: "application/json", bytes: 11 });
  const actual = { case_ref: "case:q", source_ref: "source:b", revision_ref: "revision:b", path: "b.json", digest: "digest-b", generation: 12, media_type: "application/json", bytes: 11, encoding: "utf-8", content: "B_SENTINEL!" };
  assert.equal(validateMaterialRead(expected, actual), undefined);
  for (const [field, value] of [["path", "a.json"], ["source_ref", "source:a"], ["revision_ref", "revision:a"], ["digest", "digest-a"], ["generation", 11]]) {
    assert.match(validateMaterialRead(expected, { ...actual, [field]: value }), new RegExp(`material_identity_mismatch:${field}`));
  }
  const fence = new MaterialReadFence();
  const a = fence.begin("A");
  const b = fence.begin("B");
  assert.equal(fence.accepts(a, "B"), false);
  assert.equal(fence.accepts(b, "B"), true);
  const c = fence.begin("C");
  assert.equal(fence.accepts(b, "C"), false);
  assert.equal(fence.accepts(c, "C"), true);
});

test("editor language selection distinguishes qualified Studio and YAI materials", () => {
  assert.equal(detectEditorLanguage("studio/README.md", "text/markdown"), "markdown");
  assert.equal(detectEditorLanguage("tests/policy.json", "application/json"), "json");
  assert.equal(detectEditorLanguage("application/Cargo.toml", "text/plain"), "toml");
  assert.equal(detectEditorLanguage("src/lib.rs", "text/plain"), "rust");
  assert.equal(detectEditorLanguage("src/view.tsx", "text/plain"), "typescript");
  assert.equal(detectEditorLanguage("tools/check.sh", "text/plain"), "shell");
  assert.equal(detectEditorLanguage("notes.txt", "text/plain"), "plain");
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
