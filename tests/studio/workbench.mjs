// Separate Studio browser proof. Never part of the backend Make graph.
// Uses the Playwright library only; no test runner or downloaded browser needed.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { createHash } from "node:crypto";

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
  option("--output", path.join(root, "build/studio-shell")),
);
const run = option("--run", "studio-shell-browser");
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
async function load(scenario) {
  await page.mouse.move(0, 0);
  await page.goto(`${base}/?fixture=${scenario}`, { waitUntil: "networkidle" });
  await page.evaluate(() => document.fonts.ready);
}
async function geometry() {
  const dimensions = await page.evaluate(() => {
    const rect = (selector) => {
      const r = document.querySelector(selector).getBoundingClientRect();
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
      body: document.body.scrollWidth,
      width: innerWidth,
      height: innerHeight,
      center: rect(".central-column"),
      sidebar: rect(".sidebar"),
      right: rect(".conversation"),
      tools: rect(".bottom-panel"),
      surface: rect(".work-surface"),
      footer: rect(".status-bar"),
    };
  });
  assert(dimensions.body <= dimensions.width, JSON.stringify(dimensions));
  assert(dimensions.center.w >= 390, JSON.stringify(dimensions));
  assert(dimensions.surface.h >= 260, JSON.stringify(dimensions));
  assert(
    dimensions.sidebar.right <= dimensions.center.x &&
      dimensions.center.right <= dimensions.right.x,
    JSON.stringify(dimensions),
  );
  assert(
    dimensions.tools.bottom <= dimensions.footer.y + 1 &&
      dimensions.right.bottom <= dimensions.footer.y + 1,
    JSON.stringify(dimensions),
  );
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
  return after;
}
try {
  if (!args.includes("--screenshots-only")) {
    await load("ordinary");
    await page
      .getByRole("button", { name: "Sources perspective", exact: true })
      .click();
    assert.equal(
      await page
        .getByRole("button", { name: "Sources perspective", exact: true })
        .getAttribute("aria-pressed"),
      "true",
    );
    await page
      .locator("#case-sidebar")
      .getByRole("button", { name: "Services agreement", exact: true })
      .click();
    assert.equal(
      await page
        .getByRole("tab", { name: "Services agreement", exact: true })
        .getAttribute("aria-selected"),
      "true",
    );
    await page.getByRole("tab", { name: /Discussion brief/ }).click();
    await page
      .getByRole("tab", { name: /Discussion brief/ })
      .press("ArrowRight");
    assert.equal(
      await page
        .getByRole("tab", { name: "Services agreement", exact: true })
        .getAttribute("aria-selected"),
      "true",
    );
    await page
      .getByRole("button", { name: "Close Services agreement", exact: true })
      .click();
    assert.equal(
      await page
        .getByRole("tab", { name: /Discussion brief/ })
        .getAttribute("aria-selected"),
      "true",
    );
    await page
      .getByRole("button", { name: "Close Discussion brief", exact: true })
      .click();
    await page.getByRole("button", { name: /Reopen Discussion brief/ }).click();
    await page.getByLabel("Material content", { exact: true }).focus();
    await page.keyboard.press("End");
    await page.waitForFunction(
      () => document.querySelector(".material-scroll").scrollTop > 0,
    );
    await page.screenshot({
      path: path.join(output, "ordinary-scrolled-focus.png"),
      animations: "disabled",
      caret: "hide",
    });
    await page
      .getByRole("button", { name: "Files perspective", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Work perspective", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Case perspective", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Sources perspective", exact: true })
      .hover();
    await page.screenshot({
      path: path.join(output, "ordinary-hover.png"),
      animations: "disabled",
      caret: "hide",
    });
    log(
      "Activity/item selection; tab selection, arrow navigation, close, empty state and reopen",
    );

    const left = await drag("Resize Case explorer", 40, 0);
    await drag("Resize conversation", -35, 0);
    const bottom = await drag("Resize bottom panel", 0, -55);
    await geometry();
    const splitter = page.getByRole("separator", {
      name: "Resize Case explorer",
      exact: true,
    });
    await splitter.focus();
    await splitter.press("ArrowLeft");
    assert.equal(
      Number(await splitter.getAttribute("aria-valuenow")),
      left - 10,
    );
    await splitter.press("End");
    assert.equal(
      await splitter.getAttribute("aria-valuenow"),
      await splitter.getAttribute("aria-valuemax"),
    );
    await splitter.press("Home");
    assert.equal(
      await splitter.getAttribute("aria-valuenow"),
      await splitter.getAttribute("aria-valuemin"),
    );
    await page
      .getByRole("button", { name: "Toggle Case explorer", exact: true })
      .click();
    assert.equal(await page.locator("#case-sidebar").count(), 0);
    await page
      .getByRole("button", { name: "Toggle Case explorer", exact: true })
      .click();
    log(
      "Pointer resize on all three panels; keyboard resize and bounds; sidebar collapse",
    );

    await page.getByRole("tab", { name: "Terminal", exact: true }).click();
    assert.equal(
      await page
        .getByText("Terminal host not attached in fixture mode", {
          exact: true,
        })
        .count(),
      1,
    );
    assert.equal(
      await page
        .locator(
          "#tools-panel input, #tools-panel textarea, #tools-panel [contenteditable]",
        )
        .count(),
      0,
    );
    await page.keyboard.press("Control+j");
    assert.equal(await page.locator("#case-tools").count(), 0);
    await page.keyboard.press("Control+j");
    assert.equal(
      await page
        .getByRole("tab", { name: "Terminal", exact: true })
        .getAttribute("aria-selected"),
      "true",
    );
    assert.equal(
      Math.round((await page.locator("#case-tools").boundingBox()).height),
      bottom,
    );
    for (const tab of ["Output", "Executions", "Evidence", "Problems"])
      await page.getByRole("tab", { name: tab, exact: true }).click();
    assert.equal(
      await page
        .getByText("No problems in this fixture snapshot", { exact: true })
        .count(),
      1,
    );
    await page
      .getByLabel("Local draft", { exact: false })
      .fill("Unsubmitted local note");
    await page
      .getByRole("button", { name: "Collapse conversation", exact: true })
      .click();
    assert.equal(await page.locator("#case-conversation").count(), 0);
    await page
      .getByRole("button", { name: "Toggle conversation", exact: true })
      .click();
    assert.equal(
      await page.locator("#local-draft").inputValue(),
      "Unsubmitted local note",
    );
    await page.keyboard.press("Control+Shift+b");
    assert.equal(await page.locator("#case-conversation").count(), 0);
    await page.keyboard.press("Control+Shift+b");
    await page.getByRole("button", { name: "Clear", exact: true }).click();
    log(
      "Bottom tabs/collapse/size memory; terminal non-interactivity; conversation collapse/draft preservation; shortcuts",
    );

    await page
      .getByRole("combobox", { name: "Fixture scenario" })
      .selectOption("execution");
    assert.match(page.url(), /fixture=execution/);
    assert.equal(
      await page
        .getByRole("heading", { name: "Release handoff", exact: true })
        .count(),
      1,
    );
    await page
      .getByRole("button", { name: "Inspect review", exact: true })
      .click();
    assert.equal(
      await page
        .getByRole("tab", { name: "Publication review", exact: true })
        .getAttribute("aria-selected"),
      "true",
    );
    assert.equal(
      await page.getByRole("button", { name: /approve|publish|send/i }).count(),
      0,
    );
    await page
      .getByRole("button", { name: "Providers perspective", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Provider context", exact: true })
      .click();
    assert.equal(
      await page
        .getByText("Not attached · fixture mode", { exact: true })
        .count(),
      1,
    );
    await page.goBack();
    assert.match(page.url(), /fixture=ordinary/);
    await page.getByRole("combobox", { name: "Fixture scenario" }).focus();
    await page.keyboard.press("Tab");
    const focus = await page.evaluate(() => ({
      name: document.activeElement.getAttribute("aria-label"),
      outline: getComputedStyle(document.activeElement).outlineStyle,
    }));
    assert.equal(focus.name, "Toggle Case explorer");
    assert.equal(focus.outline, "solid");
    log(
      "Scenario switching/history; review navigation without authority controls; provider context; keyboard focus",
      focus,
    );

    await load("unknown");
    assert.equal(
      await page
        .getByRole("heading", { name: "Unknown fixture: unknown" })
        .count(),
      1,
    );
    assert.equal(await page.locator(".workbench").count(), 0);
    log(
      "Unknown scenario refuses instead of silently loading replacement data",
    );
  }

  const viewports = args.includes("--matrix")
    ? [
        [1280, 800],
        [1440, 900],
        [1728, 1117],
        [1920, 1080],
      ]
    : [[1440, 900]];
  const screenshots = [];
  for (const [width, height] of viewports) {
    await page.setViewportSize({ width, height });
    for (const scenario of ["ordinary", "developer", "execution"]) {
      await load(scenario);
      const dimensions = await geometry();
      const file = path.join(output, `${scenario}-${width}x${height}.png`);
      const first = await page.screenshot({
        path: file,
        animations: "disabled",
        caret: "hide",
      });
      await load(scenario);
      const second = await page.screenshot({
        animations: "disabled",
        caret: "hide",
      });
      assert(
        first.equals(second),
        `Non-deterministic rendering: ${scenario} ${width}x${height}`,
      );
      screenshots.push({
        scenario,
        width,
        height,
        file,
        sha256: createHash("sha256").update(first).digest("hex"),
      });
      log("Deterministic fixture screenshot and desktop geometry", {
        scenario,
        width,
        height,
        file,
        dimensions,
      });
    }
  }
  assert.deepEqual(
    errors,
    [],
    "Browser errors or unexpected external/mutating requests",
  );
  await writeFile(
    path.join(output, "screenshots.json"),
    JSON.stringify(
      {
        run_id: run,
        url: base,
        browser: browser.version(),
        fixture_provenance: "Authored synthetic v1; no live Case/runtime data",
        screenshots,
        requests: [...requests],
      },
      null,
      2,
    ),
  );
  log("Browser shell verification", {
    screenshots: screenshots.length,
    browser: browser.version(),
    external_requests: 0,
    browser_errors: 0,
  });
} finally {
  await context.close();
  await browser.close();
}
