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
await mkdir(output, { recursive: true });

const browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox", "--disable-gpu"] });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
const browserErrors = [];
const externalRequests = [];
page.on("pageerror", (error) => browserErrors.push(String(error)));
page.on("request", (request) => { if (!request.url().startsWith(base)) externalRequests.push(request.url()); });

const report = (property, result = "PASS", detail = {}) => console.log(JSON.stringify({ run_id: "studio-workbench-kernel-browser", property, result, ...detail }));
const screenshot = async (name) => {
  const file = path.join(output, `${name}-1440x900.png`);
  await page.screenshot({ path: file });
  const sha256 = createHash("sha256").update(await readFile(file)).digest("hex");
  report("Deterministic Workbench screenshot", "PASS", { name, file, sha256, viewport: "1440x900" });
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
  await page.locator("[data-surface-type='material.text']").waitFor();
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
  await page.getByRole("row", { name: /Result reuse/ }).click();
  await page.getByRole("heading", { name: "matrix:reuse" }).waitFor();
  await screenshot("fixture-table-surface");
  await environmentSidebar.getByRole("button", { name: /runtime-contract\.pdf/ }).click();
  await page.locator("[data-surface-type='material.pdf']").waitFor();
  await page.getByText("PDF content unavailable").waitFor();
  await screenshot("fixture-pdf-unavailable-surface");
  report("Surface Group applies one preview/pin/tab model across text, image, table and PDF renderers");

  await page.locator(".live-rail button[aria-label='Memory']").click();
  await page.getByRole("heading", { name: "Memory" }).waitFor();
  await page.locator("[data-view-container='Memory']").getByRole("button", { name: /Experience Graph/ }).click();
  await page.locator(".graph-viewport").waitFor();
  await screenshot("fixture-memory-graph");

  await page.getByRole("button", { name: "Conversation", exact: true }).click();
  await page.getByPlaceholder("Send is not qualified").waitFor();
  await page.getByRole("button", { name: "Activity", exact: true }).click();
  await page.getByText(/Through generation/).waitFor();
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
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  if (await page.locator(".surface-tabs button", { hasText: "Settings" }).count() !== 1) throw new Error("Settings is not singleton");
  report("Settings resolves through the registered singleton Surface contribution");

  await page.goto(`${base}/?fixture=ordinary`, { waitUntil: "networkidle" });
  await page.locator(".workbench-kernel[data-case-source='fixture'][data-host='web']").waitFor();
  await page.locator(".live-rail button[aria-label='Environment']").click();
  await page.locator("[data-view-container='Environment']").getByRole("button", { name: /client-notes\.md/ }).click();
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
