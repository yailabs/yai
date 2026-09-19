// Separate Studio browser proof. Never part of the backend Make graph.
// Uses the Playwright library only; no test runner or downloaded browser needed.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = fileURLToPath(new URL("../../", import.meta.url));
const requireStudio = createRequire(path.join(root, "studio/package.json"));
const { chromium } = requireStudio("playwright-core");
const args = process.argv.slice(2);
const option = (name, fallback) =>
  args.includes(name) ? args[args.indexOf(name) + 1] : fallback;
const base = option("--url", "http://127.0.0.1:1420");
assert(
  ["127.0.0.1", "localhost", "[::1]"].includes(new URL(base).hostname),
  "Use an explicitly local Studio server",
);
const output = path.resolve(
  option("--output", path.join(root, "build/studio-information")),
);
const run = option("--run", "studio-information-browser");
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
  executablePath: process.env.STUDIO_CHROMIUM || "/usr/bin/chromium",
  headless: true,
});
const context = await browser.newContext({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 1,
  colorScheme: "dark",
  reducedMotion: "reduce",
  locale: "en-GB",
  timezoneId: "UTC",
});
const page = await context.newPage();
page.setDefaultTimeout(10000);
const errors = [];
const requests = new Set();
page.on("pageerror", (error) => errors.push(error.message));
page.on("console", (message) => {
  if (message.type() === "error") errors.push(message.text());
});
page.on("request", (request) => {
  const url = new URL(request.url());
  if (!["data:", "blob:"].includes(url.protocol)) {
    requests.add(url.origin);
    if (url.origin !== new URL(base).origin || request.method() !== "GET")
      errors.push(`Unexpected request: ${request.method()} ${request.url()}`);
  }
});
const log = (property, extra = {}) =>
  console.log(
    JSON.stringify({ run_id: run, property, result: "PASS", ...extra }),
  );
async function load(search = "") {
  await page.mouse.move(0, 0);
  await page.goto(`${base}/${search}`, { waitUntil: "networkidle" });
  await page.evaluate(() => document.fonts.ready);
}
async function selectPerspective(name) {
  await page
    .getByRole("button", { name: `${name} perspective`, exact: true })
    .click();
  await page.getByRole("tab", { name, exact: true }).waitFor();
}
async function geometry(kind) {
  const dimensions = await page.evaluate((view) => {
    const rect = (selector) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      const r = element.getBoundingClientRect();
      return {
        x: r.x,
        y: r.y,
        w: r.width,
        h: r.height,
        right: r.right,
        bottom: r.bottom,
      };
    };
    return {
      view,
      body: document.body.scrollWidth,
      width: innerWidth,
      height: innerHeight,
      center: rect(".central-column"),
      sidebar: rect(".sidebar"),
      context: rect(".context-panel"),
      start: rect(".start-center"),
      setup: rect(".case-bootstrap"),
      footer: rect(".status-bar"),
    };
  }, kind);
  assert(dimensions.body <= dimensions.width, JSON.stringify(dimensions));
  assert(dimensions.footer?.bottom <= dimensions.height + 1);
  if (kind === "workbench") {
    assert(dimensions.center.w >= 390, JSON.stringify(dimensions));
    assert(
      dimensions.sidebar.right <= dimensions.center.x &&
        dimensions.center.right <= dimensions.context.x,
      JSON.stringify(dimensions),
    );
  } else {
    assert(dimensions.start || dimensions.setup, JSON.stringify(dimensions));
  }
  return dimensions;
}
async function drag(label, dx, dy) {
  const separator = page.getByRole("separator", { name: label, exact: true });
  const before = Number(await separator.getAttribute("aria-valuenow"));
  const box = await separator.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(
    box.x + box.width / 2 + dx,
    box.y + box.height / 2 + dy,
    { steps: 8 },
  );
  await page.mouse.up();
  const after = Number(await separator.getAttribute("aria-valuenow"));
  assert.notEqual(after, before, `${label} did not resize`);
}

