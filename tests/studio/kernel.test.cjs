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
const { materialIdentity, validateMaterialRead, validateMaterialContent, MaterialReadFence } = require(path.join(studio, "contrib/surfaces/materialIdentity.js"));
const { detectEditorLanguage } = require(path.join(studio, "contrib/surfaces/editorLanguage.js"));
const { buildFileTree } = require(path.join(studio, "contrib/case/environment.js"));

test('context tools retain local state within exactly one Case and Participant', () => {
  const {WorkbenchSession}=require(path.join(studio,'workbench/kernel/session.js'));
  const {fitTool}=require(path.join(studio,'workbench/kernel/contextTools.js'));
  const window=new WorkbenchSession();
  const a=window.forCase('case:a','participant:one');
  a.contextTools.update('Inspector',{open:true,floating:true,pinnedRef:'source:a',bounds:{x:1400,y:900,width:500,height:900}});
  for(const other of [window.forCase('case:b','participant:one'),window.forCase('case:a','participant:two')]) {
    assert.deepEqual(other.contextTools.snapshot(),{});
    assert.notEqual(other.buffers,a.buffers);
    assert.notEqual(other.navigation,a.navigation);
  }
  assert.equal(window.forCase('case:a','participant:one'),a);
  a.contextTools.update('Inspector',{open:false});
  assert.equal(a.contextTools.get('Inspector').pinnedRef,'source:a');
  for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
    const box=fitTool(a.contextTools.get('Inspector').bounds,{width,height});
    assert.ok(box.x>=12&&box.y>=72&&box.x+box.width<=width-12&&box.y+box.height<=height-32);
  }
  a.contextTools.reset();assert.deepEqual(a.contextTools.snapshot(),{});window.dispose();
});

test("policy routing uses declared roles without losing dual-role material or inferring filenames", () => {
  const { environmentMaterials, isPolicySource } = require(path.join(studio, "contrib/case/environment.js"));
  const { sourceInput } = require(path.join(studio, "contrib/surfaces/inputs.js"));
  const sources = [
    {id:'source:rules',label:'ordinary.txt',roles:['policy']},
    {id:'source:handbook',label:'handbook.md',roles:['policy','knowledge']},
    {id:'source:documentation',label:'policy.json',roles:['knowledge']},
  ];
  const files=sources.map(source=>({id:`file:${source.id}`,source_ref:source.id,path:source.label}));
  const workspace={environment:{sources,files}};
  const shown=environmentMaterials(workspace);
  assert.deepEqual(shown.sources.map(item=>item.id),['source:handbook','source:documentation']);
  assert.deepEqual(shown.files.map(item=>item.path),['handbook.md','policy.json']);
  assert.equal(isPolicySource(sources[2]),false);
  assert.equal(sourceInput(workspace,'source:rules').viewId,'Authority');
  assert.equal(sourceInput(workspace,'source:handbook').viewId,'Authority');
  assert.equal(sourceInput(workspace,'source:documentation').viewId,'Environment');
  assert.equal(workspace.environment.files.length,3,'presentation must not remove original provenance');
});

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
  surfaces.pin("material:diagram");
  surfaces.open({ id: "surface:preview", identity: "material:matrix", surfaceType: "data.table", title: "Matrix", icon: "file", objectRef: "matrix", pinned: false });
  surfaces.open({ id: "settings", identity: "settings", surfaceType: "studio.settings", title: "Settings", icon: "settings", pinned: true });
  assert.deepEqual(surfaces.snapshot().inputs.map((input) => input.surfaceType), ["case.perspective", "material.image", "data.table", "studio.settings"]);
  surfaces.close("settings");
  assert.equal(surfaces.snapshot().activeId, "material:matrix");
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

test("rail preferences cannot hide core or confuse platform and pinned contributions", () => {
  const registry = new WorkbenchRegistry();
  for (const [id, order, section, fixed, defaultPinned] of [
    ["Overview", 0, "core", true, true], ["Providers", 20, "platform", true, true],
    ["tool-A", 30, "pinned", false, true], ["tool-B", 31, "pinned", false, false],
  ]) registry.registerViewContainer({id, title:id, icon:"case", order,
    rail:{section,fixed,defaultPinned}, surface:{id,identity:id,surfaceType:"test",title:id,icon:"case",pinned:true}});
  const ids = preferences => registry.railContainers(preferences).map(item => item.id);
  assert.deepEqual(ids({hidden:[], order:[]}), ["Overview", "Providers", "tool-A"]);
  assert.deepEqual(ids({hidden:["Overview","Providers","tool-A"], order:["tool-B"]}), ["Overview", "Providers", "tool-B"]);
  assert.deepEqual(ids({hidden:[], order:["tool-B","tool-A","Providers","Overview"]}), ["Overview", "Providers", "tool-B", "tool-A"]);
  assert.equal(registry.viewContainers().length, 4, "Unpinning does not deregister navigation");
  assert.deepEqual(ids({hidden:[], order:[]}), ["Overview", "Providers", "tool-A"], "Restore defaults");
});

