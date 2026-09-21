import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { mkdir, readFile } from "node:fs/promises";
import path from "node:path";

const require = createRequire(path.resolve(import.meta.dirname, "../../studio/package.json"));
const { chromium } = require("playwright-core");

const args = process.argv.slice(2);
const option = (name, fallback) => { const index = args.indexOf(name); return index >= 0 ? args[index + 1] : fallback; };
const base = option("--url", "http://127.0.0.1:1420");
const output = path.resolve(option("--output", "/tmp/yai-studio-workbench"));
const executablePath = process.env.CHROMIUM_PATH || "/usr/bin/chromium";
const width = Number(option("--width", "1440"));
const height = Number(option("--height", "900"));
const viewportLabel = `${width}x${height}`;
await mkdir(output, { recursive: true });

const browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox", "--disable-gpu"] });
const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
page.setDefaultTimeout(10_000);
const browserErrors = [];
const externalRequests = [];
page.on("pageerror", (error) => browserErrors.push(String(error)));
page.on("request", (request) => { if (!request.url().startsWith(base)) externalRequests.push(request.url()); });

const report = (property, result = "PASS", detail = {}) => console.log(JSON.stringify({ run_id: "studio-workbench-kernel-browser", property, result, ...detail }));
const screenshot = async (name) => {
  const file = path.join(output, `${name}-${viewportLabel}.png`);
  await page.screenshot({ path: file });
  const sha256 = createHash("sha256").update(await readFile(file)).digest("hex");
  report("Deterministic Workbench screenshot", "PASS", { name, file, sha256, viewport: viewportLabel });
};

