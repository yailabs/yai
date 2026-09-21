import { createRequire } from 'node:module';
import path from 'node:path';
import assert from 'node:assert/strict';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', headless: true, args: ['--no-sandbox', '--disable-gpu'] });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
page.setDefaultTimeout(8000);
const errors = [];
page.on('pageerror', error => errors.push(String(error)));
page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
try {
  await page.goto(`${process.env.STUDIO_TEST_URL ?? 'http://127.0.0.1:1422'}/?gallery=1&fixture=developer`);
  await page.evaluate(async () => {
    document.getElementById('root').style.display = 'none';
    const [{ default: { createElement } }, { default: { createRoot } }, { StudioApplication }, { WorkbenchRegistry }, { PlatformServices }, { createHostServices }, { registerContributions }, { builtInContributions }, { FixtureClient }, { FixtureDataSource }, { LiveClient }] = await Promise.all([
      import('/node_modules/.vite/deps/react.js'), import('/node_modules/.vite/deps/react-dom_client.js'), import('/src/app/StudioApplication.tsx'), import('/src/workbench/kernel/registry.ts'), import('/src/platform/services.ts'), import('/src/platform/host.ts'), import('/src/workbench/kernel/contributions.ts'), import('/src/contrib/builtins.ts'), import('/src/clients/fixture.ts'), import('/src/clients/dataSource.ts'), import('/src/clients/live.ts'),
    ]);
    const source = new FixtureDataSource(new FixtureClient());
    const summary = source.caseSummary.bind(source);
    window.refuseSnapshot = false;
    source.caseSummary = (...args) => window.refuseSnapshot ? Promise.resolve({operation_ref:'case.summary',correlation_ref:'test:refusal',result_state:'transport_unavailable',error:{code:'transport_lost',safe_message:'Test connection lost'}}) : summary(...args);
    const platform = new PlatformServices(createHostServices(false));
    const registry = new WorkbenchRegistry(); registerContributions(builtInContributions, {platform, workbench:registry});
    const host = document.createElement('div'); document.body.appendChild(host);
    createRoot(host).render(createElement(StudioApplication, {dataSource:source,platform,registry}));
    // Transport rejection stays a typed refusal; no implicit retry.
    let calls=0;
    window.__TAURI__ = {core:{invoke:async()=>{ calls++; throw new Error('dropped'); }}};
    const result = await new LiveClient().caseSummary('case:transport-test');
    window.transportProof = {state:result.result_state,calls};
    delete window.__TAURI__;
  });
  assert.deepEqual(await page.evaluate(()=>window.transportProof), {state:'transport_unavailable',calls:1});
  await page.locator('.live-rail button[aria-label="Environment"]').click();
  await page.locator('.environment-files').getByRole('button',{name:'result_access.rs',exact:true}).click();
  await page.locator('.cm-content').fill('LOCAL_DRAFT_DURING_FAILURE');
  await page.locator('.dirty-mark').waitFor();
  await page.evaluate(()=>window.refuseSnapshot=true);
  await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
  await page.getByRole('alert').getByText(/Test connection lost/).waitFor();
  assert.equal(await page.locator('.cm-content').textContent(),'LOCAL_DRAFT_DURING_FAILURE');
  await page.locator('.titlebar-case').click();
  await page.getByRole('dialog',{name:'Open Case'}).getByRole('button',{name:/Contract review/}).click();
  await page.getByRole('alert').waitFor();
  assert.equal(await page.locator('.cm-content').textContent(),'LOCAL_DRAFT_DURING_FAILURE');
  await page.evaluate(()=>window.refuseSnapshot=false);
  await page.getByRole('alert').getByRole('button',{name:'Retry',exact:true}).click();
  await page.locator('.titlebar-case').getByText('Contract review').waitFor();
  await page.locator('.titlebar-case').click();
  await page.getByRole('dialog',{name:'Open Case'}).getByRole('button',{name:/Runtime qualification/}).click();
  await page.locator('.cm-content').waitFor();
  assert.equal(await page.locator('.cm-content').textContent(),'LOCAL_DRAFT_DURING_FAILURE');
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({run_id:'studio-attachment-lifecycle',result:'PASS',proof:'IPC rejection is typed and not retried; failed refresh/open preserve drafts; retry opens requested Case; returning restores draft',lane:'authored application-response component test'}));
} finally { await browser.close(); }