test("settings registry owns ordered definitions and disposal", () => {
  const registry = new WorkbenchRegistry();
  const local = registry.settings.register({ id: "workbench.preview", title: "Preview", description: "Reuse preview", section: "Workbench", scope: "local", control: "boolean", defaultValue: true, available: true });
  registry.settings.register({ id: "host.status", title: "Host", description: "Resident host posture", section: "YAI Host", scope: "host", control: "information", available: false, unavailableReason: "Not implemented" });
  assert.deepEqual(registry.settings.entries().map((item) => item.id), ["workbench.preview", "host.status"]);
  local.dispose();
  assert.deepEqual(registry.settings.entries().map((item) => item.id), ["host.status"]);
});


test("a material has one tab, editing claims preview, no-op updates do not notify", () => {
  const group = new SurfaceGroupService();
  const input = (name, pinned = false) => ({ id: "surface:preview", identity: name, surfaceType: "material.text-editor", title: "README.md", icon: "file", pinned });
  group.open(input("A"));
  let updates = 0;
  group.subscribe(() => updates++);
  group.update("A", { dirty: false });
  const once = updates;
  group.update("A", { dirty: false });
  assert.equal(updates, once);
  group.update("A", { dirty: true });
  group.open(input("B"));
  assert.equal(group.snapshot().inputs.length, 2);
  assert.equal(group.snapshot().inputs[0].pinned, true);
  group.open(input("A"));
  assert.equal(group.snapshot().inputs.length, 2);
  assert.equal(group.snapshot().activeId, "A");
  assert.equal(group.snapshot().inputs[0].dirty, true);
  group.replace("A", { ...input("A", true), surfaceType: "material.markdown", dirty: true });
  group.open(input("A"));
  assert.equal(group.snapshot().inputs[0].surfaceType, "material.markdown");
});

test("Revert accepts the incoming baseline instead of falsely marking the old revision current", () => {
  const buffers = new SurfaceBufferService();
  const source = (key) => ({ key, caseRef: "case:A", objectRef: "file:A", sourceRef: "source:A", revisionRef: key, path: "README.md", digest: key, generation: 1 });
  buffers.initialize("A", "old", source("r1"));
  buffers.update("A", "local edit");
  buffers.initialize("A", "incoming", source("r2"));
  assert.equal(buffers.snapshot("A").value, "local edit");
  assert.equal(buffers.dirtyCount, 1);
  buffers.revert("A");
  assert.equal(buffers.snapshot("A").value, "incoming");
  assert.equal(buffers.snapshot("A").source.revisionRef, "r2");
  assert.equal(buffers.snapshot("A").stale, false);
  assert.equal(buffers.dirtyCount, 0);
});

test("material byte validation refuses equal-length contamination behind correct metadata", async () => {
  const { createHash } = require("node:crypto");
  const digest = createHash("sha256").update("AAA").digest("hex");
  const material = { encoding: "utf-8", content: "AAA", bytes: 3, digest };
  assert.equal(await validateMaterialContent(material), undefined);
  assert.equal(await validateMaterialContent({ ...material, digest: `sha256:${digest}` }), undefined);
  assert.equal(await validateMaterialContent({ ...material, content: "BBB" }), "material_identity_mismatch:content_digest");
  assert.equal(await validateMaterialContent({ ...material, encoding: "base64", content: Buffer.from("AAA").toString("base64") }), undefined);
});


test("unrelated Case generation does not stale a dirty exact revision", () => {
  const buffers = new SurfaceBufferService();
  const source = { key: "g1", caseRef: "case:A", objectRef: "file:A", sourceRef: "source:A", revisionRef: "r1", path: "README.md", digest: "digest1", generation: 1 };
  buffers.initialize("A", "baseline", source);
  buffers.update("A", "local draft");
  buffers.initialize("A", "baseline", { ...source, key: "g2", generation: 2 });
  assert.equal(buffers.snapshot("A").value, "local draft");
  assert.equal(buffers.snapshot("A").stale, false);
  assert.equal(buffers.snapshot("A").source.generation, 2);
  assert.throws(() => buffers.initialize("A", "wrong file", { ...source, key: "other", path: "other.md" }), /identity/);
});

