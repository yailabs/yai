import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const base = process.env.STUDIO_TEST_URL ?? 'http://127.0.0.1:1422';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium', headless: true, args: ['--no-sandbox', '--disable-gpu'] });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
page.setDefaultTimeout(8000);
const errors = [];
page.on('pageerror', error => errors.push(String(error)));
page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
try {
  await page.goto(`${base}/?gallery=1`);
  await page.evaluate(async () => {
    document.getElementById('root').style.display = 'none';
    const [{ default: { createElement } }, { default: { createRoot } }, { WorkbenchKernel }, { WorkbenchRegistry }, { PlatformServices }, { createHostServices }, { registerContributions }, { builtInContributions }, { FixtureClient }, { fixtureToCasePresentation }] = await Promise.all([
      import('/node_modules/.vite/deps/react.js'), import('/node_modules/.vite/deps/react-dom_client.js'), import('/src/workbench/kernel/WorkbenchKernel.tsx'), import('/src/workbench/kernel/registry.ts'), import('/src/platform/services.ts'), import('/src/platform/host.ts'), import('/src/workbench/kernel/contributions.ts'), import('/src/contrib/builtins.ts'), import('/src/clients/fixture.ts'), import('/src/clients/dataSource.ts'),
    ]);
    const workspace = fixtureToCasePresentation(new FixtureClient().workspace('developer'));
    workspace.case.case_ref = 'case:material-lifecycle-test'; workspace.case.generation = 7;
    workspace.presentation.materials = [];
    workspace.environment.files = await Promise.all(['a', 'b', 'c'].map(async (name) => {
      const bytes = new TextEncoder().encode(`EXACT_SENTINEL_${name.toUpperCase()}`);
      const digest = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map(byte => byte.toString(16).padStart(2, '0')).join('');
      return { id: `file:${name}`, source_ref: name === 'c' ? 'source:other' : 'source:shared', source_label: name, revision_ref: `revision:${name}`, path: `${name}/README.md`, digest: `sha256:${digest}`, bytes: bytes.length, media_type: 'text/markdown', backing: { posture: 'retained' } };
    }));
    window.testReads = []; window.testPending = [];
    const platform = new PlatformServices(createHostServices(false));
    const registry = new WorkbenchRegistry(); registerContributions(builtInContributions, { platform, workbench: registry });
    const host = document.createElement('div'); host.className = 'live-studio case-attached'; document.body.appendChild(host);
    const root = createRoot(host);
    const readMaterial = (request) => { window.testReads.push(request); return new Promise(resolve => window.testPending.push({ request, resolve })); };
    window.completeRead = (path, corrupt = false) => {
      const index = window.testPending.findIndex(entry => entry.request.path === path);
      if (index < 0) throw new Error(`No pending read for ${path}`);
      const {request, resolve} = window.testPending.splice(index, 1)[0];
      const file = workspace.environment.files.find(file => file.path === path);
      const content = corrupt ? 'WRONG_SENTINEL_C' : (file.testContent ?? `EXACT_SENTINEL_${path[0].toUpperCase()}`);
      resolve({ operation_ref: 'material.read', result_state: 'success', correlation_ref: `test:${path}`, data: { ...file, case_ref: request.case_ref, generation: request.expected_generation, encoding: 'utf-8', content } });
    };
    window.renderTest = () => root.render(createElement(WorkbenchKernel, { workspace: {...workspace}, platform, registry, stream: 'fixture', readMaterial, refresh() {}, openCaseSwitcher() {} }));
    window.advanceRevision = async () => {
      const file = workspace.environment.files[0];
      file.testContent = 'NEW_AUTHORITATIVE_A'; file.bytes = new TextEncoder().encode(file.testContent).length;
      file.digest = 'sha256:' + [...new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(file.testContent)))].map(byte=>byte.toString(16).padStart(2,'0')).join('');
      file.revision_ref = 'revision:a2'; workspace.case = {...workspace.case, generation: 8};
      window.renderTest();
    };
    window.renderTest();
  });
  await page.locator('.live-rail button[aria-label="Environment"]').click();
  const file = name => page.locator('.environment-files button').filter({ hasText: 'README.md' }).nth(name.charCodeAt(0) - 97);
  const complete = (name, corrupt=false) => page.evaluate(({name,corrupt})=>window.completeRead(`${name}/README.md`,corrupt), {name,corrupt});
  const pending = name => page.waitForFunction(name => window.testPending.some(entry=>entry.request.path === `${name}/README.md`), name);
  const editor = page.locator('.cm-content');
  await file('a').click(); await pending('a');
  await file('b').click(); await pending('b'); await complete('a');
  assert.equal(await editor.count(), 0, 'late A rendered under B');
  await file('c').click(); await pending('c'); await complete('b');
  assert.equal(await editor.count(), 0, 'late B rendered under C');
  await complete('c'); await editor.waitFor(); assert.equal(await editor.textContent(), 'EXACT_SENTINEL_C');
  await file('a').click(); await pending('a'); await complete('a'); await editor.waitFor();
  assert.equal(await editor.textContent(), 'EXACT_SENTINEL_A');
  await editor.fill('EXACT_SENTINEL_A local edit'); await page.locator('.dirty-mark').waitFor();
  await file('b').click(); await pending('b'); await complete('b'); await editor.waitFor();
  assert.equal(await editor.textContent(), 'EXACT_SENTINEL_B');
  await page.getByRole('tab').filter({has:page.locator('.dirty-mark')}).click(); await pending('a'); await complete('a'); await editor.waitFor();
  assert.equal(await editor.textContent(), 'EXACT_SENTINEL_A local edit');
  await editor.click(); await page.keyboard.press('Control+z');
  assert.equal(await editor.textContent(), 'EXACT_SENTINEL_A', 'undo history was lost on tab change');
  await editor.fill('LOCAL_DIRTY_A');
  await page.evaluate(()=>window.advanceRevision()); await pending('a'); await complete('a');
  await page.getByText('Stale',{exact:true}).waitFor();
  assert.equal(await editor.textContent(), 'LOCAL_DIRTY_A');
  await page.getByRole('button',{name:'Reload',exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('.cm-content')?.textContent === 'NEW_AUTHORITATIVE_A');
  await file('b').click(); await pending('b'); await complete('b', true);
  await page.getByText(/Exact material identity mismatch/).waitFor();
  assert.equal(await editor.count(), 0, 'corrupted equal-length bytes were accepted');
  const readCount = await page.evaluate(()=>window.testReads.length);
  await page.waitForTimeout(300);
  assert.equal(await page.evaluate(()=>window.testReads.length), readCount, 'presentation render restarted material reads');
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({run_id:'studio-material-lifecycle',result:'PASS',reads:readCount,proof:'A→B→C late reads, same names/directories/source, dirty isolation, retained undo, incoming revision, byte digest refusal, bounded reads',lane:'authored asynchronous application-response component test'}));
} finally { await browser.close(); }