try {
  if (!args.includes("--screenshots-only")) {
    await load();
    await page.getByRole("main", { name: "YAI Studio Start Center" }).waitFor();
    assert.equal(await page.getByText("FIXTURE", { exact: true }).count(), 1);
    await page.getByRole("button", { name: "Open Case", exact: true }).click();
    await page
      .getByPlaceholder("Search by Case or current work")
      .fill("runtime");
    const startHistoryDepth = await page.evaluate(() => history.length);
    await page
      .locator(".open-case-results")
      .getByRole("button", { name: /Runtime qualification CASE \/ 027/ })
      .click();
    await page
      .getByRole("heading", { name: "Runtime qualification", exact: true })
      .first()
      .waitFor();
    assert.equal(await page.evaluate(() => history.length), startHistoryDepth);
    assert.equal(
      await page.getByRole("button", { name: /Start Center/ }).count(),
      0,
    );
    await page.getByRole("button", { name: "File", exact: true }).click();
    await page
      .getByRole("menu", { name: "File" })
      .getByRole("menuitem", { name: "New Case from Source…", exact: true })
      .click();
    await page
      .getByRole("main", { name: "New Case fixture composition" })
      .waitFor();
    for (const label of [
      "Identity",
      "Sources",
      "Participants",
      "Authority",
      "Resources",
      "Compute",
    ])
      assert.equal(
        await page.getByRole("heading", { name: label, exact: true }).count(),
        1,
      );
    assert.equal(
      await page.locator('[data-section="sources"][data-focus="true"]').count(),
      1,
    );
    await page.getByRole("button", { name: "Close draft" }).click();
    await page
      .getByRole("heading", { name: "Runtime qualification", exact: true })
      .first()
      .waitFor();
    await page.getByRole("button", { name: "File", exact: true }).click();
    await page
      .getByRole("menu", { name: "File" })
      .getByRole("menuitem", { name: /Execution · Release handoff/ })
      .click();
    await page
      .getByRole("heading", { name: "Release handoff", exact: true })
      .first()
      .waitFor();
    await page.getByLabel("Fixture generation").selectOption("1");
    await selectPerspective("Memory");
    const earlyEvents = await page.locator(".case-timeline li").count();
    await page.getByLabel("Fixture generation").selectOption("3");
    const laterEvents = await page.locator(".case-timeline li").count();
    assert(laterEvents > earlyEvents, `${earlyEvents} !< ${laterEvents}`);
    await page.getByRole("tab", { name: "Graph", exact: true }).click();
    assert((await page.locator(".graph-node").count()) >= 6);
    await page.getByRole("tab", { name: "Activity", exact: true }).click();
    await page.getByRole("heading", { name: "Recent Case activity" }).waitFor();
    await page.getByRole("tab", { name: "Conversation", exact: true }).click();
    await page
      .getByPlaceholder("Keep a note for this conversation…")
      .fill("local fixture note");
    await page.getByRole("button", { name: "Collapse context panel" }).click();
    await page.getByRole("button", { name: "Toggle context panel" }).click();
    assert.equal(
      await page
        .getByPlaceholder("Keep a note for this conversation…")
        .inputValue(),
      "local fixture note",
    );
    await page.getByRole("tab", { name: "Inspector", exact: true }).click();
    await selectPerspective("Environment");
    await page
      .getByRole("button", { name: /Validation notes/ })
      .first()
      .click();
    await page
      .getByRole("tab", { name: "Validation notes", exact: true })
      .waitFor();
    await page.getByRole("button", { name: "Close Validation notes" }).click();
    for (const name of [
      "Overview",
      "Knowledge",
      "Authority",
      "Work",
      "Compute",
    ])
      await selectPerspective(name);
    for (const name of [
      "Terminal",
      "Output",
      "Executions",
      "Evidence",
      "Problems",
    ])
      await page.getByRole("tab", { name, exact: true }).click();
    await drag("Resize Case explorer", 35, 0);
    await drag("Resize context panel", -30, 0);
    await drag("Resize bottom panel", 0, -28);
    await page.getByRole("button", { name: "Toggle bottom panel" }).click();
    await page.keyboard.press("Control+j");
    const keyboardSplitter = page.getByRole("separator", {
      name: "Resize Case explorer",
    });
    await keyboardSplitter.focus();
    assert(
      await keyboardSplitter.evaluate(
        (element) => element === document.activeElement,
      ),
    );
    const keyboardBefore = Number(
      await keyboardSplitter.getAttribute("aria-valuenow"),
    );
    await page.keyboard.press("ArrowRight");
    assert.equal(
      Number(await keyboardSplitter.getAttribute("aria-valuenow")),
      keyboardBefore + 10,
    );
    const visualGrammar = await page.evaluate(() => {
      const style = (selector) =>
        getComputedStyle(document.querySelector(selector));
      return {
        railBorder: style(".activity-bar").borderRightWidth,
        tabStripBorder: style(".tabs").borderBottomWidth,
        tabRadius: style(".tab-wrap.active").borderRadius,
        groupRadius: style(".information-group").borderRadius,
        groupBorder: style(".information-group").borderTopWidth,
      };
    });
    const activityButtons = page.locator(".activity-bar button");
    assert.equal(await activityButtons.count(), 7);
    for (let index = 0; index < 7; index += 1)
      assert.equal(await activityButtons.nth(index).getAttribute("title"), null);
    const environmentActivity = page.getByRole("button", {
      name: "Environment perspective",
    });
    await environmentActivity.hover();
    const tooltipGeometry = await environmentActivity.locator("span").evaluate((element) => {
      const rect = element.getBoundingClientRect();
      return { width: rect.width, height: rect.height, left: rect.left };
    });
    assert(tooltipGeometry.width > tooltipGeometry.height);
    assert(tooltipGeometry.left >= 44);
    assert.deepEqual(visualGrammar, {
      railBorder: "0px",
      tabStripBorder: "1px",
      tabRadius: "0px",
      groupRadius: "10px",
      groupBorder: "0px",
    });
    await geometry("workbench");
    log(
      "Start Center, single-view Case composition and menu-owned Case navigation",
    );
    log(
      "Seven Case perspectives, Context Panel modes, local tabs and bottom tools",
    );
    log("Panel resize/collapse, icon-only rail tooltip and keyboard focus");
    log(
      "Tonal surfaces, attached work tabs and restrained group radius replace continuous border grids",
      {
        visual_grammar: visualGrammar,
      },
    );
    await load("?fixture=unknown");
    await page.getByText("No replacement data has been loaded.").waitFor();
    log("Unknown fixture refuses without silent fallback");
  }

  const views = [
    { name: "start-center", kind: "start", prepare: () => load() },
    {
      name: "new-case",
      kind: "setup",
      prepare: () => load("?view=new&focus=sources"),
    },
    {
      name: "overview",
      kind: "workbench",
      prepare: () => load("?fixture=ordinary"),
    },
    {
      name: "environment",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=developer");
        await selectPerspective("Environment");
      },
    },
    {
      name: "knowledge",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=ordinary");
        await selectPerspective("Knowledge");
      },
    },
    {
      name: "memory-timeline",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=execution&snapshot=2");
        await selectPerspective("Memory");
      },
    },
    {
      name: "memory-graph",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=execution&snapshot=3");
        await selectPerspective("Memory");
        await page.getByRole("tab", { name: "Graph", exact: true }).click();
      },
    },
    {
      name: "authority",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=execution&snapshot=3");
        await selectPerspective("Authority");
      },
    },
    {
      name: "work",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=execution&snapshot=3");
        await selectPerspective("Work");
      },
    },
    {
      name: "compute",
      kind: "workbench",
      prepare: async () => {
        await load("?fixture=developer&snapshot=3");
        await selectPerspective("Compute");
      },
    },
  ];
  const captures = [];
  async function capture(view, width, height) {
    await page.setViewportSize({ width, height });
    await view.prepare();
    const dimensions = await geometry(view.kind);
    const file = path.join(output, `${view.name}-${width}x${height}.png`);
    const first = await page.screenshot({
      path: file,
      animations: "disabled",
      caret: "hide",
    });
    await view.prepare();
    const second = await page.screenshot({
      animations: "disabled",
      caret: "hide",
    });
    assert(
      first.equals(second),
      `${view.name} fixture render is not deterministic`,
    );
    const record = {
      view: view.name,
      width,
      height,
      file,
      sha256: createHash("sha256").update(first).digest("hex"),
      dimensions,
    };
    captures.push(record);
    log("Deterministic fixture screenshot and desktop geometry", record);
  }
  for (const view of views) await capture(view, 1440, 900);
  if (args.includes("--matrix")) {
    for (const [name, width, height] of [
      ["start-center", 1280, 800],
      ["overview", 1280, 800],
      ["memory-graph", 1728, 1117],
      ["work", 1920, 1080],
    ])
      await capture(
        views.find((view) => view.name === name),
        width,
        height,
      );
  }
  assert.deepEqual(errors, []);
  assert.deepEqual([...requests], [new URL(base).origin]);
  const manifest = {
    run_id: run,
    url: base,
    browser: browser.version(),
    fixture_provenance:
      "Authored synthetic information architecture v2; no live Case/runtime data",
    screenshots: captures,
    requests: [...requests],
  };
  await writeFile(
    path.join(output, "screenshots.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  log("Browser information architecture verification", {
    screenshots: captures.length,
    browser: browser.version(),
    external_requests: 0,
    browser_errors: errors.length,
  });
} finally {
  await browser.close();
}