test("graph accounts for every qualified endpoint without inventing object detail or relations", () => {
  const { knowledgeGraph, completeGraph, graphSlice } = require(path.join(studio, "contrib/case/graph.js"));
  const edges = [{ id: "e:1", from: "unit:1", to: "document:1", kind: "derived_from" }, { id: "e:2", from: "document:1", to: "resource:1", kind: "backed_by" }, { id: "e:3", from: "unit:1", to: "missing:1", kind: "references" }];
  const workspace = { knowledge: { sources: [{ id: "document:1", path: "guide.md" }], units: [{ id: "unit:1", text: "EXACT_UNIT_SENTINEL", kind: "text_block" }], entities: [], contradictions: [], relations: edges }, environment: { sources: [], files: [], resources: [{ id: "resource:1", label: "Repository" }] } };
  const graph = knowledgeGraph(workspace);
  assert.equal(graph.nodes.length, 4);
  assert.deepEqual(graph.edges, edges);
  for (const edge of graph.edges) for (const id of [edge.from, edge.to]) assert.ok(graph.nodes.some(node => node.id === id));
  assert.equal(graph.nodes.find(node => node.id === "document:1").referenceOnly, undefined);
  assert.equal(graph.nodes.find(node => node.id === "missing:1").referenceOnly, true);
  assert.equal(graph.nodes.find(node => node.id === "unit:1").label, "EXACT_UNIT_SENTINEL");
  const many = completeGraph(Array.from({ length: 107 }, (_, i) => ({ id: `n:${i}`, label: `node ${i}` })), Array.from({ length: 106 }, (_, i) => ({ id: `e:${i}`, from: "n:0", to: `n:${i + 1}`, kind: "references" })));
  const ids = new Set(), relationIds = new Set();
  const count = graphSlice(many, "", "n:0").pages;
  for (let page = 0; page < count; page++) {
    const slice = graphSlice(many, "", "n:0", page);
    assert.ok(slice.nodes.length <= 20);
    assert.ok(slice.nodes.some(node => node.id === "n:0"));
    slice.nodes.forEach(node => ids.add(node.id)); slice.edges.forEach(edge => relationIds.add(edge.id));
  }
  assert.equal(ids.size, 107); assert.equal(relationIds.size, 106);
  assert.deepEqual(graphSlice(many, "node 106", undefined).nodes.map(node => node.id), ["n:106"]);
  assert.equal(graphSlice(many, "absent", undefined, 999).page, 0);
});

test("Inspector uses qualified Knowledge content and exact revision closure for file navigation", () => {
  const { findFact, factKind, factReferences } = require(path.join(studio, "contrib/case/facts.js"));
  const file = { id: "file:1", source_ref: "source:1", revision_ref: "r:1", path: "guide.md", digest: "digest:1" };
  const workspace = { memory: { relations: [] }, work: { edges: [], nodes: [] }, case: {}, presentation: {}, overview: { participants: [] }, environment: { files: [file, { ...file, id: "file:wrong", revision_ref: "r:2" }], sources: [], resources: [] }, knowledge: { units: [{ id: "unit:1", source_ref: "document:1", text: "EXACT_TEXT_FROM_YAI", kind: "claim", posture: "derived", references: [], topics: [] }], sources: [{ ...file, id: "document:1" }], entities: [], contradictions: [], relations: [] } };
  assert.equal(findFact(workspace, "unit:1").detail, "EXACT_TEXT_FROM_YAI");
  assert.equal(factKind(workspace, "unit:1"), "knowledge unit");
  assert.deepEqual(factReferences(workspace, "unit:1"), ["document:1"]);
  assert.deepEqual(factReferences(workspace, "document:1"), ["source:1", "file:1"]);
});

test("application discovery follows Host instances and gates operations without granting authority", async () => {
  const { ApplicationAccess, isApplicationCatalog } = require(path.join(studio, "clients/application.js"));
  const catalog = ids => ({schema:'yai.application_capability_catalog.v1',application_protocol:'yai.studio.application.v1',capabilities:[],operations:ids.map(operation_id=>({operation_id,meaning:operation_id,input_contract:'input',output_contract:'output',impact:'canonical_mutation',authority:'current_case_authority'}))});
  assert.equal(isApplicationCatalog({...catalog([]),operations:[null]}),false);
  assert.equal(isApplicationCatalog({...catalog([]),application_protocol:'future'}),false);
  let state={state:'live',telemetry:{instance_id:'host:A'}}; let notify;
  let finish; let calls=0, mutations=0;
  const host={snapshot:()=>state,subscribe:handler=>{notify=handler;return{dispose(){notify=undefined}}},capabilities:{nativeDesktop:true}};
  const client={applicationCapabilities:()=>{calls++;return new Promise(resolve=>finish=resolve)},createCase:async input=>{mutations++;return{result_state:'unauthorized',error:{code:'tenant_access_denied'},data:input}}};
  const access=new ApplicationAccess(client,host);
  assert.equal(access.supports('case.create'),false);
  assert.equal((await access.createCase({case_ref:'case:a',tenant_id:'tenant:a'})).result_state,'not_implemented');
  assert.equal(mutations,0);
  const old=finish;
  state={state:'stopped'};notify();
  old({result_state:'success',data:catalog(['case.create'])});await Promise.resolve();
  assert.equal(access.supports('case.create'),false);
  await access.refresh(); assert.equal(calls,1); // Explicit Stop cannot trigger auto-start by discovery.
  state={state:'live',telemetry:{instance_id:'host:B'}};notify();
  finish({result_state:'success',data:catalog(['case.create'])});await Promise.resolve();
  assert.equal(access.supports('case.create'),true);
  assert.equal(access.supports('review.approve'),false);
  const result=await access.createCase({case_ref:'case:a',tenant_id:'tenant:a'});
  assert.equal(result.result_state,'unauthorized');assert.equal(mutations,1); // Catalog readiness is not authority.
  notify(); assert.equal(calls,2); // Repeated telemetry is not another catalog read.
  access.dispose();assert.equal(notify,undefined);
});