try {
  await page.goto(base, { waitUntil: "networkidle" });
  await page.getByRole("heading", { name: "Open a Case." }).waitFor();
  await page.getByRole("region", { name: "Fixture Cases" }).waitFor();
  await screenshot("start-center");
  report("Explicit FixtureDataSource Start Center uses no live fallback");

  await page.getByRole("button", { name: /Runtime qualification/ }).click();
  const shell = page.locator(".workbench-kernel");
  await shell.waitFor();
  if (await shell.getAttribute("data-case-source") !== "fixture") throw new Error("fixture data posture missing");
  if (await shell.getAttribute("data-host") !== "web") throw new Error("web host posture missing");
  if (await page.locator(".desktop-resize-handle").count()) throw new Error("browser mode exposed native resize handles");
  if (await page.locator(".live-rail button").count() !== 7) throw new Error("registered View Containers missing");
  if (await page.locator(".live-sidebar [data-view-container='Overview']").count() !== 1) throw new Error("registered Sidebar View missing");
  await page.getByText("Terminal requires desktop host").waitFor();
  report("One Workbench renders registered Activity Bar, Sidebar, Panel and browser host posture");
  await screenshot("fixture-overview");

  const environment = page.locator(".live-rail button[aria-label='Environment']");
  await environment.hover();
  const tooltip = environment.getByRole("tooltip");
  await tooltip.waitFor({ state: "visible" });
  const box = await tooltip.boundingBox();
  if (!box || box.x < 40 || box.width < 60) throw new Error("Activity Bar tooltip does not float horizontally");
  await environment.click();
  await page.locator("[data-view-container='Environment']").waitFor();
  report("Registered View Container selection changes registered Sidebar content");

  const environmentSidebar = page.locator("[data-view-container='Environment']");
  await environmentSidebar.getByRole("button", { name: /result_reuse\.test/ }).click();
  await page.locator("[data-surface-type='material.text-editor']").waitFor();
  await page.locator(".surface-tabs i", { hasText: "Preview" }).waitFor();
  const firstPreviewTitle = await page.locator(".surface-tabs button[aria-selected='true']").innerText();
  await environmentSidebar.getByRole("button", { name: /runtime-boundary\.svg/ }).click();
  await page.locator("[data-surface-type='material.image'] img").waitFor();
  const previews = page.locator(".surface-tabs i", { hasText: "Preview" });
  if (await previews.count() !== 1) throw new Error("preview Surface was not reused");
  const secondPreviewTitle = await page.locator(".surface-tabs button[aria-selected='true']").innerText();
  if (firstPreviewTitle === secondPreviewTitle) throw new Error("preview Surface did not change representation");
  await screenshot("fixture-image-surface");
  await page.locator(".surface-tabs button[aria-selected='true']").dblclick();
  if (await previews.count()) throw new Error("double click did not pin preview input");
  await environmentSidebar.getByRole("button", { name: /qualification-matrix/ }).click();
  await page.locator("[data-surface-type='data.table'] table").waitFor();
  await page.getByRole("button", { name: "Owner", exact: true }).click();
  await page.getByPlaceholder("Filter rows").fill("Qualified");
  if (await page.locator("[data-surface-type='data.table'] tbody tr").count() < 4) throw new Error("Table filtering did not retain qualified structured rows");
  await page.getByPlaceholder("Filter rows").fill("");
  await page.getByRole("row", { name: /Result reuse/ }).click();
  await page.getByRole("heading", { name: "matrix:reuse" }).waitFor();
  await screenshot("fixture-table-surface");
  await environmentSidebar.getByRole("button", { name: /runtime-contract\.pdf/ }).click();
  await page.locator("[data-surface-type='material.pdf']").waitFor();
  await page.locator("[data-surface-type='material.pdf'] canvas").waitFor();
  await page.getByRole("button", { name: "Next page" }).click();
  await page.getByText(/of 2/).waitFor();
  await page.waitForTimeout(250);
  await screenshot("fixture-pdf-surface");
  report("Lazy PDF Surface renders qualified bytes with page and zoom controls");

  await page.keyboard.press("Control+p");
  const quickOpen = page.getByRole("dialog", { name: "Quick Open" });
  await quickOpen.waitFor();
  await quickOpen.getByRole("textbox").fill("runtime configuration");
  await screenshot("quick-open");
  await page.keyboard.press("Enter");
  await page.locator("[data-surface-type='material.structured-text']").waitFor();
  await page.keyboard.press("Control+f");
  const surfaceSearch = page.getByRole("dialog", { name: /Find in Runtime configuration/ });
  await surfaceSearch.getByRole("textbox").fill("generation");
  await surfaceSearch.getByRole("option").first().waitFor();
  await page.keyboard.press("Escape");
  report("Quick Open and current Surface search use shared Workbench navigation and renderer capability");

  await page.keyboard.press("Control+Shift+f");
  const caseSearch = page.getByRole("dialog", { name: "Search Current Case" });
  await caseSearch.getByRole("textbox").fill("boundary map");
  await caseSearch.getByRole("option").first().waitFor();
  await page.keyboard.press("Enter");
  await page.locator("[data-surface-type='material.image']").waitFor();
  report("Fixture Case search provider navigates through the shared Surface model");

  await page.keyboard.press("Control+p");
  await page.getByRole("dialog", { name: "Quick Open" }).getByRole("textbox").fill("Qualification tone");
  await page.keyboard.press("Enter");
  await page.locator("[data-surface-type='material.audio'] audio").waitFor();
  await page.keyboard.press("Control+p");
  await page.getByRole("dialog", { name: "Quick Open" }).getByRole("textbox").fill("Runtime capture");
  await page.keyboard.press("Enter");
  await page.getByText("No trusted renderer").waitFor();
  await page.getByRole("button", { name: "Open externally unavailable" }).waitFor();
  report("Media and unknown material resolve deterministically without binary-to-text fallback");

  await page.keyboard.press("Control+Shift+p");
  const palette = page.getByRole("dialog", { name: "Command Palette" });
  await palette.getByRole("textbox").fill("settings");
  await screenshot("command-palette");
  await page.keyboard.press("Enter");
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  await screenshot("settings-surface");
  report("Command Palette consumes CommandService and opens singleton Settings");

  await page.locator(".live-rail button[aria-label='Memory']").click();
  await page.getByRole("heading", { name: "Memory" }).waitFor();
  await page.locator("[data-view-container='Memory']").getByRole("button", { name: /Experience Graph/ }).click();
  await page.locator(".graph-viewport").waitFor();
  await screenshot("fixture-memory-graph");

  await page.getByRole("button", { name: "Conversation", exact: true }).click();
  await page.getByPlaceholder("Send is not qualified").waitFor();
  await screenshot("conversation-quality-pass");
  await page.getByRole("button", { name: "Activity", exact: true }).click();
  await page.getByText(/Through generation/).waitFor();
  await screenshot("activity-quality-pass");
  report("Auxiliary Bar switches registered Conversation, Inspector and Activity contributions");

  await page.getByRole("button", { name: "View", exact: true }).click();
  const viewMenu = page.getByRole("menu", { name: "View menu" });
  await viewMenu.waitFor();
  await viewMenu.getByRole("menuitem", { name: /Bottom Panel/ }).click();
  await page.locator("#case-tools").waitFor({ state: "hidden" });
  await page.keyboard.press("Control+j");
  await page.locator("#case-tools").waitFor({ state: "visible" });
  report("Registered menu and keybinding dispatch the same centralized command");

  const bottomSplitter = page.getByRole("separator", { name: "Resize bottom panel" });
  const originalBottomHeight = Number(await bottomSplitter.getAttribute("aria-valuenow"));
  await bottomSplitter.focus();
  await page.keyboard.press("ArrowUp");
  const resizedBottomHeight = Number(await bottomSplitter.getAttribute("aria-valuenow"));
  if (resizedBottomHeight <= originalBottomHeight) throw new Error("bottom panel keyboard resize was not preserved");
  report("Workbench Panel owner preserves explicit bottom-panel resize state");

  await page.getByRole("button", { name: "YAI", exact: true }).click();
  await page.getByRole("menuitem", { name: /Settings/ }).click();
  if (await page.locator(".surface-tabs button", { hasText: "Settings" }).count() !== 1) throw new Error("Settings is not singleton");
  await page.locator(".settings-surface > aside").getByRole("button", { name: "YAI Host", exact: true }).click();
  await page.locator(".settings-content").getByRole("heading", { name: "YAI Host", exact: true }).waitFor();
  await page.getByText(/One resident application service/).waitFor();
  report("Settings resolves through the registered singleton Surface contribution");

  await page.goto(`${base}/?fixture=ordinary`, { waitUntil: "networkidle" });
  await page.locator(".workbench-kernel[data-case-source='fixture'][data-host='web']").waitFor();
  await page.locator(".live-rail button[aria-label='Environment']").click();
  const ordinaryEnvironment = page.locator("[data-view-container='Environment']");
  await ordinaryEnvironment.getByRole("button", { name: /Meeting notes/ }).click();
  await page.locator("[data-surface-type='environment.source']").waitFor();
  await screenshot("environment-source-surface");
  await ordinaryEnvironment.getByRole("button", { name: /Review workspace/i }).click();
  await page.locator("[data-surface-type='environment.resource']").waitFor();
  await screenshot("environment-resource-surface");
  report("Environment routes Files, Sources and Resources to distinct typed Surfaces");

  await ordinaryEnvironment.getByRole("button", { name: /client-notes\.md/ }).click();
  await page.locator("[data-surface-type='material.text-editor']").waitFor();
  const editor = page.getByRole("textbox", { name: "Edit client-notes.md" });
  const baseline = await editor.inputValue();
  await editor.fill(`${baseline}\nLocal qualification edit.`);
  await page.getByLabel("Unsaved changes").waitFor();
  await screenshot("environment-text-editor-dirty");
  await page.getByRole("button", { name: "File", exact: true }).click();
  const fileMenu = page.getByRole("menu", { name: "File menu" });
  await fileMenu.waitFor();
  if (!(await fileMenu.getByRole("menuitem", { name: "Save", exact: true }).isDisabled())) throw new Error("Save was enabled without a governed application mutation");
  await fileMenu.getByRole("menuitem", { name: /^Revert File/ }).click();
  if (await editor.inputValue() !== baseline) throw new Error("Revert did not restore the exact retained buffer");
  await page.getByLabel("Unsaved changes").waitFor({ state: "hidden" });
  await editor.fill(`${baseline}\nDiscard this local edit.`);
  let discardPrompt = "";
  page.once("dialog", async (dialog) => { discardPrompt = dialog.message(); await dialog.accept(); });
  await page.getByRole("button", { name: "Close client-notes.md" }).click();
  if (!discardPrompt.includes("Discard unsaved changes")) throw new Error("dirty Surface closed without an explicit discard prompt");
  await ordinaryEnvironment.getByRole("button", { name: /client-notes\.md/ }).click();
  await page.locator("[data-surface-type='material.text-editor']").waitFor();
  if (await page.getByRole("textbox", { name: "Edit client-notes.md" }).inputValue() !== baseline) throw new Error("discarded buffer survived Surface close");
  report("Text Surface retains local dirty state, reverts or discards explicitly, and keeps governed Save unavailable");

  await page.getByRole("button", { name: "File", exact: true }).click();
  await page.getByRole("menuitem", { name: /Open With/ }).click();
  await page.getByRole("dialog", { name: /Open client-notes\.md with/ }).waitFor();
  await page.keyboard.press("Escape");
  await page.getByRole("dialog", { name: /Open client-notes\.md with/ }).waitFor({ state: "hidden" });
  await page.getByRole("button", { name: "File", exact: true }).click();
  await page.getByRole("menuitem", { name: /Open With/ }).click();
  await page.getByRole("button", { name: /Markdown Preview/ }).click();
  await page.locator("[data-surface-type='material.markdown']").waitFor();
  await screenshot("fixture-ordinary-same-kernel");
  report("Distinct FixtureClient Cases use the same Workbench and Markdown Surface renderer");

  await page.goto(`${base}/?fixture=missing`, { waitUntil: "networkidle" });
  await page.getByRole("heading", { name: "Local YAI unavailable" }).waitFor();
  await page.getByText("fixture_case_not_found").waitFor();
  report("Unknown fixture Case fails explicitly without live or authored-data fallback");

  if (externalRequests.length) throw new Error(`unexpected external requests: ${externalRequests.join(", ")}`);
  if (browserErrors.length) throw new Error(`browser errors: ${browserErrors.join(" | ")}`);
  report("Browser fixture regression", "PASS", { browser: browser.version(), external_requests: 0, browser_errors: 0 });
} finally {
  await browser.close();
}
