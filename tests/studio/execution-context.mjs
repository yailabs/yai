// Real Host, read-only execution inspection. Called with a disposable profile
// by context_capacity.py; no browser-to-provider transport.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import net from 'node:net';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const profile = JSON.parse(await readFile(process.env.STUDIO_CONTEXT_PROFILE, 'utf8'));
const discovery = JSON.parse(await readFile(path.join(profile.home, 'run/host/discovery.json'), 'utf8'));
assert.equal(discovery.yai_home, profile.home);
const calls = [];
function rpc(request) {
  assert.equal(request.operation_ref, 'execution.get');
  return new Promise((resolve, reject) => {
    const socket = net.createConnection(discovery.endpoint); let buffered = '', ready = false;
    socket.setTimeout(10000, () => socket.destroy(new Error('Host timeout')));
    socket.on('error', reject);
    socket.on('connect', () => socket.write(JSON.stringify({ kind: 'handshake', protocol: discovery.protocol,
      client_id: `context-ui:${process.pid}:${calls.length}`, client_kind: 'qualification', pid: process.pid,
      yai_home_identity: discovery.yai_home_identity }) + '\n'));
    socket.on('data', bytes => {
      buffered += bytes.toString(); let end;
      while ((end = buffered.indexOf('\n')) >= 0) {
        const value = JSON.parse(buffered.slice(0, end)); buffered = buffered.slice(end + 1);
        if (!ready && value.kind === 'handshake') { ready = true; socket.write(JSON.stringify({kind: 'application_request', request}) + '\n'); }
        else if (value.kind === 'application_response') { calls.push({request, result:value.result}); socket.end(); resolve(value.result); }
        else if (value.kind === 'error') socket.destroy(new Error(JSON.stringify(value)));
      }
    });
  });
}
const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
try {
  const page = await browser.newPage(); const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.exposeFunction('inspectHost', rpc);
  await page.goto(process.env.STUDIO_TEST_URL ?? 'http://127.0.0.1:1433/?gallery=1');
  await page.evaluate(async profile => {
    document.getElementById('root').style.display = 'none';
    const [{default:React}, {default:ReactDOM}, {ExecutionContext}, {LiveClient}] = await Promise.all([
      import('/node_modules/.vite/deps/react.js'), import('/node_modules/.vite/deps/react-dom_client.js'),
      import('/src/contrib/case/ExecutionContext.tsx'), import('/src/clients/live.ts')]);
    window.__TAURI__ = {core:{invoke: (_command, args) => window.inspectHost(args.request)}};
    const client = new LiveClient();
    const application = {observeConversation: input => client.observeConversation(input)};
    const container = document.createElement('div');container.className='conversation-view';
    container.style.cssText='width:min(430px,100%);height:100vh;overflow:auto;padding:16px';document.body.appendChild(container);
    const root = ReactDOM.createRoot(container);
    window.renderContextAtGeneration = generation => root.render(React.createElement(ExecutionContext, {
      application, execution:profile.execution, generation}));
    window.renderContextAtGeneration(profile.execution.observed_generation);
  }, profile);
  assert.equal(calls.length, 0, 'Context is inspected only on explicit request');
  await page.getByRole('button', {name:'Inspect model context',exact:true}).click();
  await page.getByText('Fits observed capacity',{exact:true}).waitFor();
  assert.equal(calls.length, 1);
  const observation=calls[0].result.data.prepared_context.invocations[0].input_observation;
  assert.equal(observation.serialized_request_digest, profile.digest);
  assert.equal(calls[0].request.input.include_context, true);
  await page.getByText('Required state, recalled evidence and omissions',{exact:true}).click();
  await page.getByText('Exact frame and transport evidence',{exact:true}).click();
  await page.evaluate(()=>document.querySelector('.conversation-view').scrollTop=0);
  for (const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
    await page.setViewportSize({width,height});
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth), false);
    await page.screenshot({path:`${process.env.STUDIO_CONTEXT_PROFILE}.${width}x${height}.png`});
  }
  await page.evaluate(generation=>window.renderContextAtGeneration(generation-1),profile.execution.observed_generation);
  await page.getByRole('alert').filter({hasText:'Context changed.'}).waitFor();
  assert.equal(await page.getByText('Fits observed capacity',{exact:true}).count(),0,'Stale context must be withheld');
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({result:'PASS',proof:['explicit real Host read','exact wire digest','capacity and archived context','four viewport matrix']}));
} finally {
  await writeFile(`${process.env.STUDIO_CONTEXT_PROFILE}.ui-exchanges.json`,JSON.stringify(calls,null,2));
  await browser.close();
}