test("typed application actions preserve exact inputs and never retry lost transport acknowledgements", async () => {
  const { LiveClient }=require(path.join(studio,'clients/live.js'));
  const oldWindow=global.window; const requests=[];
  global.window={__TAURI__:{core:{invoke:async(command,{request})=>{requests.push(request);if(request.operation_ref==='case.close')throw Error('lost acknowledgement');return{operation_ref:request.operation_ref,result_state:'success',correlation_ref:request.correlation_ref}}}}};
  try{
    const client=new LiveClient();
    await client.createCase({tenant_id:'tenant:chosen',case_ref:'case:chosen'});
    await client.resolveReview('deny',{case_ref:'case:chosen',review_ref:'review:exact',participant_ref:'participant:current',reason:'operator reason'});
    await client.recordWorkflowInput({case_ref:'case:chosen',node_ref:'node:exact',value:'EXACT INPUT'});
    const closed=await client.caseLifecycle('close',{case_ref:'case:chosen',reason:'operator close'});
    assert.deepEqual(requests.map(request=>request.operation_ref),['case.create','review.deny','workflow.input.record','case.close']);
    assert.deepEqual(requests[1].input,{case_ref:'case:chosen',review_ref:'review:exact',participant_ref:'participant:current',reason:'operator reason'});
    assert.equal(requests[2].input.value,'EXACT INPUT'); assert.equal(closed.result_state,'transport_unavailable');
    assert.equal(new Set(requests.map(request=>request.correlation_ref)).size,4);
  }finally{global.window=oldWindow;}
});

test('local preferences reject corrupt or unsafe values and retain bounded changes', () => {
  const {ConfigurationService}=require(path.join(studio,'platform/configuration.js'));
  const previous=global.window;let stored=JSON.stringify({version:1,values:{'terminal.scrollback':-1,'workbench.sidebar.width':99999,'workbench.openPreview':'yes','unknown':'value'}});
  global.window={localStorage:{getItem:()=>stored,setItem:(_key,value)=>{stored=value}}};
  try {
    const preferences=new ConfigurationService({'terminal.scrollback':5000,'workbench.sidebar.width':204,'workbench.openPreview':true});let changes=0;preferences.subscribe(()=>changes++);
    assert.equal(preferences.get('terminal.scrollback'),5000);assert.equal(preferences.get('workbench.sidebar.width'),204);assert.equal(preferences.get('workbench.openPreview'),true);assert.equal(preferences.get('unknown'),undefined);
    preferences.update('terminal.scrollback',Infinity);preferences.update('terminal.scrollback',-20);preferences.update('terminal.scrollback',3.14);preferences.update('workbench.sidebar.width',204);assert.equal(changes,0);
    preferences.update('terminal.scrollback',12000);preferences.update('workbench.sidebar.width',280);assert.equal(changes,2);assert.equal(JSON.parse(stored).values['workbench.sidebar.width'],280);
    const restored=new ConfigurationService({'terminal.scrollback':5000,'workbench.sidebar.width':204,'workbench.openPreview':true});assert.equal(restored.get('terminal.scrollback'),12000);assert.equal(restored.get('workbench.sidebar.width'),280);
    const rail=new ConfigurationService({'workbench.rail.order':[]});
    for(const invalid of [null,{},[1],['A','A'],Array.from({length:129},(_,i)=>String(i))]) rail.update('workbench.rail.order',invalid);
    assert.deepEqual(rail.get('workbench.rail.order'),[]);rail.update('workbench.rail.order',['B','A']);
    assert.deepEqual(new ConfigurationService({'workbench.rail.order':[]}).get('workbench.rail.order'),['B','A']);
  }finally{global.window=previous;}
});
